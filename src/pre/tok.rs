// rasmx86_64 - src/pre/tok.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::*,
    shr::{
        error::RASMError, ins::Mnemonic as Mnm, kwd::Keyword, math, num::Number, reg::Register,
        symbol::SymbolRef,
    },
};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Register(Register),
    Immediate(Number),
    Keyword(Keyword),
    Mnemonic(Mnm),
    Label(String),
    SymbolRef(String),
    SymbolRefExt(SymbolRef),
    String(String),
    Segment(String),
    UnknownSegment(String, RASMError),
    UnknownKeyword(String),
    UnknownReg(String),
    UnknownVal(String, RASMError),
    Unknown(String),
    Comma,

    Error(RASMError),
    //       pfx   content
    Closure(char, String),
    //       pfx     content   next token (can be another modifier)
    Modifier(Box<Token>, Box<Token>),
}

pub struct Tokenizer;

fn post_process(toks: &mut [Token]) {
    use math::MathematicalEvaluation as MathEval;
    for t in toks {
        match t {
            Token::Closure(PREFIX_VAL, content) => {
                match MathEval::from_str(content) {
                    Ok(m) => {
                        *t = Token::Immediate(Number::uint64(MathEval::eval(m).unwrap_or(0)));
                    }
                    Err(e) => {
                        *t = Token::Error(e);
                        continue;
                    }
                };
            }
            Token::Closure(PREFIX_REF, content) => match SymbolRef::try_new(content) {
                Ok(s) => *t = Token::SymbolRefExt(s),
                Err(e) => *t = Token::Error(e),
            },
            _ => continue,
        }
    }
}

impl Tokenizer {
    pub fn tokenize_line(line: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::with_capacity(8);
        let mut tmp_buf: Vec<char> = Vec::with_capacity(32);
        let mut inside_closure: Option<char> = None;
        let mut closure_pfx: Option<char> = None;
        let mut delimeter_count: usize = 0;
        let mut tmp_toks: Vec<Token> = Vec::with_capacity(4);

        for b in line.as_bytes() {
            let c = *b as char;
            match (inside_closure, c) {
                (_, COMMENT_S) => break,

                (None, '"') => {
                    inside_closure = Some('"');
                }
                (Some('"'), '"') => {
                    tokens.push(Token::String(String::from_iter(tmp_buf.iter())));
                    tmp_buf.clear();
                    inside_closure = None;
                }

                (Some('"'), c) => tmp_buf.push(c),

                (None, ':') => {
                    tokens.push(Token::Label(String::from_iter(tmp_buf.iter())));
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
                            String::from_iter(tmp_buf.iter()),
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
                        tokens.push(Token::make_modifier(tmp_toks));
                        tmp_toks = Vec::with_capacity(4);
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
                    Some(PREFIX_REG | PREFIX_VAL | PREFIX_REF | PREFIX_KWD | PREFIX_SEG | ' ')
                    | None,
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
                            String::from_iter(tmp_buf.iter()),
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
                tokens.push(Token::make_modifier(tmp_toks))
            } else {
                tokens.push(Token::make_from(
                    inside_closure,
                    String::from_iter(tmp_buf.iter()),
                ));
            }
        } else if !tmp_toks.is_empty() {
            tokens.push(Token::make_modifier(tmp_toks));
        }
        post_process(&mut tokens);
        tokens
    }
}

fn make_modf_rec(current: Token, toks: Vec<Token>, idx: usize) -> Token {
    if toks.len() > idx {
        Token::Modifier(
            Box::new(current),
            Box::new(make_modf_rec(toks[idx].clone(), toks, idx + 1)),
        )
    } else {
        current
    }
}

impl Token {
    fn make_modifier(toks: Vec<Self>) -> Self {
        if let Some(tok) = toks.first() {
            make_modf_rec(tok.clone(), toks, 1)
        } else {
            Token::Unknown("".to_string())
        }
    }
    fn make_closure(prefix: char, val: String) -> Self {
        Self::Closure(prefix, val)
    }
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
            Some(CLOSURE_START) => Self::Closure(' ', val),
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
            Self::Register(reg) => format!("{}{}", PREFIX_REG, reg.to_string()),
            Self::Immediate(v) => format!("{}{}", PREFIX_VAL, v.to_string()),
            Self::Keyword(kwd) => kwd.to_string(),
            Self::Mnemonic(m) => m.to_string(),
            Self::Label(lbl) => lbl.to_string(),
            Self::SymbolRef(lbl) => format!("{}{}", PREFIX_REF, lbl),
            Self::String(str) => str.to_string(),
            Self::UnknownReg(str) => format!("{}{}", PREFIX_REG, str),
            Self::UnknownVal(str, _) => format!("{}{}", PREFIX_VAL, str),
            Self::Error(_) => "".to_string(),
            Self::Unknown(val) => val.to_string(),
            Self::UnknownKeyword(kwd) => format!("{}{}", PREFIX_KWD, kwd),
            Self::Segment(s) => s.to_string(),
            Self::Comma => ','.to_string(),
            Self::UnknownSegment(s, _) => s.to_string(),
            Self::Closure(pfx, content) => format!("{pfx}({content})"),
            Self::Modifier(content, next) => {
                format!("{}:{}", content.to_string(), next.to_string())
            }
            Self::SymbolRefExt(r) => r.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tok_test() {
        let str = "()";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(tokens, vec![Token::Closure(' ', "".to_string())]);
        let str = "(%rax+%rcx+$4+$20)";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Closure(' ', "%rax+%rcx+$4+$20".to_string())]
        );

        // modifiers!
        let str = "$10:%eax:%rax";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Modifier(
                Box::new(Token::Immediate(Number::uint64(10))),
                Box::new(Token::Modifier(
                    Box::new(Token::Register(Register::EAX)),
                    Box::new(Token::Register(Register::RAX))
                ))
            )]
        );
        let str = "%fs:(%rax)";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Modifier(
                Box::new(Token::Register(Register::FS)),
                Box::new(Token::Closure(' ', "%rax".to_string()))
            )]
        );
        let str = "%fs:$(%rax)";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Modifier(
                Box::new(Token::Register(Register::FS)),
                Box::new(Token::Closure('$', "%rax".to_string()))
            )]
        );
        let str = "%fs:(%rax):$(%rax)";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Modifier(
                Box::new(Token::Register(Register::FS)),
                Box::new(Token::Modifier(
                    Box::new(Token::Closure(' ', "%rax".to_string())),
                    Box::new(Token::Closure('$', "%rax".to_string()))
                ))
            )]
        );
        let str = "mov %edi, %ds:(%rbx + %rcx * $4 - $10) .dword";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![
                Token::Mnemonic(crate::shr::ins::Mnemonic::MOV),
                Token::Register(Register::EDI),
                Token::Comma,
                Token::Modifier(
                    Box::new(Token::Register(Register::DS)),
                    Box::new(Token::Closure(' ', "%rbx+%rcx*$4-$10".to_string(),))
                ),
                Token::Keyword(Keyword::Dword)
            ]
        );
        let str = "\"Hello, World!\"";
        assert_eq!(
            Tokenizer::tokenize_line(str),
            vec![Token::String("Hello, World!".to_string())]
        );
    }
}
