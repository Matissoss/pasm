//  rasmx86_64  -   main.rs
//  -----------------------
//  made by matissoss
//  licensed under MPL 2.0
//  -----------------------

//  global imports go here

use std::{
    fs,
    path::PathBuf,
    process
};

// local imports go here

// rasmx86_64 modules
mod pre;
mod shr;

use pre::tok::Tokenizer;
use pre::lex::Lexer;
use pre::par::Parser;

// rasmx86_64 helper utilities
pub mod conf ;
pub mod color;
pub mod cli  ;
pub mod help ;

use cli ::{
    CLI,
    Cli
};
use help::Help;

// start

fn main(){
    let cli = &*CLI;
    cli.verbose("src/main.rs", "main", "initialized CLI");

    if let Some(_) = cli.get_arg("-h"){
        Help::main_help();
    }

    let infile : PathBuf   = if let Some(path) = cli.get_arg("-i"){
        extend_path(path)
    }
    else{
        cli.exit("src/main.rs", "main", "no input file specified; tip: try using (example) = `-i=input.asm`!", 0);
    };
    let outfile : PathBuf   = if let Some(path) = cli.get_arg("-o"){
        extend_path(path)
    }
    else{
        cli.exit("src/main.rs", "main", "no output file specified; tip: try using (example): `-o=file.asm`!", 0);
    };

    let input = fs::read_to_string(infile).unwrap();
    let mut tokens = Vec::new();
    for line in input.lines(){
        tokens.push(Tokenizer::tokenize_line(line));
    }
    for (n, line) in tokens.iter().enumerate(){
        print!("{:05}: ", n);
        for tok in line{
            print!("{:?} ", tok);
        }
        print!("\n");
    }
    //parse_file   (&infile);
    //assemble_file(&outfile);
    
    process::exit(0);
}

#[allow(dead_code)]
fn parse_file(_inpath: &PathBuf){

}

#[allow(dead_code)]
fn assemble_file(_outpath: &PathBuf){

}

#[allow(dead_code)]
fn extend_path(pathbuf: &str) -> PathBuf{
    if pathbuf.starts_with('~'){
        if let Some(hdir) = Cli::home_dir(){
            return PathBuf::from(format!("{:?}/{}", hdir, pathbuf));
        }
    }
    PathBuf::from(pathbuf)
}
