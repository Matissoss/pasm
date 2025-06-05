// rasmx86_64 - src/pre/tok.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::*,
    shr::{error::RASMError, ins::Mnemonic as Mnm, kwd::Keyword, math, num::Number, reg::Register},
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
    Modifier(Box<Token>, Option<Box<Token>>),
}

pub struct Tokenizer;

fn post_process(toks: Vec<Token>) -> Vec<Token> {
    let mut toks_1 = Vec::new();
    for t in toks {
        if let Token::Closure(PREFIX_VAL, content) = &t {
            let math = math::MathematicalEvaluation::from_str(content);
            if let Ok(eval) = math {
                if math::MathematicalEvaluation::can_eval(&eval) {
                    let result = math::MathematicalEvaluation::eval(eval);
                    if let Some(n) = result {
                        toks_1.push(Token::Immediate(Number::uint64(n)));
                    }
                } else {
                    toks_1.push(t);
                }
            } else {
                toks_1.push(t);
            }
        } else {
            toks_1.push(t);
        }
    }
    toks_1
}

impl Tokenizer {
    pub fn tokenize_line(line: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut tmp_buf: Vec<char> = Vec::new();
        let mut inside_closure: Option<char> = None;
        let mut closure_pfx: Option<char> = None;
        let mut delimeter_count: usize = 0;
        let mut tmp_toks: Vec<Token> = Vec::new();

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
                    tmp_buf = Vec::new();
                }

                (Some(':'), ':') => {
                    if delimeter_count == 0 {
                        if !tmp_buf.is_empty() {
                            tmp_toks.push(Token::make_from(
                                closure_pfx,
                                String::from_iter(tmp_buf.iter()),
                            ));
                            tmp_buf = Vec::new();
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
                        tmp_buf = Vec::new();
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
                        tmp_buf = Vec::new();
                        tokens.push(Token::make_modifier(tmp_toks));
                        tmp_toks = Vec::new();
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

                (Some(CLOSURE_START), ' ') => continue,

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

                (Some(CLOSURE_START), CLOSURE_START) => {
                    delimeter_count += 1;
                    tmp_buf.push(CLOSURE_START)
                }

                (
                    Some(PREFIX_REG | PREFIX_VAL | PREFIX_KWD | PREFIX_SEG | ' ') | None,
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

                (Some(CLOSURE_START | PREFIX_SEG), CLOSURE_END) => {
                    if delimeter_count == 1 {
                        tokens.push(Token::make_closure(
                            closure_pfx.unwrap_or(' '),
                            String::from_iter(tmp_buf.iter()),
                        ));
                        closure_pfx = None;
                        inside_closure = None;
                        tmp_buf = Vec::new();
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
        post_process(tokens)
    }
}

fn make_modf_rec(current: Token, toks: Vec<Token>, idx: usize) -> Token {
    if toks.len() > idx {
        Token::Modifier(
            Box::new(current),
            Some(Box::new(make_modf_rec(toks[idx].clone(), toks, idx + 1))),
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
            Self::Modifier(content, next) => format!(
                "{}{}",
                content.to_string(),
                if let Some(tok) = next {
                    format!(":{}", tok.to_string())
                } else {
                    "".to_string()
                }
            ),
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
        let str = "$()";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(tokens, vec![Token::Closure('$', "".to_string())]);
        let str = "(%rax+%rcx+$4+$20)";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Closure(' ', "%rax+%rcx+$4+$20".to_string())]
        );
        // nested closures!
        let str = "$(@())";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(tokens, vec![Token::Closure('$', "@()".to_string())]);
        // nested nested nest closures!
        let str = "$(@(@(@())))";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(tokens, vec![Token::Closure('$', "@(@(@()))".to_string())]);

        // modifiers!
        let str = "$10:%eax:%rax";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Modifier(
                Box::new(Token::Immediate(Number::uint64(10))),
                Some(Box::new(Token::Modifier(
                    Box::new(Token::Register(Register::EAX)),
                    Some(Box::new(Token::Register(Register::RAX)))
                )))
            )]
        );
        let str = "%fs:(%rax)";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Modifier(
                Box::new(Token::Register(Register::FS)),
                Some(Box::new(Token::Closure(' ', "%rax".to_string())))
            )]
        );
        let str = "%fs:$(%rax)";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Modifier(
                Box::new(Token::Register(Register::FS)),
                Some(Box::new(Token::Closure('$', "%rax".to_string())))
            )]
        );
        let str = "%fs:(%rax):$(%rax)";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![Token::Modifier(
                Box::new(Token::Register(Register::FS)),
                Some(Box::new(Token::Modifier(
                    Box::new(Token::Closure(' ', "%rax".to_string())),
                    Some(Box::new(Token::Closure('$', "%rax".to_string())))
                )))
            )]
        );
        let str = "mov %edi, %ds:(%rbx + %rcx * $4 - $10) !dword";
        let tokens = Tokenizer::tokenize_line(str);
        assert_eq!(
            tokens,
            vec![
                Token::Mnemonic(crate::shr::ins::Mnemonic::MOV),
                Token::Register(Register::EDI),
                Token::Comma,
                Token::Modifier(
                    Box::new(Token::Register(Register::DS)),
                    Some(Box::new(Token::Closure(
                        ' ',
                        "%rbx+%rcx*$4-$10".to_string(),
                    )))
                ),
                Token::Keyword(Keyword::Dword)
            ]
        );
    }
}
