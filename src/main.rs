//  rasmx86_64  -   main.rs
//  -----------------------
//  made by matissoss
//  licensed under MPL 2.0
//  -----------------------

//  global imports go here

use std::{
    path::PathBuf,
    process
};

// local imports go here

mod frontend;
mod cli     ;
mod help    ;
mod global  ;

use frontend::parser::Parser;
use cli ::{
    CLI,
    Cli
};
use help::Help;

// start

fn main(){
    let cli = &*CLI;
    cli.verbose("src/main.rs", "main", "initialized CLI");

    loop {
        let mut buffer = String::new();
        print!("> ");
        std::io::stdin().read_line(&mut buffer).unwrap();
        let line = Parser::parse_line(&buffer);
        println!("{:?}", line);
    }

    /*
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

    parse_file   (&infile);
    assemble_file(&outfile);
    */
    process::exit(0);
}

fn parse_file(inpath: &PathBuf){

}

fn assemble_file(outpath: &PathBuf){

}

fn extend_path(pathbuf: &str) -> PathBuf{
    if pathbuf.starts_with('~'){
        if let Some(hdir) = Cli::home_dir(){
            return PathBuf::from(format!("{:?}/{}", hdir, pathbuf));
        }
    }
    PathBuf::from(pathbuf)
}
