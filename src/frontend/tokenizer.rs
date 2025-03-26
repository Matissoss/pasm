//  rasmx86_64 - tokenizer.rs
//  -------------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::{
    collections::HashMap,
    sync::LazyLock,
    str::FromStr
};

use crate::frontend::register::Register;

pub struct Tokenizer;

#[allow(unused)]
#[derive(Debug, PartialEq, Clone)]
pub enum Value{
    Register(Register),
    Char(char)        ,
    Int(i64)          ,
    Uint(u64)         ,
    Float(f64)        ,
    String(String)    ,
    ConstRef(String)  ,
    // for unitialized data in section .bss
    BSSRef  (String)  ,
    MemAddr   (Register),
    MemWOffset(Register, i64)
}

fn hextoint(c: char) -> u8{
    match c {
        '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => return c as u8 - '0' as u8,
        'A'|'a' => return 10,
        'B'|'b' => return 11,
        'C'|'c' => return 12,
        'D'|'d' => return 13,
        'E'|'e' => return 14,
        'F'|'f' => return 15,
        _       => return 0
    }
}

impl FromStr for Value {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, <Self as FromStr>::Err>{
        if value.len() == 0 {return Err(format!("E-VAL-0001: Passed empty value!"))}
        if value.len() >= 1 {
            if let Ok(parsed) = value.parse::<u64>() {
                return Ok(Self::Uint(parsed));
            }
            if let Ok(parsed) = value.parse::<i64>() {
                return Ok(Self::Int(parsed));
            }
            else if let Ok(register) = Register::from_str(value.trim()){
                return Ok(Self::Register(register));
            }
            else if let Ok(parsed) = value.parse::<f64>() {
                return Ok(Self::Float(parsed));
            }
            else if value.len() == 1 {
                let c = value.as_bytes()[0] as char;
                if c.is_numeric(){
                    return Ok(Self::Int((c as u8 - '0' as u8).into()))
                }
                return Ok(Self::Char(value.as_bytes()[0] as char));
            }
            else if value.starts_with("0b"){
                let mut number : u64 = 0;
                let mut index  : usize = 0;
                let bytes = value.as_bytes();
                for b in (2..bytes.len()).rev(){
                    if bytes[b] == '1' as u8{
                        number += 1 << index;
                    }
                    else if bytes[b] != '0' as u8{
                        return Err(format!("ERR-VAL-0002: Invalid binary number found: `{}`", value))
                    }
                    index += 1;
                }
                return Ok(Self::Uint(number));
            }
            else if value.starts_with("-0b"){
                let mut number : i64 = 0;
                let mut index  : usize = 0;
                let bytes = value.as_bytes();
                for b in (3..bytes.len()).rev(){
                    if bytes[b] == '1' as u8{
                        number += 1 << index;
                    }
                    else if bytes[b] != '0' as u8{
                        return Err(format!("ERR-VAL-0002: Invalid binary number found: `{}`", value))
                    }
                    index += 1;
                }
                return Ok(Self::Int(-number));
            }
            else if value.starts_with("0x"){ 
                let mut number : u64 = 0;
                let mut index  : u32 = 0;
                let bytes = value.as_bytes();
                for b in (2..bytes.len()).rev(){
                    number += (hextoint(bytes[b] as char) as u64) * (16u64.pow(index));
                    index += 1;
                }
                return Ok(Self::Uint(number));
            }
            else if value.starts_with("-0x"){ 
                let mut number : i64 = 0;
                let mut index  : u32 = 0;
                let bytes = value.as_bytes();
                for b in (3..bytes.len()).rev(){
                    number += (hextoint(bytes[b] as char) as i64) * (16i64.pow(index));
                    index += 1;
                }
                return Ok(Self::Int(-number));
            }
            else {
                let mut tmp_buf : Vec<char>         = Vec::new();
                let mut register: Option<Register>  = None; 
                for b in value.as_bytes(){
                    let c           : char              = *b as char;
                    match c {
                        '-'|'+' => {
                            register = Register::from_str(&String::from_iter(tmp_buf.iter())).ok();
                            tmp_buf  = Vec::new();
                            if c == '-'{
                                tmp_buf.push(c);
                            }
                        }
                        _ => tmp_buf.push(c)
                    }
                }
                let offset = String::from_iter(tmp_buf.iter()).parse::<i64>().ok();
                match (register, offset) {
                    (Some(reg), Some(ofs)) => {
                        return Ok(Self::MemWOffset(reg, ofs));
                    }
                    (Some(reg), None) => return Ok(Self::MemAddr(reg)),
                    (None, Some(val)) => {
                        return Err(format!("E-VAL-0002: Found `offset = {}`, but didn't found register in `{}`", val, value));
                    },
                    (None, None) => {
                        return Err(format!(
                        "E-VAL-0003: In beetwen '[' and ']' expected memory address (with or without offset), found: `{}`"
                        , value));
                    }
                }
            }
        }

        return Err(format!("E-VAL-0000: Unknown value: {}!", value));
    }
}


