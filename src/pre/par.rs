// pasm - src/pre/par.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use std::{
    str::FromStr,
    collections::HashMap,
};

use crate::{
    pre::tok::Token,
    shr::{
        ast,
        smallvec::SmallVec,
        ins,
        error::Error,
    },
};

pub enum ParToken<'a> {
    Instruction(ast::Instruction<'a>),
    Label,
    None,
}

pub struct ParserStatus<'a> {
    pub attributes: HashMap<&'a str, &'a str>,
    pub current_label: &'a str,
    pub current_section: &'a str,
    pub current_content: Vec<u8>,
}

pub fn par<'a>(parser_status: *mut ParserStatus<'a>, raw_line: &'a str, line_num: usize, tokens: Vec<Token>) -> Result<ParToken<'a>, Error> {
    match tokens.first().expect("Line should have atleast 1 token") {
        Token::String(sls, sle) => {
            let slice = unsafe {
                raw_line.get_unchecked((*sls) as usize..(*sle) as usize)
            };
            // we expect instruction
            if let Ok(m) = ins::Mnemonic::from_str(slice) {
                let mut a_mnem = None;
                let (mut toks_start, mut toks_end) = (1, 1);
                let mut operands: SmallVec<ast::OperandOwned, 4> = SmallVec::new();
                for (i, t) in tokens[1..].iter().enumerate() {
                    match t {
                        Token::String(sls, sle) => {
                            let slice2 = unsafe {
                                raw_line.get_unchecked((*sls) as usize..(*sle) as usize)
                            };
                            if let Ok(m) = ins::Mnemonic::from_str(slice2) {
                                if i == 1 {
                                    a_mnem = Some(m);
                                    continue;
                                } else {
                                    return Err(Error::new_wline("this instruction contains mnemonic in middle of instruction!", panic!("tbd"), line_num));
                                }
                            } else {
                                toks_end += 1;
                            }
                        },
                        Token::Comma => {
                            match make_operand(&tokens[toks_start..toks_end]) {
                                Ok(o) => {
                                    operands.push(o);
                                },
                                Err(mut e) => {
                                    e.set_line(line_num);
                                    return Err(e);
                                }
                            }
                            toks_end += 1;
                            toks_start = toks_end;
                        }
                        Token::Semicolon => {
                            break;
                        }
                        _ => {
                            toks_end += 1;
                        }
                    }
                }
                let mut instruction = ast::Instruction::with_operands(operands);
                instruction.mnemonic = m;
                if let Some(m) = a_mnem {
                    instruction.set_addt(m);
                }
                return Ok(ParToken::Instruction(instruction));
            // we expect directive or label
            } else {
                if let Some(Token::Colon) = tokens.get(1) {
                    unsafe {
                        (&mut*parser_status).attributes.insert("label", slice);
                    };
                    return Ok(ParToken::Label);
                } else if let Some(Token::String(sls, sle)) = tokens.get(1) {
                    let slice2 = unsafe {
                        raw_line.get_unchecked((*sls) as usize..(*sle) as usize)
                    };
                    unsafe {
                        (&mut*parser_status).attributes.insert(slice, slice2);
                    };
                    return Ok(ParToken::None);
                }
            };
            todo!();
        },
        Token::Semicolon => Ok(ParToken::None),
        _ => Err(Error::new_wline("Invalid start of line", panic!("Need to set error code!"), line_num)),
    }
}

fn make_operand<'a>(tokens: &'a [Token]) -> Result<ast::OperandOwned<'a>, Error> {
    todo!();
}
