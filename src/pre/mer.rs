// rasmx86_64 - src/core/mer.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::{RString, SMALLVEC_TOKENS_LEN},
    pre::tok::Token,
    shr::{
        ast::{ASTNode, Instruction, Operand},
        error::RASMError,
        ins::Mnemonic as Mnm,
        kwd::Keyword,
        mem::Mem,
        size::Size,
        smallvec::SmallVec,
    },
};
use std::path::PathBuf;

pub fn mer(
    file: Vec<SmallVec<Token, SMALLVEC_TOKENS_LEN>>,
) -> Vec<Result<(ASTNode, usize), RASMError>> {
    let mut ast_tree: Vec<Result<(ASTNode, usize), RASMError>> = Vec::new();
    for (line_count, mut line) in file.into_iter().enumerate() {
        if line.is_empty() {
            continue;
        }

        let mut node: Option<ASTNode> = None;
        let mut error: Option<RASMError> = None;
        match line.first() {
            Some(Token::Label(lbl)) => node = Some(ASTNode::Label(lbl.clone())),
            Some(Token::Closure('#', s)) => node = Some(ASTNode::Attributes(s.clone())),
            Some(Token::Keyword(Keyword::Align)) => {
                if let Some(Token::Immediate(bits)) = line.get(1) {
                    let uint32 = bits.get_as_u32();
                    if let Ok(n) = u16::try_from(uint32) {
                        node = Some(ASTNode::Align(n));
                    } else {
                        ast_tree.push(Err(RASMError::no_tip(
                            Some(line_count),
                            Some(format!("Couldn't fit number {} in 86-bits", uint32)),
                        )));
                    }
                } else {
                    error = Some(RASMError::with_tip(
                            Some(line_count),
                            Some("Unexpected end of line after align keyword, expected string, found nothing"),
                            Some("Consider adding something after align keyword")
                        ));
                }
            }
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
                            Some("Unexpected end of line after bits keyword, expected string, found nothing"),
                            Some("Consider adding something after bits keyword")
                        ));
                }
            }
            Some(Token::Keyword(Keyword::Section)) => {
                if let Some(Token::String(str) | Token::Unknown(str)) = line.pop() {
                    node = Some(ASTNode::Section(str.clone()));
                } else {
                    error = Some(RASMError::with_tip(
                            Some(line_count),
                            Some("Unexpected end of line after section keyword, expected string, found nothing"),
                            Some("Consider adding something after section keyword")
                        ));
                }
            }
            Some(Token::Keyword(Keyword::Write)) => {
                node = Some(ASTNode::Write);
            }
            Some(Token::Keyword(Keyword::Alloc)) => {
                node = Some(ASTNode::Alloc);
            }
            Some(Token::Keyword(Keyword::Exec)) => {
                node = Some(ASTNode::Exec);
            }
            Some(Token::Keyword(Keyword::Entry)) => {
                if let Some(Token::String(entr) | Token::Unknown(entr)) = line.pop() {
                    node = Some(ASTNode::Entry(entr.clone()));
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

fn make_include(line: SmallVec<Token, SMALLVEC_TOKENS_LEN>) -> Result<PathBuf, RASMError> {
    if let Some(Token::Unknown(s) | Token::String(s)) = line.get(1) {
        Ok(PathBuf::from(s.to_string()))
    } else {
        Err(RASMError::no_tip(
            None,
            Some("Tried to use include, but without file name"),
        ))
    }
}

fn make_eval(mut line: SmallVec<Token, SMALLVEC_TOKENS_LEN>) -> Result<(RString, u64), RASMError> {
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
        Token::Immediate(n) => n.get_as_u64(),
        _ => {
            return Err(RASMError::no_tip(
                None,
                Some("Tried to make value from nothing"),
            ))
        }
    };
    let name = if let Token::Unknown(name) = line.pop().unwrap() {
        name.clone()
    } else {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make mathematical const's name with incompatible token"),
        ));
    };
    Ok((name, eval))
}