#[derive(Debug, Copy, Clone)]
pub enum KeyWord{
    Byte,
    Word,
    Dword,
    Qword,
    Global,
    Section,
    SysCall,
    EndLabel
}

pub static KEYWORDS : LazyLock<HashMap<&str, KeyWord>> = LazyLock::new(|| {
    let mut keywords : HashMap<&str, KeyWord> = HashMap::new();

    keywords.insert("dword"     , KeyWord::Dword);
    keywords.insert("qword"     , KeyWord::Qword);
    keywords.insert("word"      , KeyWord::Word);
    keywords.insert("byte"      , KeyWord::Byte);
    keywords.insert("syscall"   , KeyWord::SysCall);
    keywords.insert(".end"      , KeyWord::EndLabel);
    keywords.insert("global"    , KeyWord::Global);
    keywords.insert("section"   , KeyWord::Section);

    return keywords;
});

impl FromStr for KeyWord{
    type Err = ();
    fn from_str(keyword: &str) -> Result<Self, <Self as FromStr>::Err>{
        if let Some(kw) = KEYWORDS.get(keyword){
            return Ok(*kw);
        }
        else {return Err(())}
    }
}

#[allow(unused)]
#[derive(Debug)]
pub enum Token{
    KeyWord     (KeyWord),
    Value       (Value),
    Symbol      (char),
    String      (String),
    Label       (String),
}

impl FromStr for Token{
    type Err = String;
    fn from_str(token: &str) -> Result<Self, <Self as FromStr>::Err>{
        if token.len() == 1 {
            let c = token.as_bytes()[0] as char;
            if c.is_numeric(){
                return Ok(Token::Value(Value::Int((c as u8 - '0' as u8).into())));
            }
            return Ok(Token::Symbol(token.as_bytes()[0] as char));
        }
        if let Ok(keyword) = KeyWord::from_str(token) {
            return Ok(Token::KeyWord(keyword));
        }
        if let Ok(value) = Value::from_str(token){
            return Ok(Token::Value(value));
        }
        if token.ends_with(':'){
            return Ok(Token::Label(String::from_iter(
                token.chars().collect::<Vec<char>>()[..token.len()-1].iter()
            )));
        }
        return Ok(Token::String(token.to_string()));
    }
}

impl Tokenizer{
    pub fn tokenize_line(line: &str) -> Vec<Token>{
        let     bytes          :    &[u8]           = line.as_bytes();
        let mut inside_closure :    (bool, char)    = (false, ' ');
        let mut tokens         :    Vec<Token>      = Vec::new();
        let mut tmp_buf        :    Vec<char>       = Vec::new();
        for b in bytes{
            let c = *b as char;
            match (inside_closure, c){
                (_, ';') => break,
                ((false, _), ' '|'\t'|'\n') => {
                    if !tmp_buf.is_empty(){
                        if let Ok(token) = Token::from_str(&String::from_iter(tmp_buf.iter())){
                            tokens.push(token);
                        }
                        tmp_buf = Vec::new();
                    }
                },
                ((false, ' '), '$') => {
                    inside_closure = (true, '$');
                },
                ((false, ' '), '%') => {
                    inside_closure = (true, '%');
                },
                ((false, ' '), '"') => {
                    tmp_buf.push('"');
                    inside_closure = (true, '"');
                },
                ((true, '"'), '"') => {
                    tmp_buf.push('"');
                    tokens.push(Token::Value(Value::String(String::from_iter(tmp_buf.iter()))));
                    tmp_buf = Vec::new();
                    inside_closure = (false, ' ');
                }
                (_, ',') => continue,
                ((true, '$'), ' '|'\t'|'\n') => {
                    tokens.push(Token::Value(Value::ConstRef(String::from_iter(tmp_buf.iter()))));
                    tmp_buf = Vec::new();
                    inside_closure = (false, ' ');
                }
                ((true, '%'), ' '|'\t'|'\n') => {
                    tokens.push(Token::Value(Value::BSSRef(String::from_iter(tmp_buf.iter()))));
                    tmp_buf = Vec::new();
                    inside_closure = (false, ' ');
                }
                ((true, '"'), ' '|'\t'|'\n') => tmp_buf.push(c),
                ((true, _),   ' '|'\t'|'\n') => continue,
                ((false, ' '), '[') => {
                    inside_closure = (true, '[');
                }
                ((true, '['), ']') => {
                    match Value::from_str(&String::from_iter(tmp_buf.iter())){
                        Ok(value) => tokens.push(Token::Value(value)),
                        Err(error) => println!("{}", error)
                    }
                    tmp_buf = Vec::new();
                }
                _   => tmp_buf.push(c)
            }
        }
        if !tmp_buf.is_empty(){
            if inside_closure == (true, '$'){
                tokens.push(Token::Value(Value::ConstRef(String::from_iter(tmp_buf.iter()))));
            }
            else if inside_closure == (true, '%'){
                tokens.push(Token::Value(Value::BSSRef(String::from_iter(tmp_buf.iter()))));
            }
            else if let Ok(token) = Token::from_str(&String::from_iter(tmp_buf.iter())){
                tokens.push(token);
            }
        }
        return tokens;
    }
}
