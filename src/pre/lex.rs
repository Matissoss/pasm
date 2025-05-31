// rasmx86_64 - src/core/lex.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    pre::tok::Token,
    shr::{
        ast::{ASTNode, Instruction, Operand},
        error::RASMError,
        ins::Mnemonic as Mnm,
        kwd::Keyword,
        mem::Mem,
        num::Number,
        segment::Segment,
        size::Size,
        symbol::Visibility,
        var::{VType, VarContent, Variable},
    },
};
use std::borrow::Cow;
use std::str::FromStr;

pub struct Lexer;
impl Lexer {
    pub fn parse_file<'a>(file: Vec<Vec<Token>>) -> Vec<Result<(ASTNode<'a>, usize), RASMError>> {
        let mut line_count: usize = 0;
        let mut ast_tree: Vec<Result<(ASTNode, usize), RASMError>> = Vec::new();
        for line in file {
            line_count += 1;
            if line.is_empty() {
                continue;
            }

            let mut node: Option<ASTNode> = None;
            let mut error: Option<RASMError> = None;
            match line.first() {
                Some(Token::Label(lbl)) => node = Some(ASTNode::Label(lbl.to_string())),
                Some(Token::Keyword(Keyword::Const | Keyword::Uninit | Keyword::Ronly)) => {
                    match make_var(Cow::Owned(line)) {
                        Ok(var) => node = Some(ASTNode::Variable(var)),
                        Err(mut tmp_error) => {
                            tmp_error.set_line(line_count);
                            error = Some(tmp_error)
                        }
                    }
                }
                Some(Token::Keyword(Keyword::Bits)) => {
                    if let Some(Token::Immediate(bits)) = line.get(1) {
                        let uint8 = match bits.get_uint() {
                            Some(n) => match Number::squeeze_u64(n) {
                                Number::UInt8(n) => n,
                                _ => {
                                    ast_tree.push(Err(RASMError::no_tip(
                                        Some(line_count),
                                        Some(format!("Couldn't fit number {} in 8-bits", n)),
                                    )));
                                    continue;
                                }
                            },
                            _ => {
                                ast_tree.push(Err(RASMError::no_tip(
                                    Some(line_count),
                                    Some(format!(
                                        "Couldn't fit number {} in 8-bit unsigned integer",
                                        bits.to_string()
                                    )),
                                )));
                                continue;
                            }
                        };
                        node = Some(ASTNode::Bits(uint8));
                    } else {
                        error = Some(RASMError::with_tip(
                            Some(line_count),
                            Some("Unexpected end of line after entry keyword, expected string, found nothing"),
                            Some("Consider adding something after entry keyword")
                        ));
                    }
                }
                Some(Token::Keyword(Keyword::Entry)) => {
                    if let Some(Token::String(entr) | Token::Unknown(entr)) = line.get(1) {
                        node = Some(ASTNode::Entry(entr.to_string()));
                    } else {
                        error = Some(RASMError::with_tip(
                            Some(line_count),
                            Some("Unexpected end of line after entry keyword, expected string, found nothing"),
                            Some("Consider adding something after entry keyword")
                        ));
                    }
                }
                Some(Token::Keyword(Keyword::Extern)) => {
                    if let Some(Token::String(etrn) | Token::Unknown(etrn)) = line.get(1) {
                        node = Some(ASTNode::Extern(etrn.to_string()));
                    } else {
                        error = Some(RASMError::with_tip(
                            Some(line_count),
                            Some("Unexpected end of line after extern keyword, expected string, found nothing"),
                            Some("Consider adding something after extern keyword")
                        ));
                    }
                }
                Some(Token::Keyword(Keyword::Global)) => {
                    if let Some(Token::String(glob) | Token::Unknown(glob)) = line.get(1) {
                        node = Some(ASTNode::Global(glob.to_string()));
                    } else {
                        error = Some(RASMError::with_tip(
                            Some(line_count),
                            Some("Unexpected end of line after global keyword, expected string, found nothing"),
                            Some("Consider adding something after global keyword")
                        ));
                    }
                }
                Some(Token::Mnemonic(_)) => match make_ins(&line) {
                    Ok(mut i) => {
                        i.line = line_count;
                        node = Some(ASTNode::Ins(i));
                    }
                    Err(mut e) => {
                        e.set_line(line_count);
                        error = Some(e);
                    }
                },
                Some(Token::Unknown(s)) => ast_tree.push(Err(RASMError::no_tip(
                    Some(line_count),
                    Some(format!("Tried to start line with unknown mnemonic `{s}`")),
                ))),
                _ => {
                    ast_tree.push(Err(RASMError::with_tip(
                        Some(line_count),
                        Some("Unexpected start of line!"),
                        Some("Consider starting line with instruction, !global, section declaration or label declaration")
                    )));
                }
            }

            if let Some(node_t) = node {
                ast_tree.push(Ok((node_t, line_count - 1)));
            } else if let Some(error_t) = error {
                ast_tree.push(Err(error_t));
            }
        }
        ast_tree
    }
}

