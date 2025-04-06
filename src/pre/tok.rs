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
        num::{
            Number,
            FromStrNumberErr as NumberErr
        },
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
    Immediate(Number),
    Keyword(Keyword),
    Instruction(Instruction),
    Section(String),
    MemAddr(String),
    Label(String),
    LabelRef(String),
    String(String),
    ConstRef(String),
    UnknownKeyword(String),
    UnknownReg(String),
    UnknownVal(String, NumberErr),
    Unknown(String),
    Comma,
}

pub struct Tokenizer;
pub struct Tokens(pub Vec<Token>);

impl Tokenizer{
    pub fn tokenize_line(line: &str) -> Vec<Token>{
        let mut tokens          : Vec<Token>    = Vec::new();
        let mut tmp_buf         : Vec<char>     = Vec::new();
        let mut inside_closure  : Option<char>  = None;
        
        for b in line.as_bytes(){
            let c = *b as char;
            match (inside_closure, c) {
                (_, COMMENT_S) => break,

                (None, '"') => {
                    inside_closure = Some('"');
                }
                (Some('"'), '"') => {
                    tokens.push(Token::String(String::from_iter(tmp_buf.iter())));
                    tmp_buf = Vec::new();
                    inside_closure = None;
                }

                (Some('"'), c  ) => tmp_buf.push(c),

                (None, ':') => {
                    tokens.push(Token::Label(String::from_iter(tmp_buf.iter())));
                    tmp_buf = Vec::new();
                },

                (Some(MEM_START), ',') => tmp_buf.push(c),
                
                (Some(PREFIX_REG|PREFIX_KWD|PREFIX_VAL), ',') => {
                    if tmp_buf.is_empty() == false {
                        tokens.push(Token::make_from(inside_closure, &String::from_iter(tmp_buf.iter())));
                        tmp_buf = Vec::new();
                    }
                    tokens.push(Token::Comma)
                },
                (None, ',') => {
                    if tmp_buf.is_empty() == false {
                        tokens.push(Token::make_from(inside_closure, &String::from_iter(tmp_buf.iter())));
                        tmp_buf = Vec::new();
                    }
                    tokens.push(Token::Comma)
                },
                
                (Some(MEM_START), ' ') => continue,
                
                (None|Some(PREFIX_VAL|PREFIX_REG|PREFIX_KWD), ' '|'\t'|'\n') => {
                    if tmp_buf.is_empty() == false {
                        tokens.push(Token::make_from(inside_closure, &String::from_iter(tmp_buf.iter())));
                        tmp_buf = Vec::new();
                    }
                    inside_closure = None;
                },
                (None, PREFIX_REG|PREFIX_VAL|PREFIX_KWD|PREFIX_LAB|'.') => inside_closure = Some(c),
                
                
                (None, MEM_START|MEM_CLOSE) => {
                    tmp_buf.push(MEM_START);
                    inside_closure = Some(c);
                },
                
                (Some(MEM_START), MEM_CLOSE) => {
                    tmp_buf.push(MEM_CLOSE);
                    tokens.push(Token::make_from(Some(MEM_START), &String::from_iter(tmp_buf.iter())));
                    inside_closure = None;
                    tmp_buf = Vec::new();
                }

                _ => tmp_buf.push(c),
            }
        }
        if tmp_buf.is_empty() == false {
            tokens.push(Token::make_from(inside_closure, &String::from_iter(tmp_buf.iter())));
        }
        return tokens;
    }
}

impl Token{
    fn make_from(prefix: Option<char>, val: &str) -> Self{
        return match prefix {
            Some(PREFIX_REG) => {
                match Register::from_str(val){
                    Ok(reg) => Self::Register(reg),
                    Err(_)  => Self::UnknownReg(val.to_string()),
                }
            },
            Some(PREFIX_VAL) => {
                match Number::from_str(val){
                    Ok(val) => Self::Immediate(val),
                    Err(err) => Self::UnknownVal(val.to_string(), err),
                }
            },
            Some('.') => Self::Section(val.to_string()),
            Some(MEM_START) => {
                Self::MemAddr(val.to_string())
            },
            Some(PREFIX_KWD) => {
                if let Ok(kwd) = Keyword::from_str(val.trim()){
                    Self::Keyword(kwd)
                }
                else {
                    Self::UnknownKeyword(val.to_string())
                }
            }
            Some(PREFIX_LAB) => Self::LabelRef(val.to_string()),
            Some(PREFIX_REF) => Self::ConstRef(val.to_string()),
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
            Self::UnknownVal(str, _)    => format!("{}{}", PREFIX_VAL, str.to_string()),
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
