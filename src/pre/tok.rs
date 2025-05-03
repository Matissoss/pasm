// rasmx86_64 - src/pre/tok.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::{
        COMMENT_S, MEM_CLOSE, MEM_START, PREFIX_KWD, PREFIX_REF, PREFIX_REG, PREFIX_SEG, PREFIX_VAL,
    },
    shr::{error::RASMError, ins::Mnemonic as Mnm, kwd::Keyword, num::Number, reg::Register},
};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Register(Register),
    Immediate(Number),
    Keyword(Keyword),
    Mnemonic(Mnm),
    MemAddr(String),
    Label(String),
    SymbolRef(String),
    String(String),
    Segment(String),
    UnknownSegment(String, RASMError),
    UnknownKeyword(String),
    UnknownReg(String),
    UnknownVal(String, RASMError),
    Unknown(String),
    Comma,
}

pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize_line(line: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut tmp_buf: Vec<char> = Vec::new();
        let mut inside_closure: Option<char> = None;

        for b in line.as_bytes() {
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

                (Some('"'), c) => tmp_buf.push(c),

                (None, ':') => {
                    tokens.push(Token::Label(String::from_iter(tmp_buf.iter())));
                    tmp_buf = Vec::new();
                }

                (Some(MEM_START), ',') => tmp_buf.push(c),

                (Some(PREFIX_REG | PREFIX_KWD | PREFIX_VAL), ',') => {
                    if !tmp_buf.is_empty() {
                        tokens.push(Token::make_from(
                            inside_closure,
                            String::from_iter(tmp_buf.iter()),
                        ));
                        tmp_buf = Vec::new();
                    }
                    tokens.push(Token::Comma)
                }
                (None, ',') => {
                    if !tmp_buf.is_empty() {
                        tokens.push(Token::make_from(
                            inside_closure,
                            String::from_iter(tmp_buf.iter()),
                        ));
                        tmp_buf = Vec::new();
                    }
                    tokens.push(Token::Comma)
                }

                (None, PREFIX_REF) => inside_closure = Some(PREFIX_REF),

                (Some(MEM_START), ' ') => continue,

                (None | Some(PREFIX_VAL | PREFIX_REG | PREFIX_KWD), ' ' | '\t' | '\n') => {
                    if !tmp_buf.is_empty() {
                        tokens.push(Token::make_from(
                            inside_closure,
                            String::from_iter(tmp_buf.iter()),
                        ));
                        tmp_buf = Vec::new();
                    }
                    inside_closure = None;
                }
                (None, PREFIX_REG | PREFIX_VAL | PREFIX_KWD | PREFIX_SEG) => {
                    inside_closure = Some(c)
                }

                (None, MEM_START) => {
                    tmp_buf.push(MEM_START);
                    inside_closure = Some(c);
                }

                (Some(MEM_START | PREFIX_SEG), MEM_CLOSE) => {
                    tmp_buf.push(MEM_CLOSE);
                    tokens.push(Token::make_from(
                        inside_closure,
                        String::from_iter(tmp_buf.iter()),
                    ));
                    inside_closure = None;
                    tmp_buf = Vec::new();
                }
                _ => tmp_buf.push(c),
            }
        }
        if !tmp_buf.is_empty() {
            tokens.push(Token::make_from(
                inside_closure,
                String::from_iter(tmp_buf.iter()),
            ));
        }
        tokens
    }
}

impl Token {
    fn make_from(prefix: Option<char>, val: String) -> Self {
        match prefix {
            Some(PREFIX_REG) => match Register::from_str(&val) {
                Ok(reg) => Self::Register(reg),
                Err(_) => Self::UnknownReg(val),
            },
            Some(PREFIX_VAL) => match Number::from_str(&val) {
                Ok(val) => Self::Immediate(val),
                Err(err) => Self::UnknownVal(val, err),
            },
            Some(MEM_START) => Self::MemAddr(val),
            Some(PREFIX_KWD) => {
                if let Ok(kwd) = Keyword::from_str(val.trim()) {
                    Self::Keyword(kwd)
                } else {
                    Self::UnknownKeyword(val)
                }
            }
            Some(PREFIX_SEG) => Self::Segment(val),
            Some(PREFIX_REF) => Self::SymbolRef(val),
            _ => {
                if let Ok(mnm) = Mnm::from_str(&val) {
                    Self::Mnemonic(mnm)
                } else {
                    Self::Unknown(val)
                }
            }
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Self::Register(reg) => format!("{}{}", PREFIX_REG, format!("{:?}", reg).to_lowercase()),
            Self::MemAddr(mem) => mem.to_string(),
            Self::Immediate(v) => format!("{}{}", PREFIX_VAL, v.to_string()),
            Self::Keyword(kwd) => kwd.to_string(),
            Self::Mnemonic(m) => format!("{:?}", m).to_lowercase(),
            Self::Label(lbl) => lbl.to_string(),
            Self::SymbolRef(lbl) => format!("{}{}", PREFIX_REF, lbl),
            Self::String(str) => str.to_string(),
            Self::UnknownReg(str) => format!("{}{}", PREFIX_REG, str),
            Self::UnknownVal(str, _) => format!("{}{}", PREFIX_VAL, str),
            Self::Unknown(val) => val.to_string(),
            Self::UnknownKeyword(kwd) => format!("{}{}", PREFIX_KWD, kwd),
            Self::Segment(s) => s.to_string(),
            Self::Comma => ','.to_string(),
            Self::UnknownSegment(s, _) => s.to_string(),
        }
    }
}
