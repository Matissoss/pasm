// pasm - src/pre/tok.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::*,
    shr::{
        ast::Operand,
        error::RError,
        error::RError as Error,
        ins::Mnemonic,
        kwd::Keyword,
        math,
        num::Number,
        reg::{Purpose as RPurpose, Register},
        reloc::RelType,
        smallvec::SmallVec,
        symbol::SymbolRef,
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
    Error(Box<RError>),
    //       pfx   content
    Closure(char, RString),

    Modifier(SharedArr<Self>),

    #[default]
    None,
}

pub fn tokl(tmp_buf: &mut Vec<char>, line: &str) -> SmallVec<Token, SMALLVEC_TOKENS_LEN> {
    let mut tokens: SmallVec<Token, SMALLVEC_TOKENS_LEN> = SmallVec::new();

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
            (Some(PREFIX_REG | PREFIX_KWD | PREFIX_REF | PREFIX_VAL), ':') => {
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
    if delimeter_count != 0 {
        let er = RError::new(
            format!("unclosed delimeter `(` (`{delimeter_count}` unclosed delimeters)"),
            000,
        );
        tokens.push(Token::Error(Box::new(er)));
    }
    tokens
}

impl Token {
    fn make_modifier(toks: Vec<Self>) -> Self {
        match toks.len() {
            2 => {
                let name = &toks[0];
                let relof = &toks[1];
                let sname = if let Token::String(s) | Token::Unknown(s) = name {
                    s
                } else {
                    return Token::Modifier(toks.into());
                };
                let reltype = if let Ok(rtype) = RelType::try_from(relof) {
                    rtype
                } else {
                    return Token::Modifier(toks.into());
                };
                return Token::SymbolRef(Box::new(SymbolRef::new(
                    sname.clone(),
                    None,
                    false,
                    None,
                    Some(reltype),
                )));
            }
            3 => {
                let name = &toks[0];
                let relt = &toks[1];
                let mut offs = &toks[2];
                let sname = if let Token::String(s) | Token::Unknown(s) = name {
                    s.clone()
                } else if let Token::SymbolRef(s) = name {
                    s.symbol.clone()
                } else {
                    return Token::Modifier(toks.into());
                };
                let mut reltype = RelType::REL32;
                if let Ok(rtype) = RelType::try_from(relt) {
                    reltype = rtype;
                } else {
                    offs = relt;
                }
                let offset;
                if let Token::Immediate(n) = offs {
                    offset = n.get_as_i32();
                } else {
                    if let Token::Closure(p, v) = offs {
                        if let Token::Immediate(n) = Self::make_closure(*p, v.clone()) {
                            offset = n.get_as_i32();
                        } else {
                            return Token::Modifier(toks.into());
                        }
                    } else {
                        return Token::Modifier(toks.into());
                    }
                }
                return Token::SymbolRef(Box::new(SymbolRef::new(
                    sname.clone(),
                    if offset == 0 { None } else { Some(offset) },
                    false,
                    None,
                    Some(reltype),
                )));
            }
            _ => {}
        }
        Token::Modifier(toks.into())
    }
    fn make_closure(prefix: char, val: RString) -> Self {
        use math::MathematicalEvaluation as MathEval;
        match prefix {
            PREFIX_VAL => match MathEval::from_str(&val) {
                Ok(m) => Token::Immediate(Number::uint64(MathEval::eval(m).unwrap_or(0))),
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
            _ =>
            {
                #[cfg(not(feature = "refresh"))]
                if let Ok(mnm) = Mnemonic::from_str(&val) {
                    Self::Mnemonic(mnm)
                } else if let Ok(reg) = Register::from_str(&val) {
                    Self::Register(reg)
                } else if let Ok(num) = Number::from_str(&val) {
                    Self::Immediate(num)
                } else if let Ok(dir) = Keyword::from_str(&val) {
                    Self::Keyword(dir)
                } else {
                    Self::Unknown(val.into())
                }
            }
        }
    }
}

impl TryFrom<&Token> for RelType {
    type Error = ();
    fn try_from(tok: &Token) -> Result<Self, <Self as TryFrom<&Token>>::Error> {
        match tok {
            Token::Keyword(Keyword::Rel32) => Ok(Self::REL32),
            Token::Keyword(Keyword::Rel16) => Ok(Self::REL16),
            Token::Keyword(Keyword::Rel8) => Ok(Self::REL8),
            Token::Keyword(Keyword::Abs32) => Ok(Self::ABS32),
            _ => Err(()),
        }
    }
}

impl TryFrom<Token> for Operand {
    type Error = Error;
    fn try_from(tok: Token) -> Result<Self, <Self as TryFrom<Token>>::Error> {
        match tok {
            Token::Register(reg) => {
                if reg.is_ctrl_reg() {
                    Ok(Self::CtrReg(reg))
                } else if reg.is_dbg_reg() {
                    Ok(Self::DbgReg(reg))
                } else if reg.purpose() == RPurpose::Sgmnt {
                    Ok(Self::SegReg(reg))
                } else {
                    Ok(Self::Reg(reg))
                }
            }
            Token::String(val) => Ok(Self::String(val)),
            Token::Immediate(nm) => Ok(Self::Imm(nm)),
            Token::SymbolRef(val) => Ok(Self::SymbolRef(*val)),
            _ => Err(Error::new(
                format!(
                    "failed to create operand from \"{}\" token",
                    tok.to_string()
                ),
                3,
            )),
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
            Self::Unknown(val) => val.to_string(),
            Self::Comma => ','.to_string(),
            Self::Closure(pfx, ctt) => format!("{pfx}({ctt})"),
            Self::Modifier(content) => {
                let mut string = String::new();
                for (i, c) in content.iter().enumerate() {
                    if i != 0 {
                        string.push(':');
                    }
                    string.push_str(&c.to_string());
                }
                string
            }
            Self::Error(e) => format!("{e}"),
            Self::None => String::new(),
        }
    }
}
