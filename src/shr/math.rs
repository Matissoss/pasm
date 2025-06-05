// rasmx86_64 - src/shr/math.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(unused)]

use std::str::FromStr;

use crate::shr::{error::RASMError as Error, num::Number};

pub struct MathematicalEvaluation;

impl MathematicalEvaluation {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: &str) -> Result<MathElement, Error> {
        par(tok(str))
    }
    pub fn can_eval(math: &MathElement) -> bool {
        true
    }
    pub fn eval(math: MathElement) -> Option<u64> {
        match math {
            MathElement::Lsh(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::Lsh),
            MathElement::Rsh(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::Rsh),
            MathElement::Add(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::Add),
            MathElement::Sub(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::Sub),
            MathElement::Mul(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::Mul),
            MathElement::Div(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::Div),
            MathElement::Mod(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::Mod),
            MathElement::And(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::And),
            MathElement::Or(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::Or),
            MathElement::Xor(lhs, rhs) => Self::eval_rec(*lhs, *rhs, Mode::Xor),
            MathElement::Not(lhs) => {
                Self::eval_rec(*lhs, MathElement::Number(Number::uint64(0)), Mode::Not)
            }
            _ => None,
        }
    }
    fn eval_rec(lhs: MathElement, rhs: MathElement, mode: Mode) -> Option<u64> {
        let (mut lhs_n, mut rhs_n): (Option<Number>, Option<Number>) = (None, None);
        let (mut lhs_t, mut rhs_t): (Option<MathElement>, Option<MathElement>) = (None, None);
        match (lhs, rhs) {
            (MathElement::Number(l), MathElement::Number(r)) => {
                lhs_n = Some(l);
                rhs_n = Some(r);
            }
            (l, MathElement::Number(r)) => {
                rhs_n = Some(r);
                lhs_t = Some(l);
            }
            (MathElement::Number(l), r) => {
                lhs_n = Some(l);
                rhs_t = Some(r);
            }
            (l, r) => {
                lhs_t = Some(l);
                rhs_t = Some(r);
            }
        }
        if let (Some(lhs_n), Some(rhs_n)) = (lhs_n, rhs_n) {
            let lu64 = lhs_n.get_as_u32() as u64;
            let ru64 = rhs_n.get_as_u32() as u64;
            Some(eval(lu64, ru64, mode))
        } else if let (Some(lhs_t), Some(rhs_t)) = (&lhs_t, &rhs_t) {
            Self::eval_rec(lhs_t.clone(), rhs_t.clone(), mode)
        } else if let (Some(lhs_n), Some(rhs_t)) = (lhs_n, &rhs_t) {
            let (int_lhs, int_rhs, int_mode) = deserialize(rhs_t.clone());
            let rhs_n = Self::eval_rec(
                *int_lhs?,
                if int_mode == Mode::Not {
                    MathElement::Number(Number::uint64(0))
                } else {
                    *int_rhs.unwrap()
                },
                int_mode,
            )?;

            Some(eval(lhs_n.get_as_u32() as u64, rhs_n, mode))
        } else if let (Some(lhs_t), Some(rhs_n)) = (&lhs_t, rhs_n) {
            let (int_lhs, int_rhs, int_mode) = deserialize(lhs_t.clone());
            let lhs_n = Self::eval_rec(
                *int_lhs?,
                if int_mode == Mode::Not {
                    MathElement::Number(Number::uint64(0))
                } else {
                    *int_rhs.unwrap()
                },
                int_mode,
            )?;
            Some(eval(lhs_n, rhs_n.get_as_u32() as u64, mode))
        } else if let (Some(lhs_t), Some(rhs_t)) = (lhs_t, rhs_t) {
            let (lhs, rhs, mode) = deserialize(lhs_t);
            let (lhs, rhs) = if mode == Mode::Not {
                (
                    lhs.unwrap(),
                    Box::new(MathElement::Number(Number::uint64(0))),
                )
            } else {
                (lhs.unwrap(), rhs.unwrap())
            };
            let lhs_n = Self::eval_rec(*lhs, *rhs, mode)?;
            let (lhs, rhs, mode) = deserialize(rhs_t);
            let (lhs, rhs) = if mode == Mode::Not {
                (
                    lhs.unwrap(),
                    Box::new(MathElement::Number(Number::uint64(0))),
                )
            } else {
                (lhs.unwrap(), rhs.unwrap())
            };
            let rhs_n = Self::eval_rec(*lhs, *rhs, mode)?;
            Some(eval(lhs_n, rhs_n, mode))
        } else {
            None
        }
    }
}
fn deserialize(lhs: MathElement) -> (Option<Box<MathElement>>, Option<Box<MathElement>>, Mode) {
    match lhs {
        MathElement::Add(lhs, rhs) => (Some(lhs), Some(rhs), Mode::Add),
        MathElement::Sub(lhs, rhs) => (Some(lhs), Some(rhs), Mode::Sub),
        MathElement::Mul(lhs, rhs) => (Some(lhs), Some(rhs), Mode::Mul),
        MathElement::Div(lhs, rhs) => (Some(lhs), Some(rhs), Mode::Div),
        MathElement::Mod(lhs, rhs) => (Some(lhs), Some(rhs), Mode::Mod),
        MathElement::Rsh(lhs, rhs) => (Some(lhs), Some(rhs), Mode::Rsh),
        MathElement::Lsh(lhs, rhs) => (Some(lhs), Some(rhs), Mode::Lsh),
        MathElement::And(lhs, rhs) => (Some(lhs), Some(rhs), Mode::And),
        MathElement::Or(lhs, rhs) => (Some(lhs), Some(rhs), Mode::Or),
        MathElement::Xor(lhs, rhs) => (Some(lhs), Some(rhs), Mode::Xor),
        MathElement::Not(lhs) => (Some(lhs), None, Mode::Not),
        MathElement::Closure(c) => deserialize(*c),
        _ => panic!("Case unsupported - {:?}", lhs),
    }
}
fn eval(lu64: u64, ru64: u64, mode: Mode) -> u64 {
    match mode {
        Mode::Add => lu64 + ru64,
        Mode::Sub => lu64 - ru64,
        Mode::Mul => lu64 * ru64,
        Mode::Div => lu64 / ru64,
        Mode::Mod => lu64 % ru64,
        Mode::Lsh => lu64 << ru64,
        Mode::Rsh => lu64 >> ru64,
        Mode::And => lu64 & ru64,
        Mode::Or => lu64 | ru64,
        Mode::Xor => lu64 ^ ru64,
        Mode::Not => !lu64,
        _ => panic!("Boolean Algebra is not supported yet!"),
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum MathElement {
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

#[derive(PartialEq, Debug, Clone, Copy)]
enum Mode {
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
    Lt,
    Gt,
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

    Lt, // <
    Gt, // >

    Number(Number),
    Unknown(String),
}

type ME = MathElement;
fn par(tok: Vec<Token>) -> Result<MathElement, Error> {
    let mut tmp_toks = Vec::new();
    let mut iclosure = false;
    let mut delimeter_count = 0;
    let mut elements: Vec<MathElement> = Vec::new();
    let mut tmp_num = None;
    let mut tmp_mat: Option<MathElement> = None;
    let mut mode: Option<Mode> = None;
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
                        (None, Some(Mode::Not)) => {
                            elements.push(MathElement::Not(Box::new(tok)));
                        }
                        (None, None) => elements.push(tok),
                        (Some(lhs), None) => panic!("lhs = {:?}", lhs),
                        (_, Some(mode)) => {
                            let lhs = if let Some(num) = tmp_num {
                                MathElement::Number(num)
                            } else {
                                panic!("Sign was set, but no number was found!")
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
        match &tok[idx] {
            Token::Start => {
                iclosure = true;
                idx += 1;
                delimeter_count = 1
            }
            Token::Gt => {
                if mode == Some(Mode::Gt) {
                    mode = Some(Mode::Rsh);
                } else if mode == Some(Mode::Rsh) {
                    return Err(Error::no_tip(
                        None,
                        Some("Tried to use unknown operator <<<"),
                    ));
                } else {
                    mode = Some(Mode::Gt);
                }
                idx += 1;
            }
            Token::Lt => {
                if mode == Some(Mode::Lt) {
                    mode = Some(Mode::Lsh);
                } else if mode == Some(Mode::Lsh) {
                    return Err(Error::no_tip(
                        None,
                        Some("Tried to use unknown operator <<<"),
                    ));
                } else {
                    mode = Some(Mode::Lt);
                }
                idx += 1;
            }
            Token::Add => {
                mode = Some(Mode::Add);
                idx += 1
            }
            Token::Sub => {
                mode = Some(Mode::Sub);
                idx += 1
            }
            Token::Mul => {
                mode = Some(Mode::Mul);
                idx += 1
            }
            Token::Div => {
                mode = Some(Mode::Div);
                idx += 1
            }
            Token::Mod => {
                mode = Some(Mode::Mod);
                idx += 1
            }
            Token::And => {
                mode = Some(Mode::And);
                idx += 1
            }
            Token::Or => {
                mode = Some(Mode::Or);
                idx += 1
            }
            Token::Xor => {
                mode = Some(Mode::Xor);
                idx += 1
            }
            Token::Lsh => {
                mode = Some(Mode::Lsh);
                idx += 1
            }
            Token::Rsh => {
                mode = Some(Mode::Rsh);
                idx += 1
            }
            Token::Not => {
                mode = Some(Mode::Not);
                idx += 1
            }
            Token::Number(n) => {
                if mode == Some(Mode::Not) {
                    if let Some(t) = elements.pop() {
                        elements.push(MathElement::Not(Box::new(t)));
                    }
                    idx += 1;
                    mode = None;
                    continue;
                }
                if tmp_num.is_none() && tmp_mat.is_none() {
                    if mode.is_none() {
                        tmp_num = Some(*n);
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
                elements.push(mer2(&mode.unwrap(), lhs, MathElement::Number(*n)));
                mode = None;
                tmp_num = None;
                idx += 1;
                continue;
            }
            Token::Unknown(s) => {
                return Err(Error::no_tip(
                    None,
                    Some(format!(
                        "Expected number, found {{unknown}} string \"{}\"",
                        &s
                    )),
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
            Mode::Not => elements.push(MathElement::Not(Box::new(tmp_mat))),
            _ => {}
        }
    }
    if let Some(s) = elements.pop() {
        Ok(s)
    } else {
        if let Some(tmp_n) = tmp_num {
            Ok(MathElement::Number(tmp_n))
        } else {
            Err(Error::no_tip(
                None,
                Some(
                    "Internal Error (should not happen): Tried to pop elements, while it was empty :(",
                ),
            ))
        }
    }
}

fn mer2(mode: &Mode, lhs: MathElement, rhs: MathElement) -> MathElement {
    match mode {
        Mode::Add => ME::Add(Box::new(lhs), Box::new(rhs)),
        Mode::Sub => ME::Sub(Box::new(lhs), Box::new(rhs)),
        Mode::Mul => ME::Mul(Box::new(lhs), Box::new(rhs)),
        Mode::Div => ME::Div(Box::new(lhs), Box::new(rhs)),
        Mode::Rsh => ME::Rsh(Box::new(lhs), Box::new(rhs)),
        Mode::Lsh => ME::Lsh(Box::new(lhs), Box::new(rhs)),
        Mode::And => ME::And(Box::new(lhs), Box::new(rhs)),
        Mode::Or => ME::Or(Box::new(lhs), Box::new(rhs)),
        Mode::Xor => ME::Xor(Box::new(lhs), Box::new(rhs)),
        Mode::Lt | Mode::Gt => panic!("Boolean operations are currently unsupported!"),
        _ => panic!("Unsupported case"),
    }
}

fn tok(str: &str) -> Vec<Token> {
    let mut tmp_buf = Vec::new();
    let mut tokens = Vec::new();

    for c in str.chars() {
        match c {
            '+' | '-' | '*' | '/' | '&' | '|' | '%' | '!' | '^' | '(' | ')' | ' ' | '<' | '>'
            | '\t' => {
                if !tmp_buf.is_empty() {
                    tokens.push(make_tok(tmp_buf));
                    tmp_buf = Vec::new();
                }
                match c {
                    '<' => tokens.push(Token::Lt),
                    '>' => tokens.push(Token::Gt),
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
                Token::Number(Number::uint64(10)),
                Token::Add,
                Token::Number(Number::uint64(20))
            ]
        );
        let toks = tok("10 * 20");
        assert_eq!(
            toks,
            vec![
                Token::Number(Number::uint64(10)),
                Token::Mul,
                Token::Number(Number::uint64(20))
            ]
        );
        let toks = tok("<<");
        assert_eq!(toks, vec![Token::Lt, Token::Lt]);
    }
    #[test]
    fn par_test() {
        let toks = tok("10 * 2");
        let result = par(toks);
        assert_eq!(
            result,
            Ok(MathElement::Mul(
                Box::new(MathElement::Number(Number::uint64(10))),
                Box::new(MathElement::Number(Number::uint64(2)))
            ))
        );
        let toks = tok("10 + 20");
        let result = par(toks);
        assert_eq!(
            result,
            Ok(MathElement::Add(
                Box::new(MathElement::Number(Number::uint64(10))),
                Box::new(MathElement::Number(Number::uint64(20)))
            ))
        );

        let toks = tok("10 + 20 * 20");
        assert_eq!(
            toks,
            vec![
                Token::Number(Number::uint64(10)),
                Token::Add,
                Token::Number(Number::uint64(20)),
                Token::Mul,
                Token::Number(Number::uint64(20))
            ]
        );
        let result = par(toks);
        assert_eq!(
            result,
            Ok(MathElement::Mul(
                Box::new(MathElement::Add(
                    Box::new(MathElement::Number(Number::uint64(10))),
                    Box::new(MathElement::Number(Number::uint64(20)))
                )),
                Box::new(MathElement::Number(Number::uint64(20)))
            ))
        );
        let res = par(tok("!(10 + 20 * 20)"));
        assert_eq!(
            res,
            Ok(MathElement::Not(Box::new(MathElement::Closure(Box::new(
                MathElement::Mul(
                    Box::new(MathElement::Add(
                        Box::new(MathElement::Number(Number::uint64(10))),
                        Box::new(MathElement::Number(Number::uint64(20)))
                    )),
                    Box::new(MathElement::Number(Number::uint64(20)))
                )
            )))))
        );
        let res_1 = tok("(10 + 20 * 20) + (10 + 20)");
        assert_eq!(
            res_1,
            vec![
                Token::Start,
                Token::Number(Number::uint64(10)),
                Token::Add,
                Token::Number(Number::uint64(20)),
                Token::Mul,
                Token::Number(Number::uint64(20)),
                Token::End,
                Token::Add,
                Token::Start,
                Token::Number(Number::uint64(10)),
                Token::Add,
                Token::Number(Number::uint64(20)),
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
                        Box::new(MathElement::Number(Number::uint64(10))),
                        // 20
                        Box::new(MathElement::Number(Number::uint64(20))),
                    )),
                    // * 20
                    Box::new(MathElement::Number(Number::uint64(20)))
                )))),
                // rhs
                // (10 + 20)
                Box::new(MathElement::Closure(Box::new(MathElement::Add(
                    // 10
                    Box::new(MathElement::Number(Number::uint64(10))),
                    // 20
                    Box::new(MathElement::Number(Number::uint64(20))),
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
                        Box::new(MathElement::Number(Number::uint64(10))),
                        // 20
                        Box::new(MathElement::Number(Number::uint64(20))),
                    )),
                    // * 20
                    Box::new(MathElement::Number(Number::uint64(20)))
                )))),
                // rhs
                // (10 + (20 + 20))
                Box::new(MathElement::Closure(Box::new(MathElement::Add(
                    // 10
                    Box::new(MathElement::Number(Number::uint64(10))),
                    // (20 + 20)
                    Box::new(MathElement::Closure(Box::new(MathElement::Add(
                        // 20
                        Box::new(MathElement::Number(Number::uint64(20))),
                        // 20
                        Box::new(MathElement::Number(Number::uint64(20))),
                    ))))
                ))))
            ))
        );
        let res_1 = par(tok("1 << 2"));
        assert_eq!(
            res_1,
            Ok(MathElement::Lsh(
                Box::new(MathElement::Number(Number::uint64(1))),
                Box::new(MathElement::Number(Number::uint64(2))),
            ))
        );
    }
    #[test]
    fn eval_test() {
        let eval = MathElement::Add(
            Box::new(MathElement::Number(Number::uint64(1))),
            Box::new(MathElement::Number(Number::uint64(2))),
        );
        let eval = MathematicalEvaluation::eval(eval);
        assert_eq!(eval, Some(3));
        let eval = MathematicalEvaluation::from_str("(2 * 5) / 5");
        assert!(eval.is_ok());
        let eval = eval.unwrap();
        assert_eq!(MathematicalEvaluation::eval(eval), Some(2));
    }
}
