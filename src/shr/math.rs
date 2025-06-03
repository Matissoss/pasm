// rasmx86_64 - src/shr/math.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(unused)]

use std::str::FromStr;

use crate::shr::{error::RASMError, num::Number};

pub struct MathematicalEvaluation;

impl MathematicalEvaluation {
    fn from_str() {}
    fn try_eval() {}
    fn can_eval() {}
    fn eval_unchecked() {}
}

#[derive(PartialEq, Debug, Clone)]
enum MathElement {
    // lhs + rhs
    Add(Box<Self>, Box<Self>),
    // lhs - rhs
    Sub(Box<Self>, Box<Self>),
    // lhs * rhs
    Mul(Box<Self>, Box<Self>),
    // lhs / rhs
    Div(Box<Self>, Box<Self>),
    // lhs % rhs
    Mod(Box<Self>, Box<Self>),
    // lhs & rhs
    And(Box<Self>, Box<Self>),
    // lhs | rhs
    Or(Box<Self>, Box<Self>),
    // lhs ^ rhs
    Xor(Box<Self>, Box<Self>),
    // lhs << rhs
    Lsh(Box<Self>, Box<Self>),
    // lhs >> rhs
    Rsh(Box<Self>, Box<Self>),
    // !Xhs
    Not(Box<Self>),

    // (Xhs)
    Closure(Box<Self>),

    Number(Number),
}

#[derive(PartialEq, Debug, Clone)]
enum Token {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
    And,
    Or,
    Xor,
    Not,
    Lsh,
    Rsh,
    Start, // (
    End,   // )
    Number(Number),
    Unknown(String),
}

type ME = MathElement;
type Error = RASMError;
fn par(tok: Vec<Token>) -> Result<MathElement, Error> {
    dbg!(&tok);
    let mut tmp_toks = Vec::new();
    let mut iclosure = false;
    let mut delimeter_count = 0;
    let mut elements: Vec<MathElement> = Vec::new();
    let mut tmp_num = None;
    let mut tmp_mat: Option<MathElement> = None;
    let mut mode: Option<Token> = None;
    let mut idx = 0;
    while idx < tok.len() {
        if iclosure {
            match &tok[idx] {
                Token::Start => {
                    tmp_toks.push(Token::Start);
                    delimeter_count += 1;
                }
                Token::End => {
                    delimeter_count -= 1;
                    if delimeter_count != 0 {
                        tmp_toks.push(Token::End);
                        idx += 1;
                        continue;
                    }
                    iclosure = false;
                    let tok = MathElement::Closure(Box::new(par(tmp_toks)?));
                    match (elements.pop(), mode.take()) {
                        (Some(lhs), Some(mode_1)) => {
                            elements.push(mer2(&mode_1, lhs, tok));
                        }
                        (None, Some(Token::Not)) => {
                            elements.push(MathElement::Not(Box::new(tok)));
                        }
                        (None, None) => elements.push(tok),
                        (Some(lhs), None) => panic!("lhs = {:?}", lhs),
                        (_, Some(mode)) => {
                            let lhs = if let Some(num) = tmp_num {
                                MathElement::Number(num)
                            } else {
                                panic!("Sign is set, but no number :)");
                            };
                            elements.push(mer2(&mode, lhs, tok));
                        }
                        _ => panic!("Unexpected - {:?} - {:?}", elements, mode),
                    }
                    tmp_toks = Vec::new();
                }
                _ => tmp_toks.push(tok[idx].clone()),
            }
            idx += 1;
            continue;
        }
        match tok[idx] {
            Token::Start => {
                iclosure = true;
                idx += 1;
                delimeter_count = 1
            }
            Token::Add => {
                mode = Some(Token::Add);
                idx += 1
            }
            Token::Sub => {
                mode = Some(Token::Sub);
                idx += 1
            }
            Token::Mul => {
                mode = Some(Token::Mul);
                idx += 1
            }
            Token::Div => {
                mode = Some(Token::Div);
                idx += 1
            }
            Token::Mod => {
                mode = Some(Token::Mod);
                idx += 1
            }
            Token::And => {
                mode = Some(Token::And);
                idx += 1
            }
            Token::Or => {
                mode = Some(Token::Or);
                idx += 1
            }
            Token::Xor => {
                mode = Some(Token::Xor);
                idx += 1
            }
            Token::Lsh => {
                mode = Some(Token::Lsh);
                idx += 1
            }
            Token::Rsh => {
                mode = Some(Token::Rsh);
                idx += 1
            }
            Token::Not => {
                mode = Some(Token::Not);
                idx += 1
            }
            Token::Number(n) => {
                if mode == Some(Token::Not) {
                    if let Some(t) = elements.pop() {
                        elements.push(MathElement::Not(Box::new(t)));
                    }
                    idx += 1;
                    mode = None;
                    continue;
                }
                if tmp_num.is_none() && tmp_mat.is_none() {
                    if mode.is_none() {
                        tmp_num = Some(n);
                        idx += 1;
                        continue;
                    } else {
                        if let Some(t) = elements.pop() {
                            tmp_mat = Some(t);
                        } else {
                            return Err(Error::no_tip(
                                None,
                                Some("Expected lhs and rhs, only found sign."),
                            ));
                        }
                    }
                }
                if mode.is_none() {
                    return Err(Error::no_tip(None, Some("Expected mode, found none")));
                }
                let lhs = {
                    if let Some(n) = tmp_mat {
                        tmp_mat = None;
                        n.clone()
                    } else if let Some(n) = tmp_num {
                        ME::Number(n)
                    } else {
                        panic!("Unexpected 1")
                    }
                };
                elements.push(mer2(&mode.unwrap(), lhs, MathElement::Number(n)));
                mode = None;
                tmp_num = None;
                idx += 1;
                continue;
            }
            Token::Unknown(_) => {
                return Err(Error::no_tip(
                    None,
                    Some("Expected number, found {unknown}"),
                ))
            }
            _ => idx += 1,
        }
    }
    if let Some(tmp_mat) = tmp_mat {
        elements.push(tmp_mat);
    }
    if let Some(mode) = mode {
        let tmp_mat = elements.pop().expect("should be some");
        #[allow(clippy::single_match)]
        match mode {
            Token::Not => elements.push(MathElement::Not(Box::new(tmp_mat))),
            _ => {}
        }
    }
    Ok(elements.pop().unwrap())
}

