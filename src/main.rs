// rasmx86_64 - src/main.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(clippy::collapsible_else_if)]
#![allow(clippy::manual_map)]
#![allow(clippy::to_string_trait_impl)]
#![allow(clippy::while_let_on_iterator)]

//  global imports go here
use std::{
    borrow::Cow,
    fs,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    process,
    rc::Rc,
    time,
};

// local imports go here

// rasmx86_64 modules
pub mod core;
pub mod pre;
pub mod pre_core;
pub mod shr;

use core::comp;
use pre::lex::Lexer;
use pre::par::Parser;
use pre::tok::Tokenizer;
use shr::ast::AST;
use shr::error;

// rasmx86_64 helper utilities
pub mod cli;
pub mod color;
pub mod conf;
pub mod help;

pub use shr::rpanic::switch_panichandler;

use core::obj::{elf32::make_elf32, elf64::make_elf64};

use shr::symbol::{Symbol, SymbolType};

use cli::CLI;
use color::{ColString, Color};
use help::Help;

// start

fn main() {
    switch_panichandler();
    let cli = &*CLI;

    if cli.has_arg("-h") {
        Help::main_help();
        process::exit(0)
    }
    if cli.has_arg("supported-instructions") {
        print_supported_instructions();
        return;
    }
    if cli.has_arg("supported-instructions-raw") {
        print_supported_instructions_raw();
        return;
    }

    let infile: PathBuf = if let Some(path) = cli.get_kv_arg("-i") {
        PathBuf::from(path)
    } else {
        cli.exit(
            "src/main.rs",
            "main",
            "no input file specified; tip: try using (example) = `-i=input.asm`!",
            1,
        );
    };

    let start = if conf::TIME {
        Some(time::SystemTime::now())
    } else {
        None
    };

    let ast = parse_file(&infile);

    if cli.has_arg("check") {
        if conf::TIME {
            let end = time::SystemTime::now();
            println!(
                "Checking {:?} took {}s and ended without errors!",
                infile,
                match end.duration_since(start.unwrap()) {
                    Ok(t) => t.as_secs_f32(),
                    Err(e) => e.duration().as_secs_f32(),
                }
            )
        }
        return;
    }

    let outfile: PathBuf = if let Some(path) = cli.get_kv_arg("-o") {
        PathBuf::from(path)
    } else {
        cli.exit(
            "src/main.rs",
            "main",
            "no output file specified; tip: try using (example): `-o=file.asm`!",
            1,
        );
    };

    if let Some(form) = cli.get_kv_arg("-f") {
        assemble_file(ast, &outfile, form);
        if conf::TIME && cli.has_arg("-t") {
            let end = time::SystemTime::now();
            println!(
                "Assembling {:?} took {}",
                infile,
                ColString::new(format!(
                    "{}s",
                    match end.duration_since(start.unwrap()) {
                        Ok(t) => t.as_secs_f32(),
                        Err(e) => e.duration().as_secs_f32(),
                    }
                ))
                .set_color(Color::RED)
            )
        }
    }
}

fn parse_file(inpath: &PathBuf) -> AST<'static> {
    if let Ok(true) = fs::exists(inpath) {
        if let Ok(buf) = fs::read_to_string(inpath) {
            let mut tokenized_file = Vec::new();
            for line in buf.lines() {
                tokenized_file.push(Tokenizer::tokenize_line(line));
            }

            let lexed = Lexer::parse_file(tokenized_file);
            match Parser::build_tree(lexed) {
                Ok(mut ast) => {
                    ast.file = inpath.to_path_buf();
                    if let Err(why) = pre_core::post_process(&mut ast) {
                        error::print_error(why, &ast.file);
                        process::exit(1);
                    }
                    if conf::FAST_MODE {
                        return ast;
                    } else if let Some(errs) = pre::chk::check_ast(&ast) {
                        let mut error_count: usize = 0;
                        for (name, errors) in errs {
                            eprintln!(
                                "\n--- {}:{} ---\n",
                                &ast.file.to_string_lossy(),
                                ColString::new(name).set_color(Color::PURPLE)
                            );
                            for err in errors {
                                error_count += 1;
                                error::print_error(err, &ast.file);
                            }
                        }
                        CLI.exit(
                            "main.rs",
                            "parse_file",
                            &format!(
                                "Assembling ended unsuccesfully with {}!",
                                ColString::new(format!("{} errors", error_count))
                                    .set_color(Color::RED)
                            ),
                            1,
                        );
                    } else {
                        if !ast.includes.is_empty() {
                            let paths = {
                                let mut v = Vec::new();
                                for p in &ast.includes {
                                    v.push(p.clone())
                                }
                                v
                            };
                            for p in paths {
                                let sast = parse_file(&p);
                                if let Err(why) = ast.extend(sast) {
                                    error::print_error(why, &ast.file);
                                    process::exit(1);
                                }
                            }
                        }
                        return ast;
                    }
                }
                Err(errors) => {
                    for e in errors {
                        eprintln!("{e}");
                    }
                }
            }
            CLI.exit("main.rs", "parse_file", "Assembling ended with error!", 1);
        } else {
            CLI.exit(
                "main.rs",
                "parse_file",
                "Error occured, while reading file!",
                1,
            );
        }
    } else {
        CLI.exit("main.rs", "parse_file", "Source file doesn't exist!", 1);
    }
}

