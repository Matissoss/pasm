//  rasmx86_64   -  tok.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::str::FromStr;
use crate::{
    shr::{
        reg::Register,
        kwd::Keyword,
        mem::Mem
    },
    conf::{
        MEM_START,
        MEM_CLOSE,
        COMMENT_S,
        PREFIX_REG,
        PREFIX_VAL,
    }
};

#[allow(unused)]
#[derive(Debug)]
pub enum Token {
    Register(Register),
    Immediate(i64),
    MemAddr(Mem),
    Keyword(Keyword),
    UnknownMemAddr(String),
    UnknownReg(String),
    UnknownVal(String),
    Unknown(String)
}

pub struct Tokenizer;

impl Tokenizer{
    pub fn tokenize_line(line: &str) -> Vec<Token>{
        let mut tokens : Vec<Token> = Vec::new();
        let mut inside_closure: (bool, char) = (false, ' ');
        let mut tmp_buf : Vec<char> = Vec::new();
        for b in line.as_bytes(){
            let c = *b as char;
            match (inside_closure, c) {
                (_, COMMENT_S) => break,
                ((true, MEM_START), ',') => tmp_buf.push(c),
                ((false, _), ',') => continue,
                ((true, PREFIX_REG|PREFIX_VAL), ',') => continue,
                ((true, MEM_START) , ' '|'\t'|'\n') => continue,
                ((false, ' ')       , ' '|'\t'|'\n') => {
                    if !tmp_buf.is_empty() {
                        let str = String::from_iter(tmp_buf.iter());
                        if let Ok(kwd) = Keyword::from_str(&str){
                            tokens.push(Token::Keyword(kwd));
                        }
                        else {
                            tokens.push(Token::Unknown(str));
                        }
                        tmp_buf = Vec::new();
                    }
                    continue;
                },
                ((false,' '), PREFIX_REG) => {
                    inside_closure = (true, PREFIX_REG);
                },
                ((false,' '), PREFIX_VAL) => {
                    inside_closure = (true, PREFIX_VAL);
                },
                ((true, PREFIX_REG), ' '|'\t'|'\n') => {
                    inside_closure = (false, ' ');
                    if let Ok(reg) = Register::from_str(&String::from_iter(tmp_buf.iter())){
                        tokens.push(Token::Register(reg));
                    }
                    else {
                        tokens.push(Token::UnknownReg(String::from_iter(tmp_buf.iter())));
                    }
                    tmp_buf = Vec::new();
                },
                ((true, PREFIX_VAL), ' '|'\t'|'\n') => {
                    inside_closure = (false, ' ');
                    if let Ok(val) = parse_num(&String::from_iter(tmp_buf.iter())){
                        tokens.push(Token::Immediate(val));
                    }
                    else {
                        tokens.push(Token::UnknownVal(String::from_iter(tmp_buf.iter())));
                    }
                    tmp_buf = Vec::new();
                },
                ((false, _), MEM_START|MEM_CLOSE) => {
                    if !tmp_buf.is_empty(){
                        tokens.push(Token::Unknown(String::from_iter(tmp_buf.iter())));
                    }
                    tmp_buf = Vec::new();
                    inside_closure = (true, c);
                },
                ((true, MEM_START), MEM_CLOSE) => {
                    let val = String::from_iter(tmp_buf.iter());
                    if let Ok(maddr) = Mem::from_str(&val){
                        tokens.push(Token::MemAddr(maddr));
                    }
                    else {
                        tokens.push(Token::UnknownMemAddr(val));
                    }
                    inside_closure = (false, ' ');
                    tmp_buf = Vec::new();
                }
                _ => tmp_buf.push(c)
            }
        }
        if tmp_buf.len() != 0 {
            tokens.push(Token::Unknown(String::from_iter(tmp_buf.iter())));
        }
        return tokens;
    }
}

pub fn parse_num(numb: &str) -> Result<i64, String> {
    let numb_bytes : &[u8] = numb.as_bytes();
    match numb_bytes.len() {
        0 => return Err("Expected value, found nothing!".to_string()),
        1 => {
            if (numb_bytes[0] as char).is_numeric(){
                return Ok((numb_bytes[0] - '0' as u8) as i64);
            }
            else {
                return Ok(numb_bytes[0] as i64);
            }
        },
        2 => {
            if (numb_bytes[0] as char).is_numeric() && (numb_bytes[1] as char).is_numeric(){
                return Ok(numb.parse::<i64>().unwrap());
            }
            else {
                return Err(format!("Couldn't parse into number, value: `{}`", numb));
            }
        },
        _ => {
            if numb.starts_with("0x"){
                let mut int : i64 = 0;
                let mut index : u32 = 0;
                for i in 2..numb.len(){
                    if hexnum(numb_bytes[i] as char) != 0{
                        int += hexnum(numb_bytes[i] as char) as i64 * (16i64.pow(index));
                    }
                    else {
                        return Err(format!("Unknown characted found in hex number: {}, number = {}", 
                                            numb_bytes[i] as char, numb));
                    }
                    index += 1;
                }
                return Ok(int);
            }
            else if numb.starts_with("-0x"){
                let mut int : i64 = 0;
                let mut index : u32 = 0;
                for i in 3..numb.len(){
                    if hexnum(numb_bytes[i] as char) != 0{
                        int += hexnum(numb_bytes[i] as char) as i64 * (16i64.pow(index));
                    }
                    else {
                        return Err(format!("Unknown characted found in hex number: {}, number = {}", 
                                            numb_bytes[i] as char, numb));
                    }
                    index += 1;
                }
                return Ok(-int);
            }
            else if numb.starts_with("0b"){
                let mut uint = 0;
                let mut index = 0;
                for i in 3..numb.len(){
                    if numb_bytes[i] as char == '1'{
                        uint += 1 << index;
                    }
                    else if numb_bytes[i] as char != '0'{
                        return Err(format!("Unknown character found in binary number: {}, number = {}", 
                                            numb_bytes[i] as char, numb));
                    }
                    index += 1;
                }

                return Ok(uint);
            }
            else if numb.starts_with("-0b"){
                let mut int   = 0;
                let mut index = 0;
                
                for i in 3..numb.len(){
                    if numb_bytes[i] as char == '1'{
                        int += 1 << index;
                    }
                    else if numb_bytes[i] as char != '0'{
                        return Err(format!("Unknown character found in binary number: {}", numb));
                    }
                    index += 1;
                }

                return Ok(-int);
            }
            else {
                if let Ok(num) = numb.parse::<i64>(){
                    return Ok(num);
                }
                else{
                    return Err(format!("Unknown Value (not hex nor binary nor decimal): {}", numb));
                }
            }
        }
    }
}
fn hexnum(n: char) -> u8{
    let r = n as u16 - '0' as u16;
    if r < 10{
        return (r & 0xFF).try_into().unwrap();
    }
    else {
        match n {
            'a'|'A' => 10,
            'b'|'B' => 11,
            'c'|'C' => 12,
            'd'|'D' => 13,
            'e'|'E' => 14,
            'f'|'F' => 15,
            _ => 0,
        }
    }
}
