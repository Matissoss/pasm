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
    fs,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    process, time,
};

// local imports go here

// rasmx86_64 modules
pub mod core;
pub mod obj;
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

fn parse_file(inpath: &PathBuf) -> AST {
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

fn assemble_file(mut ast: AST, outpath: &PathBuf, form: &str) {
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

    ast.fix_entry();
    let mut sections: Vec<&crate::shr::section::Section> = Vec::new();
    for section in &mut ast.sections {
        let prev_len = to_write.len();
        for label in &section.content {
            let mut code = comp::compile_label(label, to_write.len());
            let label_symbol = Symbol {
                name: &label.name,
                offset: to_write.len() as u32,
                size: code.0.len() as u32,
                sindex: 1,
                visibility: label.visibility,
                stype: SymbolType::NoType,
                is_extern: false,
            };
            for reloc in &mut code.1 {
                reloc.offset += to_write.len() as u32;
            }
            relocs.extend(code.1);
            to_write.extend(code.0);
            symbols.push(label_symbol);
        }
        section.size = (to_write.len() - prev_len) as u32;
        sections.push(section);
    }
    match form {
        "bin" => {
            if let Err(err) = shr::reloc::relocate_addresses(&mut to_write, relocs, &symbols) {
                eprintln!("{err}");
                CLI.exit(
                    "main.rs",
                    "assemble_file",
                    "Assembling ended with 1 error!",
                    1,
                );
            }
        }
        "elf32" => {
            symbols.extend(comp::extern_trf(&ast.externs));
            to_write = make_elf32(&to_write, relocs, &symbols, outpath);
        }
        "elf64" => {
            symbols.extend(comp::extern_trf(&ast.externs));
            to_write = make_elf64(&to_write, relocs, &symbols, outpath);
        }
        "elf64-experimental" => {
            symbols.extend(comp::extern_trf(&ast.externs));
            let elf = obj::elf::make_elf(&sections, &to_write, &relocs, &symbols, true);
            if let Err(why) = elf {
                error::print_error(why, &ast.file);
                process::exit(1);
            }
            let elf = elf.unwrap();
            to_write = elf.compile(true);
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
    println!("This version of RASM supports {} mnemonics!", ins_count - 1);
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
