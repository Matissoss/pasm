// rasmx86_64 - mem.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use std::str::FromStr;
use crate::{
    pre::tok::parse_num,
    shr::{
        reg::Register,
        kwd::Keyword
    },
    conf::{
        PREFIX_REG,
        PREFIX_VAL,
        MEM_CLOSE,
        MEM_START,
        FAST_MODE,
    }
};

#[derive(Debug, PartialEq, Clone)]
pub enum Mem{
    MemAddr(Register, u8),
    MemAddrWOffset(Register, i64, u8),
    MemSIB(Register, Register, i64, u8)
}

#[derive(PartialEq, Debug, Clone)]
pub enum MemToken{
    Register(Register),
    UnknownReg(String),
    Number(i64),
    UnknownVal(String),
    Unknown(String),
    Plus,
    Comma,
    Start,
    End,
    Minus
}

impl MemToken{
    fn to_type(&self) -> String {
        match self {
            Self::Register  (_) => String::from("%register"),
            Self::Number    (_) => String::from("$number"),
            Self::Plus          => String::from("'+' (PLUS)"),
            Self::Minus         => String::from("'-' (MINUS)"),
            Self::Comma         => String::from("',' (COMMA)"),
            Self::UnknownReg(_) => String::from("%unknown_reg"),
            Self::UnknownVal(_) => String::from("$unknown_val"),
            Self::Unknown   (_) => String::from("?UNKNOWN"),
            Self::Start         => String::from(MEM_START),
            Self::End           => String::from(MEM_CLOSE),
        }
    }
}

type Opt<T> = Option<T>;

fn mem_par(tokens: Vec<MemToken>, size_spec: Option<Keyword>) -> Opt<Mem>{
    let mut tok_iter = tokens.iter();

    let mut offset      : Option<i64> = None;
    let mut base_reg    : Option<Register> = None;
    let mut index_reg   : Option<Register> = None;
    while let Some(tok) = tok_iter.next(){
        if let MemToken::Number(num) = tok{
            offset = Some(*num);
        }
        if let MemToken::Register(reg) = tok{
            if let None = base_reg{
                base_reg = Some(*reg);
            }
            else if let None = index_reg{
                index_reg = Some(*reg);
            }
            else {
                return None;
            }
        }
    }

    let size: u8 = if let Some(kwd) = size_spec{
        match kwd {
            Keyword::Qword => 8,
            Keyword::Dword => 4,
            Keyword::Word  => 2,
            Keyword::Byte  => 1,
            _              => return None
        }
    }
    else{
        return None;
    };

    return match (offset, base_reg, index_reg){
        (Some(off), Some(base), Some(index))    => Some(Mem::MemSIB(base, index, off, size)),
        (Some(off), Some(base), None)           => Some(Mem::MemAddrWOffset(base, off, size)),
        (None     , Some(base), None)           => Some(Mem::MemAddr(base, size)),
        (None     , Some(base), Some(index))    => Some(Mem::MemSIB(base, index, 0, size)),
        _ => None
    };
}


fn mem_tok(mem_addr: &str) -> Vec<MemToken>{
    let mem_raw : &[u8] = mem_addr.as_bytes();
    
    let mut tmp_buf : Vec<char>     = Vec::new();
    let mut tokens  : Vec<MemToken> = Vec::new();
    let mut minus   : bool          = false;

    let mut prefix  : char = ' ';
    for b in mem_raw{
        let c = *b as char;

        match (prefix, c) {
            (' ', '-') => minus = true,
            (' ', ' '|'+'|',') => continue,
            (PREFIX_VAL|PREFIX_REG, ' '|','|'+'|'-') => {
                tokens.push(mak_tok(prefix, String::from_iter(tmp_buf.iter()), &mut minus));
                tmp_buf = Vec::new();
                prefix = ' ';
            },
            (' ', PREFIX_REG) => prefix = PREFIX_REG,
            (' ', PREFIX_VAL) => prefix = PREFIX_VAL,
            (_, MEM_START) => {
                tokens.push(mak_tok(prefix, String::from_iter(tmp_buf.iter()), &mut minus));
                tmp_buf = Vec::new();
            },
            (_, MEM_CLOSE) => {
                tokens.push(mak_tok(prefix, String::from_iter(tmp_buf.iter()), &mut minus));
                tmp_buf = Vec::new();
            },
            _   => tmp_buf.push(c)
        }
    }
    return tokens;
}

fn mak_tok(prefix: char, string: String, minus: &mut bool) -> MemToken{
    match prefix{
        PREFIX_REG => {
            if let Ok(reg) = Register::from_str(&string){
                MemToken::Register(reg)
            }
            else {
                MemToken::UnknownReg(string)
            }
        },
        PREFIX_VAL => {
            if let Ok(num) = parse_num(&string){
                if *minus{
                    *minus = false;
                    MemToken::Number(-num)
                }
                else {
                    MemToken::Number(-num)
                }
            }
            else {
                MemToken::UnknownVal(string)
            }
        },
        _ => {
            if let Ok(num) = parse_num(&string.trim()){
                if *minus{
                    *minus = false;
                    MemToken::Number(-num)
                }
                else {
                    MemToken::Number(-num)
                }
            }
            else {
                MemToken::Unknown(string)
            }
        }
    }
}
