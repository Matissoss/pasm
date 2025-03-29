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
        FAST_MODE,
    }
};

#[derive(Debug, PartialEq, Clone)]
pub enum MSScale{
    One   = 0b00,
    Two   = 0b01,
    Four  = 0b10,
    Eight = 0b11,
}

impl TryFrom<i8> for MSScale{
    type Error = ();
    fn try_from(num: i8) -> Result<Self, <Self as TryFrom<i8>>::Error> {
        return match num {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            4 => Ok(Self::Four),
            8 => Ok(Self::Eight),
            _ => Err(())
        };
    }
}
impl TryFrom<i64> for MSScale {
    type Error = ();
    fn try_from(num: i64) -> Result<Self, <Self as TryFrom<i64>>::Error>{
        let n : Result<i8, _> = num.try_into();
        if let Ok(n) = n{
            return Self::try_from(n);
        }
        else {
            return Err(());
        }
    }
}

// classical SIB
// SIB = Scale, Index, Base
#[derive(Debug, PartialEq, Clone)]
pub struct MemSIB{
    pub base : Register,
    pub index: Register,
    pub scale: MSScale,
    pub displacement: Option<i64>
}
#[derive(Debug, PartialEq, Clone)]
pub enum Mem{
    MemAddr(Register),
    MemAddrWOffset(Register, i64),
    MemSIB(MemSIB),
    Unknown
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
            Self::Unknown   (_) => String::from("?UNKNOWN")
        }
    }
}

#[allow(unused)]
#[derive(PartialEq)]
pub enum MemErr{
    UnexpectedToken(MemToken),
    InvalidReg(String),
    InvalidVal(String),
    UnknownVal(String),
    TypeErr(String),
    Other(String)
}

type Res<T, E> = Result<T, E>;
fn type_err(expected: MemToken, found: MemToken) -> MemErr{
    match &found{
        MemToken::UnknownReg(r)|MemToken::UnknownVal(r)|MemToken::Unknown(r) => {
            return MemErr::TypeErr(format!("Expected `{}`, found `{}`: `{}`", expected.to_type(), found.to_type(), r));
        },
        MemToken::Register(r) => {
            return MemErr::TypeErr(format!("Expected `{}`, found `{}`: `{:?}`", expected.to_type(), found.to_type(), r));
        }
        MemToken::Number(n) => {
            return MemErr::TypeErr(format!("Expected `{}`, found `{}`: `{}`", expected.to_type(), found.to_type(), n));
        }
        _ => {}
    }
    return MemErr::TypeErr(format!("Expected `{}`, found `{}`", expected.to_type(), found.to_type()));
}

