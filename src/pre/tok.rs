// pasm - src/pre/tok.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::*,
    shr::{
        ast::Operand, error::RError, error::RError as Error, ins::Mnemonic, kwd::Keyword, math,
        num::Number, reg::Register, reloc::RelType, smallvec::SmallVec, symbol::SymbolRef,
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
    Comma,

    String(RString),
    Error(Box<RError>),
    //       pfx   content
    Closure(char, RString),

    Modifier(SharedArr<Self>),

    // {subexpression}
    SubExpr(RString),

    #[default]
    None,
}

pub fn tokl(tmp_buf: &mut Vec<char>, line: &str) -> SmallVec<Token, SMALLVEC_TOKENS_LEN> {
    let mut tokens: SmallVec<Token, SMALLVEC_TOKENS_LEN> = SmallVec::new();

    // inside closure
    let mut iclosure: Option<char> = None;
    // closure prefix
    let mut cprefix: Option<char> = None;

    let mut modf_toks: SmallVec<Token, 6> = SmallVec::new();

    // closure delimeter count
    let mut cdelcount = 0usize;
    // subexpression delimeter count
    let mut sdelcount = 0usize;

    for c in line.chars() {
        match c {
            CLOSURE_START => {
                if cdelcount == 0 && !tmp_buf.is_empty() {
                    tokens.push(Token::make_from(cprefix.take(), striter(tmp_buf)));
                }
                cdelcount += 1
            }
            CLOSURE_END => {
                if cdelcount == 1 {
                    let token =
                        Token::make_closure(cprefix.take().unwrap_or(' '), striter(tmp_buf).into());
                    if modf_toks.is_empty() {
                        tokens.push(token);
                    } else {
                        modf_toks.push(token);
                    }
                    tmp_buf.clear();
                    cdelcount = 0;
                } else {
                    cdelcount -= 1;
                }
            }
            SUBEXPR_CLOSE => {
                if sdelcount == 1 {
                    if !tmp_buf.is_empty() {
                        tokens.push(Token::SubExpr(striter(tmp_buf).into()));
                    }
                    sdelcount = 0;
                } else {
                    sdelcount -= 1;
                }
            }
            SUBEXPR_START => {
                if cdelcount != 0 {
                    continue;
                } else {
                    if sdelcount == 0 && !tmp_buf.is_empty() {
                        tokens.push(Token::make_from(cprefix.take(), striter(tmp_buf)));
                    }
                    sdelcount += 1;
                }
            }
            PREFIX_KWD | PREFIX_REF | PREFIX_REG | PREFIX_VAL | '#' => {
                if cdelcount == 0 {
                    if !tmp_buf.is_empty() && modf_toks.is_empty() {
                        tokens.push(Token::make_from(cprefix.take(), striter(tmp_buf)));
                    }
                    cprefix = Some(c);
                } else {
                    tmp_buf.push(c);
                }
            }
            ':' => {
                if !tmp_buf.is_empty() {
                    modf_toks.push(Token::make_from(cprefix.take(), striter(tmp_buf)));
                }
            }
            '"' => {
                if !(cdelcount != 0 && sdelcount != 0) {
                    if iclosure == Some('"') {
                        tokens.push(Token::String(striter(tmp_buf).into()));
                        iclosure.take();
                    } else {
                        iclosure = Some('"');
                    }
                } else {
                    iclosure.take();
                }
            }
            ',' => {
                if iclosure != Some('"') {
                    if !tmp_buf.is_empty() {
                        tokens.push(Token::make_from(cprefix.take(), striter(tmp_buf)));
                    }
                    tokens.push(Token::Comma);
                } else {
                    tmp_buf.push(',');
                }
            }
            ';' => {
                if !(cdelcount != 0 && sdelcount != 0) && iclosure != Some('"') {
                    break;
                }
            }
            '/' => {
                if !(cdelcount != 0 && sdelcount != 0) && iclosure != Some('"') {
                    if iclosure == Some('/') {
                        break;
                    } else {
                        iclosure = Some('/');
                    }
                } else {
                    iclosure.take();
                }
            }
            ' ' | '\t' => {
                if cdelcount != 0 || sdelcount != 0 || iclosure == Some('"') {
                    tmp_buf.push(c);
                    continue;
                }
                if !modf_toks.is_empty() {
                    if !tmp_buf.is_empty() {
                        modf_toks.push(Token::make_from(cprefix.take(), striter(tmp_buf)));
                    }
                    tokens.push(Token::make_modifier(modf_toks.into_iter()));
                    modf_toks = SmallVec::new();
                } else if !tmp_buf.is_empty() {
                    tokens.push(Token::make_from(cprefix.take(), striter(tmp_buf)));
                }
            }
            _ => tmp_buf.push(c),
        }
    }

    if cdelcount != 0 && !tmp_buf.is_empty() {
        tokens.push(Token::make_closure(
            cprefix.take().unwrap_or(' '),
            striter(tmp_buf).into(),
        ));
        cdelcount = 0;
    }

    if !modf_toks.is_empty() {
        if !tmp_buf.is_empty() {
            modf_toks.push(Token::make_from(cprefix, striter(tmp_buf)));
        }
        tokens.push(Token::make_modifier(modf_toks.into_iter()));
    }

    if !tmp_buf.is_empty() {
        tokens.push(Token::make_from(cprefix, striter(tmp_buf)));
    }

    if cdelcount != 0 {
        tokens.push(Token::Error(Box::new(Error::new(
            "unclosed delimeter `(`",
            0,
        ))));
    }
    if sdelcount != 0 {
        tokens.push(Token::Error(Box::new(Error::new(
            "unclosed delimeter `{`",
            0,
        ))));
    }
    if iclosure == Some('"') {
        tokens.push(Token::Error(Box::new(Error::new(
            "unclosed delimeter `\"`",
            0,
        ))));
    }

    tokens
}

