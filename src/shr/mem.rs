// rasmx86_64 - mem.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use std::str::FromStr;
use crate::{
    shr::{
        reg::Register,
        kwd::Keyword,
        num::Number,
        error::{
            RASMError,
            ExceptionType as ExType
        }
    },
    conf::{
        PREFIX_REG,
        PREFIX_VAL,
        MEM_CLOSE,
        MEM_START,
    }
};

#[derive(Debug, PartialEq, Clone)]
pub enum Mem{
    MemAddr(Register, u8),
    MemAddrWOffset(Register, i64, u8),
    MemSIB(Register, Register, i64, u8),
}

#[derive(PartialEq, Debug, Clone)]
pub enum MemToken{
    Register(Register),
    UnknownReg(String),
    Number(i64),
    InvalidVal(Number),
    UnknownVal(String),
    Unknown(String),
}

fn mem_par(tokens: Vec<MemToken>, size_spec: Option<Keyword>) -> Result<Mem, RASMError>{
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
                return Err(RASMError::new(
                    None,
                    ExType::Error,
                    Some(format!("{:?}", tokens)),
                    Some(format!("Too many registers found in memory declaration!")),
                    Some(format!("you can only have 2 (at max) registers in memory declaration"))
                ));
            }
        }
    }

    let size: u8 = if let Some(kwd) = size_spec{
        match kwd {
            Keyword::Qword => 8,
            Keyword::Dword => 4,
            Keyword::Word  => 2,
            Keyword::Byte  => 1,
            _              => return Err(RASMError::new(
                None,
                ExType::Error,
                Some(format!("{:?}", tokens)),
                Some(format!("Invalid size specifier found")),
                Some(format!("expected - byte, word, dword, qword"))
            ))
        }
    }
    else{
        return Err(RASMError::new(
            None,
            ExType::Error,
            Some(format!("{:?}", tokens)),
            Some(format!("No size specifier was found")),
            Some(format!("consider adding either one of these after memory declaration: byte, word, dword or qword keyword"))
        ));
    };

    return match (offset, base_reg, index_reg){
        (Some(off), Some(base), Some(index))        => Ok(Mem::MemSIB(base, index, off, size)),
        (Some(off), Some(base), None)               => Ok(Mem::MemAddrWOffset(base, off, size)),
        (None     , Some(base), None)               => Ok(Mem::MemAddr(base, size)),
        (None     , Some(base), Some(index))        => Ok(Mem::MemSIB(base, index, 0, size)),
        (None, None, _)|(Some(_), None      , _)    => {
            Err(RASMError::new(
                None,
                ExType::Error,
                Some(format!("{:?}", tokens)),
                Some(format!("Cannot index memory by absolute value")),
                Some(format!("Try adding register and number as offset.\n\t     (maybe you did forgot that registers are prefixed with '{}' and values with '{}'?)", PREFIX_REG, PREFIX_VAL)),
            ))
        },
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
            (' ', '+') => minus = false,
            (' ', ' '|',') => continue,
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
            if let Ok(num) = Number::from_str(&format!("{}{}", if *minus {"-"} else {""}, string)){
                if *minus {
                    *minus = false;
                }
                if let Some(i) = num.get_int(){
                    MemToken::Number(i)
                }
                else {
                    MemToken::InvalidVal(num)
                }
            }
            else {
                MemToken::UnknownVal(string)
            }
        },
        _ => {
            if let Ok(num) = Number::from_str(&format!("{}{}", if *minus {"-"} else {""}, string)){
                if *minus {
                    *minus = false;
                }

                if let Some(i) = num.get_int(){
                    MemToken::Number(i)
                }
                else {
                    MemToken::InvalidVal(num)
                }
            }
            else {
                MemToken::UnknownVal(string)
            }
        }
    }
}

impl Mem {
    pub fn new(memstr: &str, size_spec: Option<Keyword>) -> Result<Self, RASMError>{
        mem_par(mem_tok(memstr), size_spec)
    }
    pub fn size_bytes(&self) -> u8{
        match self{
            Self::MemSIB(_,_,_,size)|Self::MemAddrWOffset(_,_,size)|Self::MemAddr(_, size) => *size,
        }
    }
}
