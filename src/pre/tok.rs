// pasm - src/pre/tok.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::*,
    shr::{
        ast::Operand, dir::Directive, error::Error, ins::Mnemonic, math, num::Number,
        reg::Register, reloc::RelType, smallvec::SmallVec, symbol::SymbolRef,
    },
};

use std::iter::Iterator;
use std::mem::ManuallyDrop;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Register(Register),
    Immediate(Number),
    Directive(Directive),
    Mnemonic(Mnemonic),
    Label(&'a str),
    SymbolRef(Box<SymbolRef<'a>>),
    Comma,

    String(&'a str),
    Error(Box<Error>),
    //       pfx   content
    Closure(char, &'a str),

    Modifier(Box<[Self]>),

    // {subexpression}
    SubExpr(&'a str),

    EOL,
}

pub fn tokl(line: &str) -> impl Iterator<Item = Token> {
    let mut tstart: usize = 0;
    let mut tend: usize = 0;
    let bline = line.as_bytes();

    let mut tokens: SmallVec<Token, 16> = SmallVec::new();

    // inside closure
    let mut iclosure: Option<char> = None;
    // closure prefix
    let mut cprefix: Option<char> = None;

    let mut modf_toks: SmallVec<Token, 6> = SmallVec::new();

    // closure delimeter count
    let mut cdelcount = 0usize;
    // subexpression delimeter count
    let mut sdelcount = 0usize;

    for c in line.as_bytes() {
        let c = *c as char;
        match c {
            CLOSURE_START => {
                if iclosure != Some('"') {
                    if cdelcount == 0 && tend != tstart {
                        tokens.push(Token::make_from(cprefix.take(), str(bline, tstart, tend)));
                    }
                    cdelcount += 1;
                    tend += 1;
                    tstart = tend;
                } else {
                    tend += 1;
                }
            }
            CLOSURE_END => {
                if iclosure != Some('"') && cdelcount == 1 {
                    let token = Token::make_closure(
                        cprefix.take().unwrap_or(' '),
                        str(bline, tstart, tend),
                    );
                    if modf_toks.is_empty() {
                        tokens.push(token);
                    } else {
                        modf_toks.push(token);
                    }
                    tend += 1;
                    tstart = tend;
                    cdelcount = 0;
                } else {
                    cdelcount = cdelcount.saturating_sub(1);
                    tend += 1;
                }
            }
            SUBEXPR_CLOSE => {
                if iclosure != Some('"') && sdelcount == 1 {
                    if tend != tstart {
                        tokens.push(Token::SubExpr(str(bline, tstart, tend)));
                    }
                    tend += 1;
                    tstart = tend;
                    sdelcount = 0;
                } else {
                    cdelcount = cdelcount.saturating_sub(1);
                    tend += 1;
                }
            }
            SUBEXPR_START => {
                if iclosure != Some('"') && cdelcount == 0 {
                    if sdelcount == 0 && tend != tstart {
                        tokens.push(Token::make_from(cprefix.take(), str(bline, tstart, tend)));
                    }
                    sdelcount += 1;
                    tend += 1;
                    tstart = tend;
                } else {
                    tend += 1;
                }
            }
            PREFIX_VAL | PREFIX_REF | '#' => {
                if cdelcount == 0 {
                    if tend != tstart && modf_toks.is_empty() {
                        tokens.push(Token::make_from(cprefix.take(), str(bline, tstart, tend)));
                    }
                    tend += 1;
                    tstart = tend;
                    cprefix = Some(c);
                } else {
                    tend += 1;
                }
            }
            ':' => {
                if tend != tstart {
                    modf_toks.push(Token::make_from(cprefix.take(), str(bline, tstart, tend)));
                }
                tend += 1;
                tstart = tend;
            }
            '"' => {
                if !(cdelcount != 0 && sdelcount != 0) {
                    if iclosure == Some('"') && cprefix != Some('\\') {
                        tokens.push(Token::String(str(bline, tstart, tend)));
                        iclosure.take();
                    } else {
                        iclosure = Some('"');
                    }
                } else {
                    iclosure.take();
                }
                tend += 1;
                tstart = tend;
            }
            ',' => {
                if iclosure != Some('"') {
                    if tend != tstart {
                        tokens.push(Token::make_from(cprefix.take(), str(bline, tstart, tend)));
                    }
                    tend += 1;
                    tstart = tend;
                    tokens.push(Token::Comma);
                } else {
                    tend += 1;
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
                    tend += 1;
                    continue;
                }
                if !modf_toks.is_empty() {
                    if tend != tstart {
                        modf_toks.push(Token::make_from(cprefix.take(), str(bline, tstart, tend)));
                    }
                    tokens.push(Token::make_modifier(modf_toks.into_vec()));
                    modf_toks = SmallVec::new();
                } else if tend != tstart {
                    tokens.push(Token::make_from(cprefix.take(), str(bline, tstart, tend)));
                }
                tend += 1;
                tstart = tend;
            }
            _ => {
                tend += 1;
            }
        }
    }

    if cdelcount != 0 && tend != tstart {
        tokens.push(Token::make_closure(
            cprefix.take().unwrap_or(' '),
            str(bline, tstart, tend),
        ));
        cdelcount = 0;
        tend = tstart;
    }

    if !modf_toks.is_empty() {
        if tend != tstart {
            modf_toks.push(Token::make_from(cprefix, str(bline, tstart, tend)));
            tend = tstart;
        }
        tokens.push(Token::make_modifier(modf_toks.into_vec()));
    }

    if tend != tstart {
        tokens.push(Token::make_from(cprefix, str(bline, tstart, tend)));
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
    tokens.push(Token::EOL);

    tokens.into_iter()
}

// we can assert that code is already valid UTF-8, because of `utils.rs:split_str_ref`
#[inline(always)]
fn str(buf: &[u8], start: usize, end: usize) -> &str {
    unsafe { std::str::from_utf8_unchecked(&buf[start..end]) }
}

impl<'a> Token<'a> {
    #[inline(always)]
    fn make_modifier(mut toks: Vec<Self>) -> Self {
        match toks.len() {
            1 => {
                return Token::Label(if let Token::String(s) = toks.pop().unwrap() {
                    s
                } else {
                    ""
                })
            }
            2 => match &toks[..2] {
                [Token::SymbolRef(symb), Token::Directive(k)] => {
                    let rt = if let Ok(rt) = RelType::try_from(&Token::Directive(*k)) {
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
                [Token::SymbolRef(symb), Token::Immediate(n), Token::Directive(k)]
                | [Token::SymbolRef(symb), Token::Directive(k), Token::Immediate(n)] => {
                    let rt = if let Ok(rt) = RelType::try_from(&Token::Directive(*k)) {
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
    #[inline(always)]
    fn make_closure(prefix: char, val: &'a str) -> Self {
        use math::MathematicalEvaluation as MathEval;
        match prefix {
            PREFIX_VAL => match MathEval::from_str(val) {
                Ok(m) => Token::Immediate(Number::uint64(MathEval::eval(m).unwrap_or(0))),
                Err(e) => Token::Error(Box::new(e)),
            },
            _ => Self::Closure(prefix, val),
        }
    }
    #[inline(always)]
    fn make_from(prefix: Option<char>, val: &'a str) -> Self {
        match prefix {
            Some(PREFIX_REF) => {
                Token::SymbolRef(Box::new(SymbolRef::new(val, None, false, None, None)))
            }
            Some(CLOSURE_START) => Self::Closure(' ', val),
            _ => {
                if let Ok(reg) = Register::from_str(val) {
                    Self::Register(reg)
                } else if let Some(num) = Number::from_str(val) {
                    Self::Immediate(num)
                } else if let Ok(dir) = Directive::from_str(val) {
                    Self::Directive(dir)
                } else {
                    #[cfg(not(feature = "refresh"))]
                    if let Ok(mnm) = Mnemonic::from_str(val) {
                        return Self::Mnemonic(mnm);
                    }
                    Self::String(val)
                }
            }
        }
    }
}

impl<'a> TryFrom<&'a Token<'a>> for RelType {
    type Error = ();
    fn try_from(tok: &'a Token<'a>) -> Result<Self, <Self as TryFrom<&'a Token<'a>>>::Error> {
        match tok {
            Token::Directive(Directive::Rel32) => Ok(Self::REL32),
            Token::Directive(Directive::Rel16) => Ok(Self::REL16),
            Token::Directive(Directive::Rel8) => Ok(Self::REL8),
            Token::Directive(Directive::Abs32) => Ok(Self::ABS32),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<Token<'a>> for Operand<'a> {
    type Error = Error;
    #[inline(always)]
    fn try_from(tok: Token<'a>) -> Result<Self, <Self as TryFrom<Token<'a>>>::Error> {
        match tok {
            Token::Register(reg) => Ok(Self::Register(reg)),
            Token::String(val) => Ok(Self::String(ManuallyDrop::new(val.into()))),
            Token::Immediate(nm) => Ok(Self::Imm(nm)),
            Token::SymbolRef(val) => Ok(Self::SymbolRef(ManuallyDrop::new(val))),
            Token::Closure(' ', _) => Err(Error::new(
                "you cannot create memory addressing without using size directive",
                3,
            )),
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
impl ToString for Token<'_> {
    fn to_string(&self) -> String {
        match self {
            Self::Register(reg) => format!("{}{}", PREFIX_REG, reg.to_string()),
            Self::Immediate(v) => format!("{}{}", PREFIX_VAL, v.to_string()),
            Self::Directive(dir) => dir.to_string(),
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
            Self::EOL => String::new(),
        }
    }
}

#[cfg(test)]
mod tokn_test {
    use super::*;
    #[test]
    fn tok_test() {
        let line = "add rax, rcx";
        assert_eq!(
            tokl(line).collect::<Vec<Token>>(),
            vec![
                Token::Mnemonic(Mnemonic::ADD),
                Token::Register(Register::RAX),
                Token::Comma,
                Token::Register(Register::RCX),
                Token::EOL,
            ]
        );
        let line = "_label:";
        assert_eq!(
            tokl(line).collect::<Vec<Token>>(),
            vec![Token::Label("_label"), Token::EOL]
        );
        let line = "$(10)";
        assert_eq!(
            tokl(line).collect::<Vec<Token>>(),
            vec![Token::Immediate(Number::uint64(10)), Token::EOL]
        );
        let line = "@sref:rel32 // comment ";
        assert_eq!(
            tokl(line).collect::<Vec<Token>>(),
            vec![
                Token::SymbolRef(Box::new(SymbolRef::new(
                    "sref",
                    None,
                    false,
                    None,
                    Some(RelType::REL32)
                ))),
                Token::EOL
            ]
        );
        let line = "adc (rax + rcx * 4 + 10) qword, r10";
        assert_eq!(
            tokl(line).collect::<Vec<Token>>(),
            vec![
                Token::Mnemonic(Mnemonic::ADC),
                Token::Closure(' ', "rax + rcx * 4 + 10"),
                Token::Directive(Directive::Qword),
                Token::Comma,
                Token::Register(Register::R10),
                Token::EOL,
            ]
        );
        let line = "fs:(%rax + %rcx * $4 + $10) qword";
        assert_eq!(
            tokl(line).collect::<Vec<Token>>(),
            vec![
                Token::Modifier(Box::from([
                    Token::Register(Register::FS),
                    Token::Closure(' ', "%rax + %rcx * $4 + $10")
                ])),
                Token::Directive(Directive::Qword),
                Token::EOL,
            ]
        );
        let line = "{k1} {z}";
        assert_eq!(
            tokl(line).collect::<Vec<Token>>(),
            vec![Token::SubExpr("k1"), Token::SubExpr("z"), Token::EOL]
        );
        let line = "\"(Hello {, World!\"";
        assert_eq!(
            tokl(line).collect::<Vec<Token>>(),
            vec![Token::String("(Hello {, World!"), Token::EOL]
        );
    }
}