fn striter(v: &mut Vec<char>) -> String {
    let s = String::from_iter(v.iter());
    v.clear();
    s
}

#[cfg(test)]
mod tokn_test {
    use super::*;
    #[test]
    fn tok_test() {
        let mut tmp_buf = Vec::new();
        let line = "add %rax, %rcx";
        assert_eq!(
            tokl(&mut tmp_buf, line).into_iter(),
            vec![
                Token::Mnemonic(Mnemonic::ADD),
                Token::Register(Register::RAX),
                Token::Comma,
                Token::Register(Register::RCX)
            ]
        );
        tmp_buf.clear();
        let line = "_label:";
        assert_eq!(
            tokl(&mut tmp_buf, line).into_iter(),
            vec![Token::Label(RString::from("_label"))]
        );
        let line = "$(10)";
        tmp_buf.clear();
        assert_eq!(
            tokl(&mut tmp_buf, line).into_iter(),
            vec![Token::Immediate(Number::uint64(10))]
        );
        let line = "@sref:rel32 // comment ";
        tmp_buf.clear();
        assert_eq!(
            tokl(&mut tmp_buf, line).into_iter(),
            vec![Token::SymbolRef(Box::new(SymbolRef::new(
                RString::from("sref"),
                None,
                false,
                None,
                Some(RelType::REL32)
            )))]
        );
        tmp_buf.clear();
        let line = "adc (%rax + %rcx * $4 + $10) .qword, %r10";
        assert_eq!(
            tokl(&mut tmp_buf, line).into_iter(),
            vec![
                Token::Mnemonic(Mnemonic::ADC),
                Token::Closure(' ', RString::from("%rax + %rcx * $4 + $10")),
                Token::Keyword(Keyword::Qword),
                Token::Comma,
                Token::Register(Register::R10),
            ]
        );
        tmp_buf.clear();
        let line = "%fs:(%rax + %rcx * $4 + $10) .qword";
        assert_eq!(
            tokl(&mut tmp_buf, line).into_iter(),
            vec![
                Token::Modifier(Shared::from([
                    Token::Register(Register::FS),
                    Token::Closure(' ', RString::from("%rax + %rcx * $4 + $10"))
                ])),
                Token::Keyword(Keyword::Qword),
            ]
        );
        let line = "{k1} {z}";
        tmp_buf.clear();
        assert_eq!(
            tokl(&mut tmp_buf, line).into_iter(),
            vec![
                Token::SubExpr(RString::from("k1")),
                Token::SubExpr(RString::from("z")),
            ]
        );
        let line = "\"Hello, World!\"";
        tmp_buf.clear();
        assert_eq!(
            tokl(&mut tmp_buf, line).into_iter(),
            vec![Token::String(RString::from("Hello, World!"))]
        );
    }
}

impl Token {
    fn make_modifier(mut toks: Vec<Self>) -> Self {
        match toks.len() {
            1 => {
                return Token::Label(if let Token::String(s) = toks.pop().unwrap() {
                    s
                } else {
                    RString::from("")
                })
            }
            2 => match &toks[..2] {
                [Token::SymbolRef(symb), Token::Keyword(k)] => {
                    let rt = if let Ok(rt) = RelType::try_from(&Token::Keyword(*k)) {
                        rt
                    } else {
                        return Token::Modifier(toks.into());
                    };
                    let mut s = *symb.clone();
                    s.set_reltype(rt);
                    return Token::SymbolRef(Box::new(s));
                }
                [Token::SymbolRef(symb), Token::Immediate(n)] => {
                    let mut s = *symb.clone();
                    s.set_addend(n.get_as_i32());
                    return Token::SymbolRef(Box::new(s));
                }
                _ => return Token::Modifier(toks.into()),
            },
            3 => match &toks[..3] {
                [Token::SymbolRef(symb), Token::Immediate(n), Token::Keyword(k)]
                | [Token::SymbolRef(symb), Token::Keyword(k), Token::Immediate(n)] => {
                    let rt = if let Ok(rt) = RelType::try_from(&Token::Keyword(*k)) {
                        rt
                    } else {
                        return Token::Modifier(toks.into());
                    };
                    let offset = n.get_as_i32();
                    let mut symb = symb.clone();
                    symb.set_addend(offset);
                    symb.set_reltype(rt);
                    return Token::SymbolRef(Box::new(*symb));
                }
                _ => return Token::Modifier(toks.into()),
            },
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
            Some(PREFIX_REF) => Token::SymbolRef(Box::new(SymbolRef::new(
                val.into(),
                None,
                false,
                None,
                None,
            ))),
            Some(PREFIX_REG) => match Register::from_str(&val) {
                Ok(reg) => Self::Register(reg),
                Err(_) => Self::String(val.into()),
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
                    Self::String(val.into())
                }
            }
            _ => {
                #[cfg(not(feature = "refresh"))]
                if let Ok(mnm) = Mnemonic::from_str(&val) {
                    return Self::Mnemonic(mnm);
                }

                if let Ok(reg) = Register::from_str(&val) {
                    Self::Register(reg)
                } else if let Ok(num) = Number::from_str(&val) {
                    Self::Immediate(num)
                } else if let Ok(dir) = Keyword::from_str(&val) {
                    Self::Keyword(dir)
                } else {
                    Self::String(val.into())
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
            Token::Register(reg) => Ok(Self::Register(reg)),
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
            Self::SubExpr(s) => format!("{{{s}}}"),
            Self::None => String::new(),
        }
    }
}
