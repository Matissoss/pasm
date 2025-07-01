// pasm - src/core/mer.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::{RString, SMALLVEC_TOKENS_LEN},
    pre::tok::Token,
    shr::{
        ast::{ASTNode, Instruction, Operand},
        error::RError as Error,
        ins::Mnemonic,
        kwd::Keyword,
        mem::Mem,
        size::Size,
        smallvec::SmallVec,
    },
};
use std::path::PathBuf;

pub fn mer(
    file: Vec<SmallVec<Token, SMALLVEC_TOKENS_LEN>>,
) -> Vec<Result<(ASTNode, usize), Error>> {
    let mut ast_tree: Vec<Result<(ASTNode, usize), Error>> = Vec::new();
    for (mut line_count, mut line) in file.into_iter().enumerate() {
        line_count += 1;
        if line.is_empty() {
            continue;
        }

        let mut node: Option<ASTNode> = None;
        let mut error: Option<Error> = None;
        match line.first() {
            Some(Token::Label(lbl)) => node = Some(ASTNode::Label(lbl.clone())),
            Some(Token::Closure('#', s)) => node = Some(ASTNode::Attributes(s.clone())),
            Some(Token::Keyword(Keyword::Align)) => {
                if let Some(Token::Immediate(bits)) = line.get(1) {
                    let uint32 = bits.get_as_u32();
                    if let Ok(n) = u16::try_from(uint32) {
                        node = Some(ASTNode::Align(n));
                    } else {
                        let mut er = Error::new("you tried to use align directive, but you did not provide a 16-bit valid number", 8);
                        er.set_line(line_count);
                        ast_tree.push(Err(er));
                    }
                } else {
                    let mut er = Error::new("you tried to use align directive, but you did not provide a 16-bit valid number", 8);
                    er.set_line(line_count);
                    error = Some(er);
                }
            }
            Some(Token::Keyword(Keyword::Bits)) => {
                if let Some(Token::Immediate(bits)) = line.get(1) {
                    let uint32 = bits.get_as_u32();
                    if let Ok(n) = u8::try_from(uint32) {
                        node = Some(ASTNode::Bits(n));
                    } else {
                        let mut er = Error::new("you tried to use bits directive, but you did not provide a 8-bit valid number", 8);
                        er.set_line(line_count);
                        ast_tree.push(Err(er));
                    }
                } else {
                    let mut er = Error::new("you tried to use bits directive, but you did not provide a 8-bit valid number", 8);
                    er.set_line(line_count);
                    error = Some(er);
                }
            }
            Some(Token::Keyword(Keyword::Section)) => {
                if let Some(Token::String(str)) = line.pop() {
                    node = Some(ASTNode::Section(str.clone()));
                } else {
                    let mut er = Error::new(
                        "you tried to use section directive, but you did not provide string",
                        8,
                    );
                    er.set_line(line_count);
                    error = Some(er);
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
            Some(Token::Keyword(Keyword::Include)) => match make_include(line) {
                Ok(i) => node = Some(ASTNode::Include(i)),
                Err(mut e) => {
                    e.set_line(line_count);
                    error = Some(e)
                }
            },
            Some(Token::Keyword(Keyword::Extern)) => {
                if let Some(Token::String(etrn)) = line.get(1) {
                    node = Some(ASTNode::Extern(etrn.clone()));
                } else {
                    let mut er = Error::new(
                        "you tried to use extern directive, but you did not provide string",
                        8,
                    );
                    er.set_line(line_count);
                    error = Some(er);
                }
            }
            Some(Token::Keyword(Keyword::Define)) => match make_eval(line) {
                Ok(n) => node = Some(ASTNode::Define(n.0, n.1)),
                Err(mut e) => {
                    e.set_line(line_count);
                    error = Some(e)
                }
            },
            Some(Token::Modifier(m)) => match m.len() {
                2..4 => {
                    let mut er = false;
                    let mut sae = false;
                    let mut zero = false;
                    let mut mask = None;

                    for m in &m[1..] {
                        match m {
                            Token::Register(r) => {
                                if !r.is_mask() {
                                    error = Some(Error::new_wline("expected mask register at mnemonic modifer index 1, found other-purposed register", 8, line_count));
                                    break;
                                }
                                mask = Some(r.to_byte());
                            }
                            Token::Keyword(Keyword::Er) => er = true,
                            Token::Keyword(Keyword::Z) => zero = true,
                            Token::Keyword(Keyword::Sae) => sae = true,
                            _ => {
                                error = Some(Error::new_wline(
                                    "unknown token found in mnemonic modifier",
                                    8,
                                    line_count,
                                ));
                                break;
                            }
                        }
                    }
                    match make_ins(line) {
                        Ok(mut i) => {
                            if sae {
                                i.set_sae();
                            }
                            if zero {
                                i.set_z();
                            }
                            if er {
                                i.set_er();
                            }
                            if let Some(mask) = mask {
                                i.set_mask(mask as u16);
                            }
                            i.line = line_count;
                            node = Some(ASTNode::Ins(i));
                        }
                        Err(mut e) => {
                            e.set_line(line_count);
                            error = Some(e);
                        }
                    }
                }
                _ => ast_tree.push(Err(Error::new_wline(
                    "unexpected start of line",
                    8,
                    line_count,
                ))),
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
            _ => ast_tree.push(Err(Error::new_wline(
                "unexpected start of line",
                8,
                line_count,
            ))),
        }

        if let Some(node_t) = node {
            ast_tree.push(Ok((node_t, line_count)));
        } else if let Some(error_t) = error {
            ast_tree.push(Err(error_t));
        }
    }
    ast_tree
}

fn make_include(line: SmallVec<Token, SMALLVEC_TOKENS_LEN>) -> Result<PathBuf, Error> {
    if let Some(Token::String(s)) = line.get(1) {
        Ok(PathBuf::from(s.to_string()))
    } else {
        Err(Error::new(
            "you tried to use include directive, but you forgot to provide file name (string)",
            8,
        ))
    }
}

fn make_eval(mut line: SmallVec<Token, SMALLVEC_TOKENS_LEN>) -> Result<(RString, u64), Error> {
    if line.is_empty() {
        return Err(Error::new(
            "you tried to use math directive, but you didn't provide anything beside directive",
            8,
        ));
    }
    if line.len() > 3 {
        return Err(Error::new(
            "you tried to use math directive, but you provided more than 3 tokens",
            8,
        ));
    }
    // we assert that first (index 0) element is math keyword
    if line.get(1).is_none() || line.get(2).is_none() {
        return Err(Error::new(
            "you tried to use math directive, but you forgot to provide name and/or content",
            8,
        ));
    }
    let eval = match line.pop().unwrap() {
        Token::Immediate(n) => n.get_as_u64(),
        _ => {
            return Err(Error::new("you tried to use math directive, but you provided unexpected token instead of immediate/math closure", 8))
        }
    };
    let name = if let Token::String(name) = line.pop().unwrap() {
        name.clone()
    } else {
        return Err(Error::new(
            "you tried to use math directive, but you provided invalid token for name",
            8,
        ));
    };
    Ok((name, eval))
}

const OPS: SmallVec<Operand, 5> = SmallVec::new();
fn make_ins(line: SmallVec<Token, SMALLVEC_TOKENS_LEN>) -> Result<Instruction, Error> {
    if line.is_empty() {
        return Err(Error::new("you tried to make instruction from nothing", 8));
    }
    let mut mnems: SmallVec<Mnemonic, 2> = SmallVec::new();
    let mut tmp_buf: SmallVec<Token, 4> = SmallVec::new();
    let mut iter = line.into_iter().into_iter();
    while let Some(t) = iter.next() {
        if let Token::Mnemonic(m) = t {
            if mnems.can_push() {
                mnems.push(m);
            } else {
                return Err(Error::new(
                    "you tried to make instruction with 2+ mnemonics",
                    8,
                ));
            }
        } else if let Token::Modifier(ref a) = t {
            if let Some(Token::Mnemonic(m)) = a.first() {
                if mnems.can_push() {
                    mnems.push(*m);
                } else {
                    return Err(Error::new(
                        "you tried to make instruction with 2+ mnemonics",
                        8,
                    ));
                }
            } else {
                tmp_buf.push(t);
                break;
            }
        } else {
            if t != Token::Comma {
                tmp_buf.push(t);
            }
            break;
        }
    }

    let mut ops = OPS;
    while let Some(t) = iter.next() {
        if t == Token::Comma {
            if !tmp_buf.is_empty() {
                if !ops.can_push() {
                    return Err(Error::new(
                        "you tried to make instruction with too many operands (5+)",
                        8,
                    ));
                }
                ops.push(make_op(tmp_buf)?);
                tmp_buf = SmallVec::new();
            }
        } else {
            tmp_buf.push(t);
        }
    }
    if !tmp_buf.is_empty() {
        ops.push(make_op(tmp_buf)?);
    }
    if mnems.is_empty() {
        return Err(Error::new(
            "you tried to make instruction without mnemonic",
            8,
        ));
    }

    let (mnem, addt) = {
        match mnems.len() {
            1 => (*mnems.get_unchecked(0), None),
            _ => (*mnems.get_unchecked(1), Some(*mnems.get_unchecked(0))),
        }
    };
    Ok(Instruction {
        mnem,
        addt,
        oprs: ops,
        line: 0,
        meta: 0,
    })
}

fn make_op(line: SmallVec<Token, 4>) -> Result<Operand, Error> {
    if line.is_empty() {
        return Err(Error::new("you tried to make operand from nothing", 8));
    }

    if line.len() == 1 {
        return Operand::try_from(line.get_unchecked(0).clone());
    }

    if line.len() == 2 {
        match (line.get_unchecked(0).clone(), line.get_unchecked(1).clone()){
            (Token::Keyword(Keyword::Ref),
             Token::SymbolRef(s)
            ) => {
                let mut s = s.clone();
                s.deref(false);
                return Ok(Operand::SymbolRef(*s.clone()));
            },
            (Token::Keyword(Keyword::Deref), Token::SymbolRef(_)) => return Err(Error::new("you cannot create dereference to a symbol without providing size directive", 8)),
            (Token::Keyword(sz), Token::SymbolRef(mut s)) | (Token::SymbolRef(mut s), Token::Keyword(sz)) => {
                let size = match Size::try_from(sz){
                    Ok(s) => s,
                    Err(_) => return Err(Error::new(
                        "expected size specifier, found unknown directive", 8)),
                };
                s.set_size(size);
                s.deref(true);
                return Ok(Operand::SymbolRef(*s));
            }
            (Token::Keyword(Keyword::Deref), Token::Modifier(m)) => {
                let (sref, sz) = match (m.first(), m.get(1)) {
                    (Some(Token::SymbolRef(s)), Some(Token::Keyword(s1))) => (s, s1),
                    (Some(Token::SymbolRef(s)), None) => (s, &Keyword::Any),
                    _ => return Err(Error::new("you tried to make operand from modifier with deref directive", 8)),
                };
                let sz = if let Ok(sz) = Size::try_from(*sz) {
                    sz
                } else {
                    return Err(Error::new("expected size directive after symbol reference", 8))
                };
                let mut sref = sref.clone();
                sref.deref(true);
                sref.set_size(sz);
                return Ok(Operand::SymbolRef(*sref));
            }
            (Token::Closure(' ', m), Token::Modifier(mods)) |
            (Token::Modifier(mods), Token::Closure(' ', m)) => {
                if mods.len() == 2 {
                    let sz = if let Token::Keyword(modk) = mods[0] {
                        match Size::try_from(modk){
                            Ok(s) => s,
                            Err(_) => return Err(Error::new(
                                "expected size specifier, found unknown directive", 8)),
                        }
                    } else {
                        return Err(Error::new("expected size directive at index 0 of modifier", 8));
                    };
                    if let Token::Keyword(Keyword::Bcst) = mods[1] {} else {
                        return Err(Error::new("expected bcst directive at index 1 of modifier", 8));
                    }
                    let mut m = Mem::new(&m, sz)?;
                    m.set_bcst(true);
                    return Ok(Operand::Mem(m));
                } else {
                    return Err(Error::new("expected modifier of size directive and bcst keyword", 8));
                }
            }
             (Token::Closure(' ', m), Token::Keyword(ref k))
            |(Token::Keyword(ref k), Token::Closure(' ', m)) =>
                return Ok(Operand::Mem(Mem::new(&m, Size::try_from(*k).unwrap_or(Size::Unknown))?)),
             (Token::Modifier(ref m), Token::Keyword(ref k))
            |(Token::Keyword(ref k), Token::Modifier(ref m)) => {
                let size = match Size::try_from(*k){
                    Ok(s) => s,
                    Err(_) => return Err(Error::new(
                        "expected size specifier, found unknown directive", 8)),
                };
                let seg_reg = if let Some(Token::Register(r)) = m.first() {
                    if r.is_sgmnt() {
                        r
                    } else {
                        return Err(Error::new("expected sreg, found other register", 8))
                    }
                } else {
                    return Err(Error::new("expected sreg, found {unknown}", 8))
                };
                let mut mem = if let Some(Token::Closure(' ', s)) = m.get(1) {
                    Mem::new(s, size)?
                } else {
                    return Err(Error::new("expected memory address at index 1", 8))
                };
                mem.set_segment(*seg_reg);
                return Ok(Operand::Mem(mem));
            }
            _ => return Err(Error::new("you tried to make operand from two tokens, but ones you provided could not be parsed into one", 8))
        }
    }

    if line.len() == 3 {
        match (line.first().unwrap(), line.get(1).unwrap().clone(), line.get(2).unwrap()) {
            (Token::Keyword(Keyword::Deref), Token::SymbolRef(mut s), Token::Keyword(sz)) |
            (Token::Keyword(sz), Token::SymbolRef(mut s), Token::Keyword(Keyword::Deref)) => {
                let sz = match Size::try_from(*sz) {
                    Ok(s) => s,
                    Err(_) => return Err(Error::new("you tried to use symbol dereference, but you did not provide valid size directive", 8))
                };
                s.set_size(sz);
                s.deref(true);
                return Ok(Operand::SymbolRef(*s));
            }
            _ => return Err(Error::new("you tried to make operand from three tokens, but ones you provided could not be parsed into one", 8))
        }
    }

    Err(Error::new(
        "you tried to make operand from more than 3 tokens",
        8,
    ))
}