fn make_ins(line: SmallVec<Token, SMALLVEC_TOKENS_LEN>) -> Result<Instruction, RASMError> {
    if line.is_empty() {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make instruction from nothing"),
        ));
    }
    let mut mnems: Vec<Mnm> = Vec::with_capacity(2);
    let mut tmp_buf: Vec<Token> = Vec::with_capacity(6);
    let mut iter = line.into_vec().into_iter();
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
                ops[opi] = Some(make_op(&mut tmp_buf)?);
                if opi > 5 {
                    return Err(RASMError::no_tip(
                        None,
                        Some("More than max operands in instruction (5) were used!"),
                    ));
                }
                opi += 1;
                tmp_buf.clear();
            }
        } else {
            tmp_buf.push(t);
        }
    }
    if !tmp_buf.is_empty() {
        ops[opi] = Some(make_op(&mut tmp_buf)?);
    }
    if mnems.is_empty() {
        return Err(RASMError::no_tip(
            None,
            Some("Tried to make instruction with no mnemonics"),
        ));
    }

    let (mnem, addt) = {
        match mnems.len() {
            1 => (mnems[0], None),
            _ => (mnems[1], Some(mnems[0])),
        }
    };

    Ok(Instruction {
        mnem,
        addt,
        oprs: ops,
        line: 0,
    })
}

fn make_op(line: &mut Vec<Token>) -> Result<Operand, RASMError> {
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
            (Token::Keyword(Keyword::Deref|Keyword::Ref),
             Token::SymbolRef(s)
            ) => {
                let mut s = s.clone();
                s.deref(line[0] == Token::Keyword(Keyword::Deref));
                return Ok(Operand::SymbolRef(*s.clone()));
            }
            (Token::Keyword(Keyword::Deref), Token::Modifier(m)) => {
                let (sref, sz) = match (m.first(), m.get(1)) {
                    (Some(Token::SymbolRef(s)), Some(Token::Keyword(s1))) => (s, s1),
                    (Some(Token::SymbolRef(s)), None) => (s, &Keyword::Any),
                    _ => return Err(RASMError::msg("Unexpected combination of deref keyword with modifier")),
                };
                let sz = if let Ok(sz) = Size::try_from(*sz) {
                    sz
                } else {
                    return Err(RASMError::msg(
                        "Expected size directive after symbolref, found unknown keyword",
                    ));
                };
                let mut sref = sref.clone();
                sref.deref(true);
                sref.set_size(sz);
                return Ok(Operand::SymbolRef(*sref));
            }
             (Token::Closure(' ', m), Token::Keyword(ref k))
            |(Token::Keyword(ref k), Token::Closure(' ', m)) =>
                return Ok(Operand::Mem(Mem::new(m, Size::try_from(*k).unwrap_or(Size::Unknown))?)),
             (Token::Modifier(ref m), Token::Keyword(ref k))
            |(Token::Keyword(ref k), Token::Modifier(ref m)) => {
                let size = match Size::try_from(*k){
                    Ok(s) => s,
                    Err(_) => return Err(RASMError::no_tip(
                        None,
                        Some(format!("Couldn't parse size specifier `{}`", k.to_string())),
                    ))
                };
                let seg_reg = if let Some(Token::Register(r)) = m.first() {
                    if r.is_sgmnt() {
                        r
                    } else {
                        return Err(RASMError::msg("Expected segment-purposed register, found non-segment register"));
                    }
                } else {
                    return Err(RASMError::msg("Expected segment-purposed register, found {unknown}"));
                };
                let mut mem = if let Some(Token::Closure(' ', s)) = m.get(1) {
                    Mem::new(s, size)?
                } else {
                    return Err(RASMError::no_tip(None, Some("Expected memory address as second (idx 1) modifier")));
                };
                mem.set_segment(*seg_reg);
                return Ok(Operand::Mem(mem));
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
