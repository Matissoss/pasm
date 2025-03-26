//  rasmx86_64  -   parser.rs
//  -------------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::{
    fs,
    path::PathBuf
};

use crate::{
    frontend::{
        tokenizer   ::{
            Token,
            Tokenizer
        },
    }
};

pub struct Parser;
impl Parser{
    pub fn parse_file(inpath: &PathBuf) -> Result<Vec<Vec<Token>>, String>{
        if let Ok(true) = fs::exists(inpath){
            if let Ok(buf) = fs::read_to_string(inpath){
                let mut tokens : Vec<Vec<Token>> = Vec::new();
                for line in buf.lines(){
                    tokens.push(Self::parse_line(&line));
                }
                return Ok(tokens);
            }
            else {
                return Err(format!("Could not read file {:?}!", inpath));
            }
        }
        else {
            return Err(format!("File Not Found: `{:?}`!", inpath));
        }
    }
    pub fn parse_line(line: &str) -> Vec<Token>{
        return Tokenizer::tokenize_line(line);
    }
}
