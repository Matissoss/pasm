//  rasmx86_64   -  tok.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::str::FromStr;
use crate::{
    shr::{
        reg::Register,
        kwd::Keyword,
        ins::Instruction,
    },
    conf::{
        MEM_START,
        MEM_CLOSE,
        COMMENT_S,
        PREFIX_REG,
        PREFIX_VAL,
        PREFIX_REF,
        PREFIX_LAB,
        PREFIX_KWD
    }
};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Register(Register),
    Immediate(i64),
    Keyword(Keyword),
    Section(String),

    Comma,

    Instruction(Instruction),
    Label(String),
    LabelRef(String),
    String(String),
    ConstRef(String),
    MemAddr(String),
    
    UnknownKeyword(String),
    UnknownReg(String),
    UnknownVal(String),
    Unknown(String),
}

pub struct Tokenizer;
pub struct Tokens(pub Vec<Token>);

impl Tokenizer{
    pub fn tokenize_line(line: &str) -> Vec<Token>{
        let mut tokens          : Vec<Token>    = Vec::new();
        let mut tmp_buf         : Vec<char>     = Vec::new();
        let mut inside_closure  : (bool, char)  = (false, ' ');
        
        for b in line.as_bytes(){
            let c = *b as char;
            match (inside_closure, c) {
                (_, COMMENT_S) => break,

                ((false, _), '"') => {
                    inside_closure = (true, '"');
                }
                ((true, '"'), '"') => {
                    tokens.push(Token::String(String::from_iter(tmp_buf.iter())));
                    tmp_buf = Vec::new();
                    inside_closure = (false, ' ');
                }

                ((true, '"'), c  ) => tmp_buf.push(c),
                ((false, _), ':') => {
                    tokens.push(Token::Label(String::from_iter(tmp_buf.iter())));
                    tmp_buf = Vec::new();
                },

                ((false, _), '.')        => inside_closure = (true, '.'),
                ((false, _), PREFIX_KWD) => inside_closure = (true, PREFIX_KWD),
                ((false, _), PREFIX_REF) => inside_closure = (true, PREFIX_REF),
                ((false, _), PREFIX_LAB) => inside_closure = (true, PREFIX_LAB),

                ((true, MEM_START), ',') => tmp_buf.push(c),
                
                ((true, PREFIX_REG|PREFIX_VAL), ',') => {
                    if tmp_buf.is_empty() == false {
                        tokens.push(Token::make_from(inside_closure.1, &String::from_iter(tmp_buf.iter())));
                        tmp_buf = Vec::new();
                    }
                    tokens.push(Token::Comma)
                },
                ((false, _), ',') => {
                    if tmp_buf.is_empty() == false {
                        tokens.push(Token::make_from(inside_closure.1, &String::from_iter(tmp_buf.iter())));
                        tmp_buf = Vec::new();
                    }
                    tokens.push(Token::Comma)
                },
                
                ((true, MEM_START), ' ') => continue,
                
                ((false, _), ' '|'\t'|'\n') => {
                    if tmp_buf.is_empty() == false {
                        tokens.push(Token::make_from(inside_closure.1, &String::from_iter(tmp_buf.iter())));
                        tmp_buf = Vec::new();
                    }
                    continue;
                },
                
                ((false, _), PREFIX_REG) => {
                    inside_closure = (true, PREFIX_REG);
                }
                ((false, _), PREFIX_VAL) => {
                    inside_closure = (true, PREFIX_VAL);
                },
                
                ((true, PREFIX_VAL|PREFIX_REG|PREFIX_KWD), ' '|'\t'|'\n') => {
                    if tmp_buf.is_empty() == false{
                        tokens.push(Token::make_from(inside_closure.1, &String::from_iter(tmp_buf.iter())));
                        tmp_buf = Vec::new();
                    }
                    inside_closure = (false, ' ');
                },
                
                ((false, _), MEM_START|MEM_CLOSE) => {
                    tmp_buf.push(MEM_START);
                    inside_closure = (true, c);
                },
                
                ((true, MEM_START), MEM_CLOSE) => {
                    tmp_buf.push(MEM_CLOSE);
                    tokens.push(Token::make_from(MEM_START, &String::from_iter(tmp_buf.iter())));
                    inside_closure = (false, ' ');
                    tmp_buf = Vec::new();
                }

                _ => tmp_buf.push(c),
            }
        }
        if tmp_buf.is_empty() == false {
            tokens.push(Token::make_from(inside_closure.1, &String::from_iter(tmp_buf.iter())));
        }
        return tokens;
    }
}