fn mem_par(tok: Vec<MemToken>) -> Res<Mem, MemErr>{
    match tok.len() {
        1 => {
            if let MemToken::Register(reg) = tok[0] {
                return Ok(Mem::MemAddr(reg));
            }
        },
        3 => {
            if let MemToken::Register(reg) = &tok[0] {
                if let MemToken::Number(n) = &tok[2] {
                    if let MemToken::Minus = &tok[1] {
                        return Ok(Mem::MemAddrWOffset(*reg, -n));
                    }
                    if let MemToken::Plus  = &tok[1] {
                        return Ok(Mem::MemAddrWOffset(*reg, *n));
                    }
                    if let MemToken::Comma = &tok[1] {
                        return Err(MemErr::UnexpectedToken(tok[1].clone()));
                    }
                }
            }
        },
        5 => {
            if let MemToken::Register(reg)     = tok[0]{
                if let MemToken::Register(r)   = tok[2]{
                    if let MemToken::Number(n) = tok[4]{
                        if let Ok(msscale) = MSScale::try_from(n){
                            if FAST_MODE {
                                return Ok(Mem::MemSIB(MemSIB{base: reg, index: r, scale: msscale, displacement: None}));
                            }
                            else {
                                if let (MemToken::Comma, MemToken::Comma) = (&tok[1], &tok[3]){
                                    return Ok(Mem::MemSIB(MemSIB{base: reg, index: r, scale: msscale, displacement: None}));
                                }
                            }
                        }
                    }
                    else if let MemToken::Unknown(kwd) = tok[4].clone(){
                        if let Ok(kwd) = Keyword::from_str(&kwd){
                            let msscale = match kwd {
                                Keyword::Qword => MSScale::Eight,
                                Keyword::Dword => MSScale::Four,
                                Keyword::Word  => MSScale::Two,
                                Keyword::Byte  => MSScale::One,
                                _ => return Err(MemErr::Other(
                                        format!("Expected either: [Qword, Dword, Word, Byte] found: {:?}", kwd)))
                            };
                            if FAST_MODE {
                                return Ok(Mem::MemSIB(MemSIB{base: reg, index: r, scale: msscale, displacement: None}));
                            }
                            else {
                                if let (MemToken::Comma, MemToken::Comma) = (&tok[1], &tok[3]){
                                    return Ok(Mem::MemSIB(MemSIB{base: reg, index: r, scale: msscale, displacement: None}));
                                }
                            }
                        }
                    }
                }
            }
            if FAST_MODE {
                return Ok(Mem::Unknown);
            }
            else {
                match &tok[0] {
                    MemToken::Register(_) => {}
                    _ => {
                        return Err(type_err(MemToken::Register(Register::AL), tok[0].clone()));
                    }
                }
                match &tok[1] {
                    MemToken::Comma => {},
                    _ => {
                        return Err(type_err(MemToken::Comma, tok[0].clone()));
                    }
                }
                match &tok[2] {
                    MemToken::Register(_) => {}
                    _ => {
                        return Err(type_err(MemToken::Register(Register::AL), tok[0].clone()));
                    }
                }
                match &tok[3] {
                    MemToken::Comma => {},
                    _ => {
                        return Err(type_err(MemToken::Comma, tok[0].clone()));
                    }
                }
                match &tok[4] {
                    MemToken::Number(n) => {
                        if n != &1 && n != &2 && n != &4 && n != &8{
                            return Err(MemErr::Other(format!("Expected either one of these numbers: [1, 2, 4, 8], found: {}", n)));
                        }
                    }
                    _ => {
                        return Err(type_err(MemToken::Number(0), tok[0].clone()));
                    }
                }
            }
        },
        7 => {
            if let MemToken::Register(reg)   = tok[0]{
                if let MemToken::Register(r) = tok[2]{
                    if let MemToken::Number(n) = tok[4]{
                        if let Ok(msscale) = MSScale::try_from(n){
                            if let MemToken::Number(d) = tok[6] {
                                if FAST_MODE {
                                    return Ok(Mem::MemSIB(
                                        MemSIB{base: reg, index: r, scale: msscale, displacement: Some(d)}));
                                }
                                else {
                                    if let (MemToken::Comma, MemToken::Comma) = (&tok[1], &tok[3]){
                                        return Ok(Mem::MemSIB(
                                            MemSIB{base: reg, index: r, scale: msscale, displacement: Some(d)}));
                                    }
                                }
                            }
                        }
                    }
                    else if let MemToken::Unknown(kwd) = tok[4].clone(){
                        if let Ok(kwd) = Keyword::from_str(&kwd){
                            let msscale = match kwd {
                                Keyword::Qword => MSScale::Eight,
                                Keyword::Dword => MSScale::Four,
                                Keyword::Word  => MSScale::Two,
                                Keyword::Byte  => MSScale::One,
                                _ => return Err(MemErr::Other(
                                        format!("Expected either: [Qword, Dword, Word, Byte] found: {:?}", kwd)))
                            };
                            if let MemToken::Number(d) = tok[6] {
                                if FAST_MODE {
                                    return Ok(Mem::MemSIB(
                                        MemSIB{base: reg, index: r, scale: msscale, displacement: Some(d)}));
                                }
                                else {
                                    if let (MemToken::Comma, MemToken::Comma) = (&tok[1], &tok[3]){
                                        return Ok(Mem::MemSIB(
                                            MemSIB{base: reg, index: r, scale: msscale, displacement: Some(d)}));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if FAST_MODE {
                return Ok(Mem::Unknown);
            }
            else {
                match &tok[0] {
                    MemToken::Register(_) => {}
                    _ => {
                        return Err(type_err(MemToken::Register(Register::AL), tok[0].clone()));
                    }
                }
                match &tok[1] {
                    MemToken::Comma => {},
                    _ => {
                        return Err(type_err(MemToken::Comma, tok[0].clone()));
                    }
                }
                match &tok[2] {
                    MemToken::Register(_) => {}
                    _ => {
                        return Err(type_err(MemToken::Register(Register::AL), tok[0].clone()));
                    }
                }
                match &tok[3] {
                    MemToken::Comma => {},
                    _ => {
                        return Err(type_err(MemToken::Comma, tok[0].clone()));
                    }
                }
                match &tok[4] {
                    MemToken::Number(n) => {
                        if n != &1 && n != &2 && n != &4 && n != &8{
                            return Err(MemErr::Other(format!("Expected either one of these numbers: [1, 2, 4, 8], found: {}", n)));
                        }
                    }
                    _ => {
                        return Err(type_err(MemToken::Number(0), tok[0].clone()));
                    }
                }
                match &tok[5] {
                    MemToken::Comma => {}
                    _ => {
                        return Err(type_err(MemToken::Comma, tok[0].clone()));
                    }
                }
                match &tok[6] {
                    MemToken::Number(_) => {}
                    _ => {
                        return Err(type_err(MemToken::Number(0), tok[0].clone()));
                    }
                }
            }
        }
        _ => {}
    }

    return Ok(Mem::Unknown);
}

fn mem_tok(str: &str) -> Vec<MemToken>{
    let mut splitted = Vec::new();
    let mut tmp_buf  = Vec::new();
    let mut reg      = false;
    let mut val      = false;
    for b in str.as_bytes(){
        let tmp_c = *b as char;
        match tmp_c {
            PREFIX_REG => {
                reg = true;
                val = false;
            },
            PREFIX_VAL => {
                val = true;
                reg = false;
            }
            _ => {
                if tmp_c != '+' && tmp_c != '-' && tmp_c != ',' {
                    tmp_buf.push(tmp_c);
                    continue;
                }
                if tmp_buf.is_empty() == false {
                    if reg {
                        let reg_str = String::from_iter(tmp_buf.iter());
                        if let Ok(reg) = Register::from_str(&reg_str){
                            splitted.push(MemToken::Register(reg));
                        }
                        else {
                            splitted.push(MemToken::UnknownReg(reg_str));
                        }
                        reg = false;
                    }
                    else if val {
                        let val_str = String::from_iter(tmp_buf.iter());
                        if let Ok(num) = parse_num(&val_str){
                            splitted.push(MemToken::Number(num));
                        }
                        else {
                            splitted.push(MemToken::UnknownVal(val_str));
                        }
                        val = false;
                    }
                    else {
                        let val = String::from_iter(tmp_buf.iter());
                        splitted.push(MemToken::Unknown(val));
                    }
                    tmp_buf = Vec::new();
                }
                if tmp_c == '+' {
                    splitted.push(MemToken::Plus);
                }
                else if tmp_c == '-' {
                    splitted.push(MemToken::Minus);
                }
                else if tmp_c == ',' {
                    splitted.push(MemToken::Comma);
                }
            },
        }
    }
    if tmp_buf.is_empty() == false {
        if reg {
            let reg_str = String::from_iter(tmp_buf.iter());
            if let Ok(reg) = Register::from_str(&reg_str){
                splitted.push(MemToken::Register(reg));
            }
            else {
                splitted.push(MemToken::UnknownReg(reg_str));
            }
        }
        else if val {
            let val_str = String::from_iter(tmp_buf.iter());
            if let Ok(num) = parse_num(&val_str){
                splitted.push(MemToken::Number(num));
            }
            else {
                splitted.push(MemToken::UnknownVal(val_str));
            }
        }
        else {
            splitted.push(MemToken::Unknown(String::from_iter(tmp_buf.iter())));
        }
    }
    return splitted;
}

impl FromStr for Mem{
    type Err = MemErr;
    fn from_str(raw_mem: &str) -> Result<Self, <Self as FromStr>::Err>{
        match mem_par(mem_tok(raw_mem)){
            Ok(mem)  => return Ok(mem),
            Err(err) => return Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mem_tok_t() {
        let mem_str = "%rax-$10,%rcx,$8";
        let mem_tokenized = mem_tok(mem_str);
        println!("{:?}", mem_tokenized);
        assert!(mem_tokenized == vec![
            MemToken::Register(Register::RAX), MemToken::Minus, MemToken::Number(10), MemToken::Comma, 
            MemToken::Register(Register::RCX), MemToken::Comma, MemToken::Number(8)
        ]);
    }
    #[test]
    fn mem_par_t(){
        let mem_tokenized = vec![
            MemToken::Register(Register::RAX), MemToken::Comma, MemToken::Register(Register::RCX), MemToken::Comma, 
            MemToken::Number(8), MemToken::Comma, MemToken::Number(10)
        ];
        assert!(mem_par(mem_tokenized) == 
            Ok(Mem::MemSIB(MemSIB{base: Register::RAX, index: Register::RCX, scale: MSScale::Eight, displacement: Some(10)})));
    }
}