fn make_ins(line: &[Token]) -> Result<Instruction, RASMError> {
    if line.is_empty() {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make instruction from nothing"),
        ));
    }
    let mut mnems: Vec<Mnm> = Vec::new();
    let mut tmp_buf: Vec<&Token> = Vec::new();
    let mut idx = 0;
    for t in &line[0..] {
        if let Token::Mnemonic(m) = t {
            mnems.push(*m);
            idx += 1;
        } else {
            if t != &Token::Comma {
                tmp_buf.push(t);
            }
            idx += 1;
            break;
        }
    }

    let mut ops = Vec::new();
    for t in &line[idx..] {
        if t == &Token::Comma {
            if !tmp_buf.is_empty() {
                ops.push(make_op(&tmp_buf)?);
                tmp_buf = Vec::new();
            }
        } else {
            tmp_buf.push(t);
        }
    }
    if !tmp_buf.is_empty() {
        ops.push(make_op(&tmp_buf)?);
    }
    if mnems.is_empty() {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make instruction with no mnemonics"),
        ));
    }

    let addt = {
        match mnems.len() {
            1 => None,
            _ => Some(mnems[1..].to_vec()),
        }
    };

    Ok(Instruction {
        mnem: mnems[0],
        addt,
        oprs: ops,
        line: 0,
    })
}

fn make_op(line: &[&Token]) -> Result<Operand, RASMError> {
    if line.is_empty() {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make operand from nothing"),
        ));
    }

    if line.len() == 1 {
        match Operand::try_from(line[0]) {
            Ok(o) => return Ok(o),
            Err(_) => {
                return Err(RASMError::no_tip(
                    None,
                    Some(format!(
                        "Failed to create operand from {}",
                        line[0].to_string()
                    )),
                ))
            }
        }
    }

    if line.len() == 2 {
        match (&line[0], &line[1]){
             (Token::MemAddr(m), Token::Keyword(k))
            |(Token::Keyword(k), Token::MemAddr(m)) => {
                match Mem::new(m, Size::try_from(*k).unwrap_or(Size::Unknown)){
                    Ok(m) => return Ok(Operand::Mem(m)),
                    Err(e) => return Err(e),
                }
            },
             (Token::Segment(s), Token::Keyword(k))
            |(Token::Keyword(k), Token::Segment(s)) => {
                let size = match Size::try_from(*k){
                    Ok(s) => s,
                    Err(_) => return Err(RASMError::no_tip(
                        None,
                        Some(format!("Couldn't parse size specifier `{}`", k.to_string())),
                    ))
                };
                let mut segment = Segment::from_str(s)?;
                segment.address.set_size(size);
                return Ok(Operand::Segment(segment));
            }
            _ => return Err(RASMError::no_tip(
                None,
                Some("Tried to make unexpected operand from two tokens; expected memory address along with size specifier".to_string()),
            ))
        }
    }

    Err(RASMError::no_tip(
        None,
        Some(format!(
            "Tried to make operand from too large set of tokens ({})",
            line.len()
        )),
    ))
}

