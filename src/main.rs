//  rasmx86_64  -   main.rs
//  -----------------------
//  made by matissoss
//  licensed under MPL 2.0

//  global imports go here

use std::{
    fs,
    path::PathBuf,
    process
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
    }

    let infile : PathBuf   = if let Some(path) = cli.get_arg("-i"){
        PathBuf::from(path)
    }
    else{
        cli.exit("src/main.rs", "main", "no input file specified; tip: try using (example) = `-i=input.asm`!", 0);
    };
    let outfile : PathBuf   = if let Some(path) = cli.get_arg("-o"){
        PathBuf::from(path)
    }
    else{
        cli.exit("src/main.rs", "main", "no output file specified; tip: try using (example): `-o=file.asm`!", 0);
    };

    let ast = parse_file   (&infile);

    //println!("{:?}", ast);
    assemble_file(ast, &outfile);
    
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
                        if let Some(errs) = pre::chk::check_file(&ast){
                            let mut error_count: usize = 0;
                            for (name, errors) in errs{
                                println!("\n--- {} ---\n", ColString::new(name).set_color(BaseColor::PURPLE));
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
                        println!("{}", e.to_string());
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

fn assemble_file(ast: AST, _outpath: &PathBuf){
    for label in ast.labels{
        let result = comp::compile_label(label);
        for r in result{
            match r {
                comp::CompIns::Compiled(bytes) => {
                    for b in bytes{
                        print!("{:02x} ", b)
                    }
                    print!("| ")
                }
                comp::CompIns::NeedsContext(c) => {
                    println!("NOT COMPILED: {:?}", c)
                }
            }
        }
    }
}
