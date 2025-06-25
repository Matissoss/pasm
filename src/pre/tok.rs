// pasm - src/pre/tok.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::*,
    shr::{
        error::RASMError, ins::Mnemonic, kwd::Keyword, math, num::Number, reg::Register,
        smallvec::SmallVec, symbol::SymbolRef,
    },
};
use std::str::FromStr;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Token {
    Register(Register),
    Immediate(Number),
    Keyword(Keyword),
    Mnemonic(Mnemonic),
    Label(RString),
    SymbolRef(Box<SymbolRef>),
    String(RString),
    Comma,

    Unknown(RString),
    Error(Box<RASMError>),
    //       pfx   content
    Closure(char, RString),

    Modifier(SharedArr<Self>),

    #[default]
    None,
}

pub fn tokl(line: &str) -> SmallVec<Token, SMALLVEC_TOKENS_LEN> {
    let mut tokens: SmallVec<Token, SMALLVEC_TOKENS_LEN> = SmallVec::new();

    let mut tmp_buf: Vec<char> = Vec::with_capacity(24);
    let mut inside_closure: Option<char> = None;
    let mut closure_pfx: Option<char> = None;
    let mut delimeter_count: usize = 0;
    let mut tmp_toks: SmallVec<Token, 4> = SmallVec::new();

    for b in line.as_bytes() {
        let c = *b as char;
        match (inside_closure, c) {
            (None, COMMENT_S) => break,

            (None, '"') => {
                inside_closure = Some('"');
            }
            (Some('"'), '"') => {
                tokens.push(Token::String(String::from_iter(tmp_buf.iter()).into()));
                tmp_buf.clear();
                inside_closure = None;
            }

            (Some('"'), c) => tmp_buf.push(c),

            (None, ':') => {
                tokens.push(Token::Label(String::from_iter(tmp_buf.iter()).into()));
                tmp_buf.clear();
            }
            (Some(PREFIX_REG | PREFIX_KWD | PREFIX_VAL), ':') => {
                delimeter_count = 0;
                if !tmp_buf.is_empty() {
                    tmp_toks.push(Token::make_from(
                        inside_closure,
                        String::from_iter(tmp_buf.iter()),
                    ));
                }
                closure_pfx = None;
                inside_closure = Some(':');
                tmp_buf.clear();
            }

            (Some(':'), ':') => {
                if delimeter_count == 0 {
                    if !tmp_buf.is_empty() {
                        tmp_toks.push(Token::make_from(
                            closure_pfx,
                            String::from_iter(tmp_buf.iter()),
                        ));
                        tmp_buf.clear();
                    }
                    closure_pfx = None;
                } else {
                    tmp_buf.push(':');
                }
            }
            (Some(':'), PREFIX_KWD | PREFIX_REG | PREFIX_VAL) => {
                if delimeter_count == 0 {
                    closure_pfx = Some(c);
                } else {
                    tmp_buf.push(c);
                }
            }
            (Some(':'), CLOSURE_START) => {
                if delimeter_count != 0 {
                    tmp_buf.push(CLOSURE_START);
                }
                delimeter_count += 1;
            }
            (Some(':'), CLOSURE_END) => {
                if delimeter_count == 1 {
                    tmp_toks.push(Token::make_closure(
                        closure_pfx.unwrap_or(' '),
                        String::from_iter(tmp_buf.iter()).into(),
                    ));
                    tmp_buf.clear();
                    closure_pfx = None;
                    delimeter_count = 0;
                } else {
                    delimeter_count -= 1;
                }
            }
            (Some(':'), ' ') => {
                if delimeter_count == 0 {
                    if !tmp_buf.is_empty() {
                        tmp_toks.push(Token::make_from(
                            closure_pfx,
                            String::from_iter(tmp_buf.iter()),
                        ));
                    }
                    tmp_buf.clear();
                    tokens.push(Token::make_modifier(tmp_toks.into_iter()));
                    tmp_toks = SmallVec::new();
                    inside_closure = None;
                    closure_pfx = None;
                }
            }

            (Some(CLOSURE_START), ',') => tmp_buf.push(c),

            (Some(PREFIX_REG | PREFIX_KWD | PREFIX_VAL), ',') => {
                if !tmp_buf.is_empty() {
                    tokens.push(Token::make_from(
                        inside_closure,
                        String::from_iter(tmp_buf.iter()),
                    ));
                    tmp_buf.clear();
                }
                tokens.push(Token::Comma)
            }
            (None, ',') => {
                if !tmp_buf.is_empty() {
                    tokens.push(Token::make_from(
                        inside_closure,
                        String::from_iter(tmp_buf.iter()),
                    ));
                    tmp_buf.clear();
                }
                tokens.push(Token::Comma)
            }

            (Some(CLOSURE_START), ' ') => continue,

            (None | Some(PREFIX_VAL | PREFIX_REG | PREFIX_KWD), ' ' | '\t' | '\n') => {
                if !tmp_buf.is_empty() {
                    tokens.push(Token::make_from(
                        inside_closure,
                        String::from_iter(tmp_buf.iter()),
                    ));
                    tmp_buf.clear();
                }
                inside_closure = None;
            }
            (None, PREFIX_REG | PREFIX_VAL | PREFIX_REF | PREFIX_KWD | PREFIX_SEG) => {
                inside_closure = Some(c)
            }

            (Some(CLOSURE_START), CLOSURE_START) => {
                delimeter_count += 1;
                tmp_buf.push(CLOSURE_START)
            }

            (
                Some(PREFIX_REG | PREFIX_VAL | PREFIX_REF | PREFIX_KWD | PREFIX_SEG | ' ') | None,
                CLOSURE_START,
            ) => {
                if delimeter_count != 0 {
                    tmp_buf.push(CLOSURE_START);
                } else {
                    closure_pfx = inside_closure;
                    inside_closure = Some(CLOSURE_START);
                }
                delimeter_count += 1;
            }

            (Some(CLOSURE_START | PREFIX_SEG | PREFIX_REF), CLOSURE_END) => {
                if delimeter_count == 1 {
                    tokens.push(Token::make_closure(
                        closure_pfx.unwrap_or(' '),
                        String::from_iter(tmp_buf.iter()).into(),
                    ));
                    closure_pfx = None;
                    inside_closure = None;
                    tmp_buf.clear();
                    delimeter_count = 0;
                } else {
                    delimeter_count -= 1;
                    tmp_buf.push(CLOSURE_END);
                }
            }
            _ => tmp_buf.push(c),
        }
    }
    if !tmp_buf.is_empty() {
        if !tmp_toks.is_empty() {
            tmp_toks.push(Token::make_from(
                closure_pfx,
                String::from_iter(tmp_buf.iter()),
            ));
            tokens.push(Token::make_modifier(tmp_toks.into_iter()))
        } else {
            tokens.push(Token::make_from(
                inside_closure,
                String::from_iter(tmp_buf.iter()),
            ));
        }
    } else if !tmp_toks.is_empty() {
        tokens.push(Token::make_modifier(tmp_toks.into_iter()));
    }
    tokens
}