fn make_var(line: Cow<'_, Vec<Token>>) -> Result<Variable<'_>, RASMError> {
    let vtype = match line.first() {
        Some(Token::Keyword(k)) => {
            match k {
                Keyword::Uninit => VType::Uninit,
                Keyword::Const  => VType::Const,
                Keyword::Ronly  => VType::Readonly,
                _               => return Err(RASMError::no_tip(
                    None,
                    Some("Unexpected keyword found at index 0; expected variable type"),
                ))
            }
        },
        Some(_) => return Err(RASMError::no_tip(
            None,
            Some("Unexpected token at index 0; expected variable type"),
        )),
        None => return Err(RASMError::with_tip(
            None,
            Some("Expected variable type at index 0, found nothing"),
            Some("Consider adding variable type on index like `!ronly` (.rodata), `!const` (.data) or `!uninit` (.bss)")
        ))
    };

    let vname = match line.get(1) {
        Some(t) => t.to_string(),
        None => {
            return Err(RASMError::no_tip(
                None,
                Some("Expected variable name at index 1; found nothing"),
            ))
        }
    };

    let size = match line.get(2) {
        Some(Token::Keyword(k)) => match Size::try_from(*k) {
            Ok(s) => <Size as Into<u8>>::into(s) as u32,
            Err(_) => {
                return Err(RASMError::no_tip(
                    None,
                    Some(format!(
                        "Couldn't parse keyword `{}` into size specifier",
                        k.to_string()
                    )),
                ))
            }
        },
        Some(t) => {
            if vtype == VType::Uninit {
                if let Token::Immediate(n) = t {
                    match n.get_uint(){
                        Some(n) => n as u32,
                        None    => return Err(RASMError::no_tip(
                            None,
                            Some("Invalid size specifier at index 2; expected 32-bit unsigned integer"),
                        ))
                    }
                } else {
                    return Err(RASMError::no_tip(
                        None,
                        Some("Unexpected token found at index 2; expected keyword (!byte, !word, !dword or !qword) or 32-bit unsigned integer"),
                    ));
                }
            } else {
                return Err(RASMError::no_tip(
                    None,
                    Some("Unexpected token found at index 2; expected keyword (!byte, !word, !dword or !qword)"),
                ));
            }
        }
        None => {
            return Err(RASMError::no_tip(
                None,
                Some("Expected variable name at index 2; found nothing"),
            ))
        }
    };

    let mut content = String::new();
    for i in &line[3..] {
        content.push_str(&i.to_string());
    }
    let content = par_str(content);
    match (&vtype, &content){
        (VType::Const|VType::Readonly, VarContent::String(_)|VarContent::Number(_))|
        (VType::Uninit, VarContent::Uninit) => {},
        (VType::Const|VType::Readonly, VarContent::Uninit) =>
        return Err(RASMError::no_tip(
            None,
            Some("Variable type mismatch: declared variable is const/readonly, but content is undefined"),
        )),
        (VType::Uninit, VarContent::String(_)|VarContent::Number(_)) =>
        return Err(RASMError::no_tip(
            None,
            Some("Variable type mismatch: declared variable is uninitialized, but content is defined"),
        ))
    }
    Ok(Variable {
        name: std::borrow::Cow::Owned(vname),
        vtype,
        size: size as u32,
        content,
        visibility: Visibility::Local,
    })
}

#[inline]
fn par_str<'a>(cont: String) -> VarContent<'a> {
    if let Ok(n) = Number::from_str(&cont) {
        VarContent::Number(n)
    } else if cont.is_empty() {
        VarContent::Uninit
    } else {
        VarContent::String(Cow::Owned(cont.as_bytes().to_vec()))
    }
}