fn mer2(mode: &Token, lhs: MathElement, rhs: MathElement) -> MathElement {
    match mode {
        Token::Add => ME::Add(Box::new(lhs), Box::new(rhs)),
        Token::Sub => ME::Sub(Box::new(lhs), Box::new(rhs)),
        Token::Mul => ME::Mul(Box::new(lhs), Box::new(rhs)),
        Token::Div => ME::Div(Box::new(lhs), Box::new(rhs)),
        _ => panic!("Unsupported case"),
    }
}

fn tok(str: &str) -> Vec<Token> {
    let mut tmp_buf = Vec::new();
    let mut tokens = Vec::new();

    for c in str.chars() {
        match c {
            '+' | '-' | '*' | '/' | '&' | '|' | '%' | '!' | '^' | '(' | ')' | ' ' | '\t' => {
                if !tmp_buf.is_empty() {
                    tokens.push(make_tok(tmp_buf));
                    tmp_buf = Vec::new();
                }
                match c {
                    '+' => tokens.push(Token::Add),
                    '-' => tokens.push(Token::Sub),
                    '*' => tokens.push(Token::Mul),
                    '/' => tokens.push(Token::Div),
                    '&' => tokens.push(Token::And),
                    '|' => tokens.push(Token::Or),
                    '^' => tokens.push(Token::Xor),
                    '%' => tokens.push(Token::Mod),
                    '!' => tokens.push(Token::Not),
                    '(' => tokens.push(Token::Start),
                    ')' => tokens.push(Token::End),
                    _ => {}
                }
            }
            _ => tmp_buf.push(c),
        }
    }
    if !tmp_buf.is_empty() {
        tokens.push(make_tok(tmp_buf));
    }
    tokens
}

fn make_tok(vec: Vec<char>) -> Token {
    let str = String::from_iter(vec.iter());
    if let Ok(num) = Number::from_str(&str) {
        Token::Number(num)
    } else {
        Token::Unknown(str)
    }
}

