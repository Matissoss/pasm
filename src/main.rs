// rasmx86_64 - src/main.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(clippy::collapsible_else_if)]

//  global imports go here
use std::{
    borrow::Cow,
    fs,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    process,
    rc::Rc,
};

// local imports go here

// rasmx86_64 modules
pub mod core;
pub mod pre;
pub mod shr;

use core::comp;
use pre::lex::Lexer;
use pre::par::Parser;
use pre::tok::Tokenizer;
use shr::ast::AST;

// rasmx86_64 helper utilities
pub mod cli;
pub mod color;
pub mod conf;
pub mod help;

pub use shr::rpanic::switch_panichandler;

use core::obj::{elf32::make_elf32, elf64::make_elf64};

use shr::symbol::{Symbol, SymbolType};

use cli::CLI;
use color::{BaseColor, ColString};
use help::Help;

// start

fn main() {
    switch_panichandler();
    let cli = &*CLI;

    if cli.get_arg("-h").is_some() {
        Help::main_help();
        process::exit(0)
    }

    let infile: PathBuf = if let Some(path) = cli.get_arg("-i") {
        PathBuf::from(path)
    } else {
        cli.exit(
            "src/main.rs",
            "main",
            "no input file specified; tip: try using (example) = `-i=input.asm`!",
            1,
        );
    };
    let outfile: PathBuf = if let Some(path) = cli.get_arg("-o") {
        PathBuf::from(path)
    } else {
        cli.exit(
            "src/main.rs",
            "main",
            "no output file specified; tip: try using (example): `-o=file.asm`!",
            1,
        );
    };

    let ast = parse_file(infile);
    if let Some(form) = cli.get_arg("-f") {
        assemble_file(ast, &outfile, form);
    }

    process::exit(0);
}

fn parse_file(inpath: PathBuf) -> AST<'static> {
    if let Ok(true) = fs::exists(&inpath) {
        if let Ok(buf) = fs::read_to_string(inpath) {
            let mut tokenized_file = Vec::new();
            for line in buf.lines() {
                tokenized_file.push(Tokenizer::tokenize_line(line));
            }

            let lexed = Lexer::parse_file(tokenized_file);

            match Parser::build_tree(lexed) {
                Ok(ast) => {
                    if conf::FAST_MODE {
                        return ast;
                    } else if let Some(errs) = pre::chk::check_ast(&ast) {
                        let mut error_count: usize = 0;
                        for (name, errors) in errs {
                            eprintln!(
                                "\n--- {} ---\n",
                                ColString::new(name).set_color(BaseColor::PURPLE)
                            );
                            for err in errors {
                                error_count += 1;
                                println!("{}", err)
                            }
                        }
                        CLI.exit(
                            "main.rs",
                            "parse_file",
                            &format!(
                                "Assembling ended unsuccesfully with {}!",
                                ColString::new(format!("{} errors", error_count))
                                    .set_color(BaseColor::RED)
                            ),
                            1,
                        );
                    } else {
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
        let mut res = comp::compile_label(&label.inst, astrc_ref.bits.unwrap_or(64));
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
            if let Some(errs) = core::reloc::relocate_addresses(&mut to_write, relocs, &symbols) {
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
    process::exit(0);
}
