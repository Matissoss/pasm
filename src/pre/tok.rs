//  rasmx86_64   -  tok.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

use crate::conf::{
    MEM_START,
    MEM_CLOSE,
    COMMENT_S,
};

#[derive(Debug)]
pub enum Token {
    Symbol(char),
    String(String),
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
                (_, ',') => continue,
                ((true, MEM_START) , ' '|'\t'|'\n') => continue,
                ((false, _ )      , ' '|'\t'|'\n') => {
                    if tmp_buf.len() != 0 {
                        tokens.push(Token::String(String::from_iter(tmp_buf.iter())));
                        tmp_buf = Vec::new();
                    }
                },
                ((false, _), MEM_START|MEM_CLOSE) => {
                    tokens.push(Token::String(String::from_iter(tmp_buf.iter())));
                    tokens.push(Token::Symbol(c));
                    tmp_buf = Vec::new();
                    inside_closure = (true, c);
                },
                ((true, MEM_START), MEM_CLOSE) => {
                    tokens.push(Token::String(String::from_iter(tmp_buf.iter())));
                    tokens.push(Token::Symbol(MEM_CLOSE));
                    inside_closure = (false, ' ');
                    tmp_buf = Vec::new();
                }
                _ => tmp_buf.push(c)
            }
        }
        if tmp_buf.len() != 0 {
            tokens.push(Token::String(String::from_iter(tmp_buf.iter())));
        }
        return tokens;
    }
}

#[derive(Debug)]
enum Num{
    UInt(u64),
    Int(i64)
}
pub fn parse_num(numb: &str) -> Result<Num, String> {
    let numb_bytes : &[u8] = numb.as_bytes();
    match numb_bytes.len() {
        0 => return Err("Expected value, found nothing!".to_string()),
        1 => {
            if (numb_bytes[0] as char).is_numeric(){
                return Ok(Num::UInt((numb_bytes[0] - '0' as u8) as u64));
            }
            else {
                return Err(format!("Couldn't parse into number, value: `{}`", numb));
            }
        },
        2 => {
            if (numb_bytes[0] as char).is_numeric() && (numb_bytes[1] as char).is_numeric(){
                return Ok( Num::UInt (((numb_bytes[0] - '0' as u8) + (numb_bytes[1] - '0' as u8)) as u64));
            }
            else {
                return Err(format!("Couldn't parse into number, value: `{}`", numb));
            }
        },
        _ => {
            if numb.starts_with("0x"){
                let mut uint : u64 = 0;
                let mut index : u32 = 0;
                for i in 2..numb.len(){
                    if hexnum(numb_bytes[i] as char) != 0{
                        uint += hexnum(numb_bytes[i] as char) as u64 * (16u64.pow(index));
                    }
                    else {
                        return Err(format!("Unknown characted found in hex number: {}, number = {}", 
                                            numb_bytes[i] as char, numb));
                    }
                    index += 1;
                }
                return Ok(Num::UInt(uint));
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
                return Ok(Num::Int(-int));
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

                return Ok(Num::UInt(uint));
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

                return Ok(Num::Int(-int));
            }
            else {
                if let Ok(num) = numb.parse::<u64>(){
                    return Ok(Num::UInt(num));
                }
                else if let Ok(num) = numb.parse::<i64>(){
                    return Ok(Num::Int(num));
                }
                else{
                    return Err(format!("Unknown Value (not hex nor binary nor decimal): {}", numb));
                }
            }
        }
    }
}
fn hexnum(n: char) -> u8{
    let r = n as u8 - '0' as u8;
    if r >= 0 && r < 10{
        return r;
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
