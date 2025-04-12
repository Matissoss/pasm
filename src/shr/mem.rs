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
        },
        size::Size,
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
    MemAddr(Register, Size),
    MemAddrWOffset(Register, i32, Size),
    MemSIB(Register, Register, i32, Size, Size),
}

#[derive(PartialEq, Debug, Clone)]
pub enum MemToken{
    Register(Register),
    UnknownReg(String),
    Number(i32),
    InvalidVal(Number),
    UnknownVal(String),
    Unknown(String),
}

fn mem_par(tokens: Vec<MemToken>, size_spec: Option<Keyword>) -> Result<Mem, RASMError>{
    let mut tok_iter = tokens.iter();

    let mut offset      : Option<i32> = None;
    let mut scale       : Option<Size> = None;
    let mut base_reg    : Option<Register> = None;
    let mut index_reg   : Option<Register> = None;
    while let Some(tok) = tok_iter.next(){
        if let MemToken::Number(num) = tok{
            if let None = offset{
                offset = Some(*num);
            }
            else if let None = scale{
                if let Ok(s) = Size::try_from(*num as u8){
                    scale = Some(s);
                }
                else {
                    return Err(RASMError::new(
                        None,
                        ExType::Error,
                        Some(format!("{:?}", tokens)),
                        Some(format!("Couldn't parse number {} into size (1, 2, 4 or 8)", num)),
                        Some(format!("Consider changing that number into 1, 2, 4 or 8"))
                    ))
                }
            }
            else {
                return Err(RASMError::new(
                    None,
                    ExType::Error,
                    Some(format!("{:?}", tokens)),
                    Some(format!("Too many numbers found in memory declaration")),
                    Some(format!("max number of numbers in memory declaration is 2 (offset and scale)"))
                ))
            }
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

    let size: Size = if let Some(kwd) = size_spec{
        match kwd {
            Keyword::Qword => Size::Qword,
            Keyword::Dword => Size::Dword,
            Keyword::Word  => Size::Word,
            Keyword::Byte  => Size::Byte,
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
    return match (offset, base_reg, index_reg, scale){
        (Some(off)  , Some(base), Some(index)   , Some(scale))      => Ok(Mem::MemSIB(base, index, off, scale, size)),
        (Some(off)  , Some(base), None          , None)             => Ok(Mem::MemAddrWOffset(base, off, size)),
        (None       , Some(base), None          , None)             => Ok(Mem::MemAddr(base, size)),
        (None       , Some(base), Some(index)   , Some(scale))      => Ok(Mem::MemSIB(base, index, 0, scale, size)),
        (Some(_)    , None      , _             , _)                => {
            Err(RASMError::new(
                None,
                ExType::Error,
                Some(format!("{:?}", tokens)),
                Some(format!("Cannot index memory by absolute value")),
                Some(format!("Try adding register and number as offset.\n\t     (maybe you did forgot that registers are prefixed with '{}' and values with '{}'?)", PREFIX_REG, PREFIX_VAL)),
            ))
        },
        (_, Some(_), Some(_), None) => {
            Err(RASMError::new(
                None,
                ExType::Error,
                Some(format!("{:?}", tokens)),
                Some(format!("Expected scale in SIB memory declaration, found None")),
                Some(format!("Consider adding scale like: 1, 2, 4 or 8"))
            ))
        },
        (None, None, None, None) => {
            Err(RASMError::new(
                None,
                ExType::Error,
                Some(format!("{:?}", tokens)),
                Some(format!("Expected memory, found nothing")),
                Some(format!("Consider adding memory declaration like `{}{}rax{}`", PREFIX_REG, MEM_START, MEM_CLOSE))
            ))
        },
        (None, None, Some(i), Some(s)) => Ok(Mem::MemSIB(Register::RBP, i, 0, s, size)),
        _ => Err(RASMError::new(
            None,
            ExType::Error,
            Some(format!("{:?}", tokens)),
            Some(format!("Idk what happended here")),
            Some(format!("Idk what happended here")),
        ))
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
                    if let Ok(i) = i.try_into(){
                        MemToken::Number(i)
                    }
                    else {
                       MemToken::InvalidVal(num)
                    }
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
            if let Ok(num) = Number::from_str(&format!("{}{}", if *minus {"-"} else {""}, string).trim()){
                if *minus {
                    *minus = false;
                }

                if let Some(i) = num.get_int(){
                    if let Ok(i) = i.try_into(){
                        MemToken::Number(i)
                    }
                    else {
                       MemToken::InvalidVal(num)
                    }
                }
                else if let Some(i) = num.get_uint(){
                    if let Ok(i) = i.try_into(){
                        MemToken::Number(i)
                    }
                    else {
                        MemToken::InvalidVal(num)
                    }
                }
                else {
                    MemToken::InvalidVal(num)
                }
            }
            else {
                MemToken::Unknown(string)
            }
        }
    }
}

impl Mem {
    pub fn new(memstr: &str, size_spec: Option<Keyword>) -> Result<Self, RASMError>{
        mem_par(mem_tok(memstr), size_spec)
    }
    pub fn size(&self) -> Size{
        match self{
            Self::MemSIB(_,_,_,_,size)|Self::MemAddrWOffset(_,_,size)|Self::MemAddr(_, size) => *size,
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn mem_new_t(){
        let m = Mem::new("(%rax+20)", Some(Keyword::Qword));
        println!("{:?}", m);
        assert!(m == Ok(Mem::MemAddrWOffset(Register::RAX, 20, Size::Qword)))
    }
}