impl Token {
    fn make_modifier(toks: Vec<Self>) -> Self {
        Token::Modifier(toks.into())
    }
    fn make_closure(prefix: char, val: RString) -> Self {
        use math::MathematicalEvaluation as MathEval;
        match prefix {
            PREFIX_VAL => match MathEval::from_str(&val) {
                Ok(m) => Token::Immediate(Number::uint64(MathEval::eval(m).unwrap_or(0))),
                Err(e) => Token::Error(Box::new(e)),
            },
            PREFIX_REF => match SymbolRef::try_new(&val) {
                Ok(s) => Token::SymbolRef(Box::new(s)),
                Err(e) => Token::Error(Box::new(e)),
            },
            _ => Self::Closure(prefix, val),
        }
    }
    fn make_from(prefix: Option<char>, val: String) -> Self {
        match prefix {
            Some(PREFIX_REG) => match Register::from_str(&val) {
                Ok(reg) => Self::Register(reg),
                Err(_) => Self::Unknown(val.into()),
            },
            Some(PREFIX_VAL) => match Number::from_str(&val) {
                Ok(val) => Self::Immediate(val),
                Err(err) => Self::Error(Box::new(err)),
            },
            Some(CLOSURE_START) => Self::Closure(' ', val.into()),
            Some(PREFIX_KWD) => {
                if let Ok(kwd) = Keyword::from_str(val.trim()) {
                    Self::Keyword(kwd)
                } else {
                    Self::Unknown(val.into())
                }
            }
            Some(PREFIX_REF) => match SymbolRef::try_new(&val) {
                Ok(s) => Self::SymbolRef(Box::new(s)),
                Err(e) => Self::Error(Box::new(e)),
            },
            _ => {
                #[cfg(not(feature = "refresh"))]
                if let Ok(mnm) = Mnemonic::from_str(&val) {
                    Self::Mnemonic(mnm)
                } else {
                    Self::Unknown(val.into())
                }
                #[cfg(feature = "refresh")]
                Self::Unknown(val.into())
            }
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Self::Register(reg) => format!("{}{}", PREFIX_REG, reg.to_string()),
            Self::Immediate(v) => format!("{}{}", PREFIX_VAL, v.to_string()),
            Self::Keyword(kwd) => kwd.to_string(),
            #[cfg(feature = "iinfo")]
            Self::Mnemonic(m) => m.to_string(),
            #[cfg(not(feature = "iinfo"))]
            Self::Mnemonic(_) => "".to_string(),
            Self::Label(lbl) => lbl.to_string(),
            Self::SymbolRef(lbl) => format!("{}{}", PREFIX_REF, lbl.to_string()),
            Self::String(str) => str.to_string(),
            Self::Error(_) => "".to_string(),
            Self::Unknown(val) => val.to_string(),
            Self::Comma => ','.to_string(),
            Self::Closure(pfx, ctt) => format!("{pfx}({ctt})"),
            Self::Modifier(content) => {
                let mut string = String::new();
                for c in content.iter() {
                    string.push_str(&format!("{}:", c.to_string()));
                }
                string
            }
            Self::None => String::new(),
        }
    }
}
