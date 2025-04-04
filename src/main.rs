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
//pub mod core;

use pre::tok::Tokenizer;
use pre::lex::Lexer;
use pre::par::Parser;
use shr::ast::AST;

// rasmx86_64 helper utilities
pub mod conf ;
pub mod color;
pub mod cli  ;
pub mod help ;

use cli ::CLI;
use help::Help;

// start

fn main(){
    let cli = &*CLI;
    cli.verbose("src/main.rs", "main", "initialized CLI");

    if let Some(_) = cli.get_arg("-h"){
        Help::main_help();
    }

    let infile : PathBuf   = if let Some(path) = cli.get_arg("-i"){
        PathBuf::from(path)
    }
    else{
        cli.exit("src/main.rs", "main", "no input file specified; tip: try using (example) = `-i=input.asm`!", 0);
    };
    let _outfile : PathBuf   = if let Some(path) = cli.get_arg("-o"){
        PathBuf::from(path)
    }
    else{
        cli.exit("src/main.rs", "main", "no output file specified; tip: try using (example): `-o=file.asm`!", 0);
    };

    let ast = parse_file   (&infile);

    println!("{:?}", ast);
    //assemble_file(&outfile);
    
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
                            for e in errs{
                                println!("{}", e.to_string());
                            }
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

#[allow(dead_code)]
fn assemble_file(_outpath: &PathBuf){

}