#[cfg(test)]
mod math_tests {
    use super::*;
    #[test]
    fn tok_test() {
        let toks = tok("10 + 20");
        assert_eq!(
            toks,
            vec![
                Token::Number(Number::UInt8(10)),
                Token::Add,
                Token::Number(Number::UInt8(20))
            ]
        );
        let toks = tok("10 * 20");
        assert_eq!(
            toks,
            vec![
                Token::Number(Number::UInt8(10)),
                Token::Mul,
                Token::Number(Number::UInt8(20))
            ]
        );
    }
    #[test]
    fn par_test() {
        let toks = tok("10 + 20");
        let result = par(toks);
        assert_eq!(
            result,
            Ok(MathElement::Add(
                Box::new(MathElement::Number(Number::UInt8(10))),
                Box::new(MathElement::Number(Number::UInt8(20)))
            ))
        );

        let toks = tok("10 + 20 * 20");
        assert_eq!(
            toks,
            vec![
                Token::Number(Number::UInt8(10)),
                Token::Add,
                Token::Number(Number::UInt8(20)),
                Token::Mul,
                Token::Number(Number::UInt8(20))
            ]
        );
        let result = par(toks);
        assert_eq!(
            result,
            Ok(MathElement::Mul(
                Box::new(MathElement::Add(
                    Box::new(MathElement::Number(Number::UInt8(10))),
                    Box::new(MathElement::Number(Number::UInt8(20)))
                )),
                Box::new(MathElement::Number(Number::UInt8(20)))
            ))
        );
        let res = par(tok("!(10 + 20 * 20)"));
        assert_eq!(
            res,
            Ok(MathElement::Not(Box::new(MathElement::Closure(Box::new(
                MathElement::Mul(
                    Box::new(MathElement::Add(
                        Box::new(MathElement::Number(Number::UInt8(10))),
                        Box::new(MathElement::Number(Number::UInt8(20)))
                    )),
                    Box::new(MathElement::Number(Number::UInt8(20)))
                )
            )))))
        );
        let res_1 = tok("(10 + 20 * 20) + (10 + 20)");
        assert_eq!(
            res_1,
            vec![
                Token::Start,
                Token::Number(Number::UInt8(10)),
                Token::Add,
                Token::Number(Number::UInt8(20)),
                Token::Mul,
                Token::Number(Number::UInt8(20)),
                Token::End,
                Token::Add,
                Token::Start,
                Token::Number(Number::UInt8(10)),
                Token::Add,
                Token::Number(Number::UInt8(20)),
                Token::End,
            ]
        );
        let res_1 = par(res_1);
        assert_eq!(
            res_1,
            Ok(MathElement::Add(
                // (10 + 20 * 20)
                Box::new(MathElement::Closure(Box::new(MathElement::Mul(
                    // 10 + 20
                    Box::new(MathElement::Add(
                        // 10
                        Box::new(MathElement::Number(Number::UInt8(10))),
                        // 20
                        Box::new(MathElement::Number(Number::UInt8(20))),
                    )),
                    // * 20
                    Box::new(MathElement::Number(Number::UInt8(20)))
                )))),
                // rhs
                // (10 + 20)
                Box::new(MathElement::Closure(Box::new(MathElement::Add(
                    // 10
                    Box::new(MathElement::Number(Number::UInt8(10))),
                    // 20
                    Box::new(MathElement::Number(Number::UInt8(20))),
                ))))
            ))
        );
        let res_1 = par(tok("(10 + 20 * 20) + (10 + (20 + 20))"));
        assert_eq!(
            res_1,
            Ok(MathElement::Add(
                // (10 + 20 * 20)
                Box::new(MathElement::Closure(Box::new(MathElement::Mul(
                    // 10 + 20
                    Box::new(MathElement::Add(
                        // 10
                        Box::new(MathElement::Number(Number::UInt8(10))),
                        // 20
                        Box::new(MathElement::Number(Number::UInt8(20))),
                    )),
                    // * 20
                    Box::new(MathElement::Number(Number::UInt8(20)))
                )))),
                // rhs
                // (10 + (20 + 20))
                Box::new(MathElement::Closure(Box::new(MathElement::Add(
                    // 10
                    Box::new(MathElement::Number(Number::UInt8(10))),
                    // (20 + 20)
                    Box::new(MathElement::Closure(Box::new(MathElement::Add(
                        // 20
                        Box::new(MathElement::Number(Number::UInt8(20))),
                        // 20
                        Box::new(MathElement::Number(Number::UInt8(20))),
                    ))))
                ))))
            ))
        );
    }
    #[test]
    fn eval_test() {}
}
