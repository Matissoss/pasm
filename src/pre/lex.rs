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
        segment::Segment,
        size::Size,
    },
};
use std::path::PathBuf;

pub struct Lexer;
impl Lexer {
    pub fn parse_file(file: Vec<Vec<Token>>) -> Vec<Result<(ASTNode, usize), RASMError>> {
        let mut ast_tree: Vec<Result<(ASTNode, usize), RASMError>> = Vec::new();
        for (line_count, line) in file.into_iter().enumerate() {
            if line.is_empty() {
                continue;
            }

            let mut node: Option<ASTNode> = None;
            let mut error: Option<RASMError> = None;
            match line.first() {
                Some(Token::Label(lbl)) => node = Some(ASTNode::Label(lbl.to_string())),
                Some(Token::Closure('#', str)) => node = Some(ASTNode::Attributes(str.to_string())),
                Some(Token::Keyword(Keyword::Bits)) => {
                    if let Some(Token::Immediate(bits)) = line.get(1) {
                        let uint32 = bits.get_as_u32();
                        if let Ok(n) = u8::try_from(uint32) {
                            node = Some(ASTNode::Bits(n));
                        } else {
                            ast_tree.push(Err(RASMError::no_tip(
                                Some(line_count),
                                Some(format!("Couldn't fit number {} in 8-bits", uint32)),
                            )));
                        }
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
                Some(Token::Keyword(Keyword::Include)) => match make_include(line) {
                    Ok(i) => node = Some(ASTNode::Include(i)),
                    Err(mut e) => {
                        e.set_line(line_count);
                        error = Some(e)
                    }
                },
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
                Some(Token::Keyword(Keyword::Math)) => match make_eval(line) {
                    Ok(n) => node = Some(ASTNode::MathEval(n.0, n.1)),
                    Err(mut e) => {
                        e.set_line(line_count);
                        error = Some(e)
                    }
                },
                Some(Token::Mnemonic(_)) => match make_ins(line) {
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
                        Some("Unexpected start of line"),
                        Some("Consider starting line with instruction, !global, section declaration or label declaration")
                    )));
                }
            }

            if let Some(node_t) = node {
                ast_tree.push(Ok((node_t, line_count)));
            } else if let Some(error_t) = error {
                ast_tree.push(Err(error_t));
            }
        }
        ast_tree
    }
}

fn make_include(line: Vec<Token>) -> Result<PathBuf, RASMError> {
    if let Some(Token::Unknown(s) | Token::String(s)) = line.get(1) {
        Ok(PathBuf::from(s))
    } else {
        Err(RASMError::no_tip(
            None,
            Some("Tried to use include, but without file name"),
        ))
    }
}

fn make_eval(mut line: Vec<Token>) -> Result<(String, String), RASMError> {
    if line.is_empty() {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make mathematical evaluation from nothing"),
        ));
    }
    if line.len() > 3 {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make mathematical evaluation from too many tokens"),
        ));
    }
    // we assert that first (index 0) element is math keyword
    if line.get(1).is_none() || line.get(2).is_none() {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make mathematical const without name/content"),
        ));
    }
    let eval = match line.pop().unwrap() {
        Token::Closure('$', m) => m,
        Token::Immediate(n) => n.to_string(),
        _ => {
            return Err(RASMError::no_tip(
                None,
                Some("Tried to make value from nothing"),
            ))
        }
    };
    let name = if let Token::Unknown(name) = line.pop().unwrap() {
        name
    } else {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make mathematical const's name with incompatible token"),
        ));
    };
    Ok((name, eval))
}

fn make_ins(line: Vec<Token>) -> Result<Instruction, RASMError> {
    if line.is_empty() {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make instruction from nothing"),
        ));
    }
    let mut mnems: Vec<Mnm> = Vec::new();
    let mut tmp_buf: Vec<Token> = Vec::new();
    let mut iter = line.into_iter();
    while let Some(t) = iter.next() {
        if let Token::Mnemonic(m) = t {
            mnems.push(m);
        } else {
            if t != Token::Comma {
                tmp_buf.push(t);
            }
            break;
        }
    }

    let mut ops = [None, None, None, None, None];
    let mut opi = 0;
    while let Some(t) = iter.next() {
        if t == Token::Comma {
            if !tmp_buf.is_empty() {
                ops[opi] = Some(make_op(tmp_buf)?);
                if opi > 5 {
                    return Err(RASMError::no_tip(
                        None,
                        Some("More than max operands in instruction (5) were used!"),
                    ));
                }
                opi += 1;
                tmp_buf = Vec::new();
            }
        } else {
            tmp_buf.push(t);
        }
    }
    if !tmp_buf.is_empty() {
        ops[opi] = Some(make_op(tmp_buf)?);
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
            _ => Some(mnems[1]),
        }
    };

    Ok(Instruction {
        mnem: mnems[0],
        addt,
        oprs: ops,
        line: 0,
    })
}

fn make_op(mut line: Vec<Token>) -> Result<Operand, RASMError> {
    if line.is_empty() {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make operand from nothing"),
        ));
    }

    if line.len() == 1 {
        return Operand::try_from(line.pop().unwrap());
    }

    if line.len() == 2 {
        match (&line[0], &line[1]){
             (Token::Closure(' ', m), Token::Keyword(k))
            |(Token::Keyword(k), Token::Closure(' ', m)) =>
                return Ok(Operand::Mem(Mem::new(m, Size::try_from(*k).unwrap_or(Size::Unknown))?)),
             (Token::Modifier(r, m), Token::Keyword(k))
            |(Token::Keyword(k), Token::Modifier(r, m)) => {
                let size = match Size::try_from(*k){
                    Ok(s) => s,
                    Err(_) => return Err(RASMError::no_tip(
                        None,
                        Some(format!("Couldn't parse size specifier `{}`", k.to_string())),
                    ))
                };
                let r = if let Token::Register(r) = **r {
                    if r.purpose() == crate::shr::reg::Purpose::Sgmnt {
                        r
                    } else {
                        return Err(RASMError::no_tip(None, Some("Registers used in segments must be segment-purposed")))
                    }
                } else {
                    return Err(RASMError::no_tip(None, Some(format!("Couldn't make register from \"{}\" (asserted that modifier is segment)",
                            r.to_string()
                    ))))
                };
                if m.is_none() {
                    return Err(RASMError::no_tip(None, Some("Expected memory address at second (idx 1) modifier")));
                }
                let mem = if let Token::Closure(' ', s) = *m.clone().unwrap() {
                    Mem::new(&s, size)?
                } else {
                    return Err(RASMError::no_tip(None, Some("Expected memory address as second (idx 1) modifier")));
                };
                return Ok(Operand::Segment(Segment { segment: r, address: mem}));
            }
            _ => return Err(RASMError::no_tip(
                None,
                Some("Tried to make unexpected operand from two tokens; expected memory address (or segment) along with size specifier".to_string()),
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