pub fn parse_num(numb: &str) -> Result<i64, ()> {
    let numb_bytes : &[u8] = numb.as_bytes();
    match numb_bytes.len() {
        0 => Err(()),
        1 => {
            if (numb_bytes[0] as char).is_numeric(){
                Ok((numb_bytes[0] - '0' as u8) as i64)
            }
            else {
                Err(())
            }
        },
        2 => numb.parse::<i64>().map_err(|_| ()),
        _ => {
            if numb.starts_with("0x"){
                let mut int : i64 = 0;
                let mut index : u32 = 0;
                for i in 2..numb.len(){
                    let hn = hexnum(numb_bytes[i] as char);
                    if hn != 16{
                        if hn == 0{
                            int *= 10;
                        }
                        else{
                            int += hexnum(numb_bytes[i] as char) as i64 * (16i64.pow(index));
                        }
                    }
                    else {
                        return Err(());
                    }
                    index += 1;
                }
                return Ok(int);
            }
            else if numb.starts_with("-0x"){
                let mut int : i64 = 0;
                let mut index : u32 = 0;
                for i in 3..numb.len(){
                    let hn = hexnum(numb_bytes[i] as char);
                    if hn != 16{
                        if hn == 0{
                            int *= 10;
                        }
                        else{
                            int += hexnum(numb_bytes[i] as char) as i64 * (16i64.pow(index));
                        }
                    }
                    else {
                        return Err(());
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
                        return Err(());
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
                        return Err(());
                    }
                    index += 1;
                }

                return Ok(-int);
            }
            else {
                return numb.parse::<i64>().map_err(|_| ());
            }
        }
    }
}
fn hexnum(n: char) -> u8{
    match n {
        '0'|'1'|'2'|'3'|'4'|
        '5'|'6'|'7'|'8'|'9'=> n as u8 - '0' as u8,
        'a'|'A' => 10,
        'b'|'B' => 11,
        'c'|'C' => 12,
        'd'|'D' => 13,
        'e'|'E' => 14,
        'f'|'F' => 15,
        _ => 16,
    }
}

impl Token{
    fn make_from(prefix: char, val: &str) -> Self{
        return match prefix {
            PREFIX_REG => {
                match Register::from_str(val){
                    Ok(reg) => Self::Register(reg),
                    Err(_)  => Self::UnknownReg(val.to_string()),
                }
            },
            PREFIX_VAL => {
                match parse_num(val){
                    Ok(val) => Self::Immediate(val),
                    Err(_ ) => Self::UnknownVal(val.to_string()),
                }
            },
            '.' => Self::Section(val.to_string()),
            MEM_START => {
                Self::MemAddr(val.to_string())
            },
            PREFIX_KWD => {
                if let Ok(kwd) = Keyword::from_str(val.trim()){
                    Self::Keyword(kwd)
                }
                else {
                    Self::UnknownKeyword(val.to_string())
                }
            }
            PREFIX_LAB => Self::LabelRef(val.to_string()),
            PREFIX_REF => Self::ConstRef(val.to_string()),
            _   => {
                if let Ok(ins) = Instruction::from_str(val){
                    Self::Instruction(ins)
                }
                else {
                    Self::Unknown(val.to_string())
                }
            }
        }
    }
}

impl ToString for Token{
    fn to_string(&self) -> String{
        match self{
            Self::Register(reg)         => format!("{}{}", PREFIX_REG, reg.to_string()),
            Self::MemAddr(mem)          => mem.to_string(),
            Self::Immediate(v)          => format!("{}{}", PREFIX_VAL, v.to_string()),
            Self::Keyword(kwd)          => kwd.to_string(),
            Self::Instruction(i)        => format!("{}", format!("{:?}", i).to_lowercase()),
            Self::Label(lbl)            => lbl.to_string(),
            Self::LabelRef(lbl)         => format!("{}{}", PREFIX_LAB, lbl),
            Self::String(str)           => format!("\"{}\"", str),
            Self::UnknownReg(str)       => format!("{}{}", PREFIX_REG, str.to_string()),
            Self::UnknownVal(str)       => format!("{}{}", PREFIX_VAL, str.to_string()),
            Self::Unknown(val)          => val.to_string(),
            Self::ConstRef(cref)        => format!("{}{}", PREFIX_REF, cref),
            Self::UnknownKeyword(kwd)   => format!("{}{}", PREFIX_KWD, kwd),
            Self::Comma                 => format!("{}", ','),
            Self::Section(sec)          => format!(".{}", sec)
        }
    }
}

impl ToString for Tokens{
    fn to_string(&self) -> String{
        let mut to_return = String::new();
        for (i, token) in self.0.iter().enumerate(){
            to_return.push_str(&token.to_string());
            if i + 1 < self.0.len(){
                to_return.push(' ');
            }
        }
        return to_return;
    }
}