fn assemble_file(mut ast: AST<'static>, outpath: &PathBuf, form: &str) {
    match fs::exists(outpath) {
        Ok(false) => match File::create(outpath) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{err}");
                process::exit(1);
            }
        },
        Ok(true) => {
            if let Err(why) = fs::remove_file(outpath) {
                eprintln!("{why}");
                process::exit(1);
            }
            if let Err(why) = File::create(outpath) {
                eprintln!("{why}");
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
    let file = OpenOptions::new().write(true).open(outpath);

    if let Err(why) = file {
        eprintln!("{why}");
        process::exit(1);
    }

    let mut file = file.unwrap();
    let mut symbols = Vec::new();
    let mut relocs = Vec::new();
    let mut to_write: Vec<u8> = Vec::new();

    ast.make_globals();
    ast.fix_entry();

    let astrc = Rc::new(ast);
    let astrc_ref = Rc::clone(&astrc);

    let filtered_vars = &AST::filter_vars(&astrc_ref.vars);
    for v in filtered_vars {
        let section = comp::compile_section(&v.1, 0, v.0 as u8);
        symbols.extend(section.1);
    }
    for label in &astrc_ref.labels {
        let mut res = comp::compile_label(label, to_write.len());
        let mut symb = Symbol {
            name: Cow::Borrowed(&label.name),
            offset: to_write.len() as u64,
            size: None,
            sindex: 1,
            visibility: label.visibility,
            stype: SymbolType::Func,
            content: None,
            addend: -4,
            addt: 0,
        };
        for r in &mut res.1 {
            r.offset += to_write.len() as u64;
        }
        relocs.extend(res.1);
        to_write.extend(res.0);
        symb.size = Some((to_write.len() as u64 - symb.offset) as u32);
        symbols.push(symb);
    }

    match form {
        "bin" => {
            if let Some(errs) = shr::reloc::relocate_addresses(&mut to_write, relocs, &symbols) {
                for e in &errs {
                    eprintln!("{e}");
                }
                CLI.exit(
                    "main.rs",
                    "assemble_file",
                    &format!("Assembling ended with {} errors!", errs.len()),
                    1,
                );
            }
        }
        "elf32" => {
            let astref = Rc::clone(&astrc);
            symbols.extend(comp::extern_trf(&astref.externs));
            to_write = make_elf32(&to_write, relocs, &symbols, outpath);
            drop(astref);
        }
        "elf64" => {
            let astref = Rc::clone(&astrc);
            symbols.extend(comp::extern_trf(&astref.externs));
            to_write = make_elf64(&to_write, relocs, &symbols, outpath);
            drop(astref);
        }
        _ => CLI.exit(
            "main.rs",
            "assemble_file",
            &format!("Unknown format `{}`!", form),
            1,
        ),
    }
    if let Err(why) = file.write_all(&to_write) {
        eprintln!("Couldn't save output to file: {}", why);
        process::exit(1);
    }
}

use crate::shr::ins::Mnemonic;
fn print_supported_instructions() {
    let ins_count = Mnemonic::__LAST as u16;
    println!("RASM supports {} x86-64 instructions!", ins_count - 1);
    println!("Here's a list of all of them:");
    for idx in (2..ins_count).step_by(3) {
        let ins2 = unsafe { std::mem::transmute::<u16, Mnemonic>(idx - 2).to_string() };
        let ins1 = unsafe { std::mem::transmute::<u16, Mnemonic>(idx - 1).to_string() };
        let ins0 = unsafe { std::mem::transmute::<u16, Mnemonic>(idx).to_string() };
        println!(
            "{ins2}{}{ins1}{}{ins0}",
            " ".repeat(conf::LINE_WIDTH - ins2.len()),
            " ".repeat(conf::LINE_WIDTH - ins1.len()),
        )
    }
    let mut mod_ = ins_count % 3;
    while mod_ != 0 {
        let ins0 = unsafe { std::mem::transmute::<u16, Mnemonic>(ins_count - mod_).to_string() };
        if mod_ - 1 == 0 {
            print!("{ins0}")
        } else {
            print!("{ins0}{}", " ".repeat(conf::LINE_WIDTH - ins0.len()));
        }
        mod_ -= 1;
    }
}
fn print_supported_instructions_raw() {
    for idx in 0..Mnemonic::__LAST as u16 {
        if idx + 1 == Mnemonic::__LAST as u16 {
            print!(
                "{}",
                format!("{:?}", unsafe { std::mem::transmute::<u16, Mnemonic>(idx) })
                    .to_lowercase()
            );
        } else {
            println!(
                "{}",
                format!("{:?}", unsafe { std::mem::transmute::<u16, Mnemonic>(idx) })
                    .to_lowercase()
            );
        }
    }
}
