//  rasmx86_64  -   main.rs
//  -----------------------
//  made by matissoss
//  licensed under MPL 2.0

//  global imports go here

use std::{
    fs::{
        OpenOptions,
        File
    },
    fs,
    path::PathBuf,
    process,
    io::Write,
};

// local imports go here

// rasmx86_64 modules
pub mod pre;
pub mod shr;
pub mod core;

use pre::tok::Tokenizer;
use pre::lex::Lexer;
use pre::par::Parser;
use shr::ast::AST;
use core::comp;

// rasmx86_64 helper utilities
pub mod conf ;
pub mod color;
pub mod cli  ;
pub mod help ;

pub use shr::rpanic::rpanic;

use core::obj::elf::make_elf32;
use shr::symbol::{
    Symbol,
    SymbolType,
    Visibility
};

use color::{
    ColString,
    BaseColor,
};
use cli ::CLI;
use help::Help;

// start

fn main(){
    let cli = &*CLI;
    cli.verbose("src/main.rs", "main", "initialized CLI");
    cli.debug("src/main.rs", "main", &format!("FAST_MODE = {}", conf::FAST_MODE.to_string()));

    if let Some(_) = cli.get_arg("-h"){
        Help::main_help();
        process::exit(0)
    }

    let infile : PathBuf   = if let Some(path) = cli.get_arg("-i"){
        PathBuf::from(path)
    }
    else{
        cli.exit("src/main.rs", "main", "no input file specified; tip: try using (example) = `-i=input.asm`!", 1);
    };
    let outfile : PathBuf   = if let Some(path) = cli.get_arg("-o"){
        PathBuf::from(path)
    }
    else{
        cli.exit("src/main.rs", "main", "no output file specified; tip: try using (example): `-o=file.asm`!", 1);
    };

    let ast = parse_file   (&infile);
    if let Some(form) = cli.get_arg("-f"){
        assemble_file(ast, &outfile, form);
    }
    
    process::exit(0);
}

fn parse_file(inpath: &PathBuf) -> AST{
    if let Ok(true) = fs::exists(inpath){
        if let Ok(buf) = fs::read_to_string(inpath){
            let mut tokenized_file = Vec::new();
            for line in buf.lines(){
                tokenized_file.push(Tokenizer::tokenize_line(line));
            }

            let lexed = Lexer::parse_file(tokenized_file);

            match Parser::build_tree(lexed){
                Ok(ast) => {
                    if conf::FAST_MODE {
                        return ast
                    }
                    else {
                        if let Some(errs) = pre::chk::check_ast(&ast){
                            let mut error_count: usize = 0;
                            for (name, errors) in errs{
                                eprintln!("\n--- {} ---\n", ColString::new(name).set_color(BaseColor::PURPLE));
                                for err in errors{
                                    error_count += 1;
                                    println!("{}", err)
                                }
                            }
                            CLI.exit("main.rs", "parse_file", 
                                &format!("Assembling ended unsuccesfully with {}!", 
                                ColString::new(format!("{} errors", error_count)).set_color(BaseColor::RED)), 
                            1);
                        }
                        else {
                            return ast;
                        }
                    }
                },
                Err(errors) => {
                    for e in errors{
                        eprintln!("{}", e.to_string());
                    }
                }
            }

            CLI.exit("main.rs", "parse_file", "Assembling ended with error!", 1);
        }
        else {
            CLI.exit("main.rs", "parse_file", "Error occured, while reading file!", 1);
        }
    }
    else {
        CLI.exit("main.rs", "parse_file", "Source file doesn't exist!", 1);
    }
}

fn assemble_file(ast: AST, outpath: &PathBuf, form: &str){
    match fs::exists(outpath) {
        Ok(false) => {
            match File::create(outpath){
                Ok(_) => {},
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1);
                }
            }
        },
        Ok(true) => {
            if let Err(why) = fs::remove_file(outpath){
                eprintln!("{}", why);
                process::exit(1);
            }
            if let Err(why) = File::create(outpath){
                eprintln!("{}", why);
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    }
    let file = OpenOptions::new()
        .write(true)
        .open(outpath);
    
    if let Err(why) = file {
        eprintln!("{}", why);
        process::exit(1);
    }
    let mut file = file.unwrap();
    
    let mut symbols = Vec::new();
    let mut relocs  = Vec::new();

    let mut to_write : Vec<u8> = Vec::new();


    for label in &ast.labels{
        let mut symb = Symbol{ 
            name: label.name.clone(), 
            offset: to_write.len() as u32, 
            size: None,
            sindex: 0,
            visibility: Visibility::Global,
            stype: SymbolType::Func,
            content: None,
        };
        
        let mut res = comp::compile_label(label.clone());
        for r in &mut res.1{
            r.offset += to_write.len() as u32;
        }

        relocs  .extend(res.1);
        to_write.extend(res.0);
        symb.size = Some(to_write.len() as u32 - symb.offset);
        symbols.push(symb);
    }

    if form == "baremetal"{
        let filtered_vars = ast.filter_vars();

        for v in filtered_vars{
            let section = comp::compile_section(v.1, v.0 as u16);
            symbols.extend(section.1);
        }

        match core::reloc::relocate_addresses(&mut to_write, relocs, &symbols){
            Some(errs) => {
                for e in &errs{
                    eprintln!("{e}");
                }
                CLI.exit("main.rs", "assemble_file", &format!("Assembling ended with {} errors!", errs.len()), 1);
            },
            None => {}
        }
    }
    else if form == "elf32"{
        let filtered_vars = ast.filter_vars();

        for v in filtered_vars{
            let section = comp::compile_section(v.1, v.0 as u16);
            symbols.extend(section.1);
        }
        to_write = make_elf32(
            &to_write,
            relocs,
            &symbols,
            &outpath,
        );
    }
    if let Err(why) = file.write_all(&to_write){
        eprintln!("{}", why);
        process::exit(1);
    }
}
