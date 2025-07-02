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
        label::{Label, LabelAttributes},
        mem::Mem,
        num::Number,
        size::Size,
        smallvec::SmallVec,
        symbol::SymbolType,
        visibility::Visibility,
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
    let mut subexpr: Vec<RString> = Vec::new();
    while let Some(t) = iter.next() {
        if let Token::SubExpr(s) = t {
            subexpr.push(s.clone());
            continue;
        }
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
        if let Token::SubExpr(s) = t {
            subexpr.push(s.clone());
            continue;
        } else if t == Token::Comma {
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
    let mut ins = Instruction {
        mnem,
        addt,
        operands: ops,
        line: 0,
        meta: 0,
    };

    for s in subexpr {
        match &*s {
            "k0" => ins.set_mask(0b000),
            "k1" => ins.set_mask(0b001),
            "k2" => ins.set_mask(0b010),
            "k3" => ins.set_mask(0b011),
            "k4" => ins.set_mask(0b100),
            "k5" => ins.set_mask(0b101),
            "k6" => ins.set_mask(0b110),
            "k7" => ins.set_mask(0b111),
            "sae" => ins.set_sae(),
            "er" => ins.set_er(),
            "z" => ins.set_z(),
            "m" => {}
            "evex" => ins.set_evex(),
            "vex" => ins.set_vex(),
            _ => {
                return Err(Error::new(
                    format!("you tried to use unknown/unsupported subexpression: \"{s}\""),
                    18,
                ))
            }
        }
    }

    Ok(ins)
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

// mer new

pub enum RootNode {
    Format(RString),
    Include(RString),
    Extern(RString),
    Define(RString, Number),
    Bits(u8),
}

pub enum BodyNode {
    Section(RString),

    Label(Label),
    Instruction(Instruction),

    Attributes(RString),
    Bits(u8),
    Align(u16),
    // section attributes
    Alloc,
    Exec,
    Write,
}

pub struct MergerResult {
    pub root: Vec<RootNode>,
    pub body: Vec<BodyNode>,
}

#[allow(unused_assignments)]
pub fn mern(lines: Vec<SmallVec<Token, SMALLVEC_TOKENS_LEN>>) -> Result<MergerResult, Vec<Error>> {
    let mut errors = Vec::new();
    let mut root = Vec::new();
    let mut body = Vec::new();

    for (mut lnum, mut line) in lines.into_iter().enumerate() {
        lnum += 1;
        if line.is_empty() {
            continue;
        }

        // defines if we are in root, set false if we meet first section/label
        let mut inroot: bool = true;
        // scary :D
        let start = unsafe { line.take_owned(0).unwrap() };

        // legend:
        // <something here> - optional
        match start {
            Token::Error(mut e) => errors.push({
                e.set_line(lnum);
                *e
            }),

            // attributes
            Token::Closure('#', attr) => {
                inroot = false;
                let garbage = unsafe { line.take_owned(1) };
                if garbage.is_some() {
                    if let Token::Closure('#', _) = garbage.unwrap() {
                        errors.push(Error::new_wline(
                            "only one #() attribute closure is allowed per line",
                            17,
                            lnum,
                        ));
                    } else {
                        errors.push(Error::new_wline(
                            "found garbage after #() attribute closure",
                            17,
                            lnum,
                        ));
                    }
                    continue;
                }
                body.push(BodyNode::Attributes(attr));
            }
            // assert that layout of line is something like label: <instruction>
            Token::Label(lname) => {
                inroot = false;
                let l = Label {
                    name: lname,
                    debug_line: lnum,
                    ..Default::default()
                };
                body.push(BodyNode::Label(l));
                match unsafe { line.take_owned(1) } {
                    Some(Token::Mnemonic(m)) => {
                        unsafe { line.insert(1, Token::Mnemonic(m)) };
                        match make_instruction(line, 0) {
                            Ok(mut i) => {
                                i.line = lnum;
                                body.push(BodyNode::Instruction(i))
                            }
                            Err(e) => errors.push(e),
                        }
                    }
                    Some(_) => {
                        errors.push(Error::new_wline("you provided invalid token at index 1: expected instruction or nothing", 17, lnum));
                        continue;
                    }
                    None => {}
                }
            }
            Token::Mnemonic(m) => {
                unsafe { line.insert(0, Token::Mnemonic(m)) };
                match make_instruction(line, 0) {
                    Ok(mut i) => {
                        i.line = lnum;
                        body.push(BodyNode::Instruction(i))
                    }
                    Err(e) => errors.push(e),
                }
            }
            // assert that layout of line is: .section "name" and nothing past that
            Token::Keyword(Keyword::Section) => {
                inroot = false;
                let name = unsafe { line.take_owned(1) };
                let garbage = unsafe { line.take_owned(2) };
                if garbage.is_some() {
                    errors.push(Error::new_wline(
                        "you tried to make section, but you provided a token at index 2",
                        17,
                        lnum,
                    ));
                    continue;
                }
                let name = if let Some(Token::String(s)) = name {
                    s
                } else {
                    if name.is_none() {
                        errors.push(Error::new_wline(
                            "section directive requires name at index 1, found nothing",
                            17,
                            lnum,
                        ));
                    } else {
                        errors.push(Error::new_wline(
                            "you provided invalid token at index 1, expected string",
                            17,
                            lnum,
                        ));
                    }
                    continue;
                };
                body.push(BodyNode::Section(name));
            }
            // assert that layout of line is .align $align
            Token::Keyword(Keyword::Align) => {
                if inroot {
                    errors.push(Error::new_wline(
                        "you tried to use align directive inside of root; consider using external attributes #() on label or use it inside sections", 15, lnum));
                }
                let align = unsafe { line.take_owned(1) };
                // if is some, then error
                let invalid = unsafe { line.take_owned(2) };
                if invalid.is_some() {
                    errors.push(Error::new_wline(
                        "you tried to make bits, but you provided a token at index 2",
                        17,
                        lnum,
                    ));
                    continue;
                }

                let align = if let Some(Token::Immediate(n)) = align {
                    n.get_as_u64() as u16
                } else {
                    if align.is_none() {
                        errors.push(Error::new_wline(
                            "align directive requires 16-bit number at index 1, found nothing",
                            17,
                            lnum,
                        ));
                    } else {
                        errors.push(Error::new_wline(
                            "you provided invalid token at index 1, expected number",
                            17,
                            lnum,
                        ));
                    }
                    continue;
                };
                body.push(BodyNode::Align(align));
            }
            // assert that layout of line is .define name $value
            Token::Keyword(Keyword::Define) => {
                if inroot {
                    let name = unsafe { line.take_owned(1) };
                    let value = unsafe { line.take_owned(2) };
                    // if is some, then error
                    let invalid = unsafe { line.take_owned(3) };

                    if invalid.is_some() {
                        errors.push(Error::new_wline(
                            "you tried to make define, but you provided a token at index 3",
                            17,
                            lnum,
                        ));
                        continue;
                    }

                    let name = if let Some(Token::String(s)) = name {
                        s
                    } else {
                        if name.is_none() {
                            errors.push(Error::new_wline(
                                "define directive requires name at index 1, found nothing",
                                17,
                                lnum,
                            ));
                        } else {
                            errors.push(Error::new_wline(
                                "you provided invalid token at index 1, expected string",
                                17,
                                lnum,
                            ));
                        }
                        continue;
                    };
                    let value = if let Some(Token::Immediate(n)) = value {
                        n
                    } else {
                        if value.is_none() {
                            errors.push(Error::new_wline(
                                "define directive requires value at index 2, found nothing",
                                17,
                                lnum,
                            ));
                        } else {
                            errors.push(Error::new_wline(
                                "you provided invalid token at index 2, expected number",
                                17,
                                lnum,
                            ));
                        }
                        continue;
                    };
                    root.push(RootNode::Define(name, value));
                } else {
                    errors.push(Error::new_wline(
                        "you tried to use define directive outside of root",
                        15,
                        lnum,
                    ));
                }
            }
            // assert that layout of line is .include "file_path"
            Token::Keyword(Keyword::Include) => {
                if inroot {
                    let path = unsafe { line.take_owned(1) };
                    // if is some, then error
                    let invalid = unsafe { line.take_owned(2) };

                    if invalid.is_some() {
                        errors.push(Error::new_wline(
                            "you tried to make include, but you provided a token at index 2",
                            17,
                            lnum,
                        ));
                        continue;
                    }

                    let path = if let Some(Token::String(s)) = path {
                        s
                    } else {
                        if path.is_none() {
                            errors.push(Error::new_wline(
                                "include directive requires path at index 1, found nothing",
                                17,
                                lnum,
                            ));
                        } else {
                            errors.push(Error::new_wline(
                                "you provided invalid token at index 1, expected string",
                                17,
                                lnum,
                            ));
                        }
                        continue;
                    };
                    root.push(RootNode::Include(path));
                } else {
                    errors.push(Error::new_wline(
                        "you tried to use include directive outside of root",
                        15,
                        lnum,
                    ));
                }
            }
            // assert that layout of line is .bits $align
            Token::Keyword(Keyword::Bits) => {
                let bits = unsafe { line.take_owned(1) };
                // if is some, then error
                let invalid = unsafe { line.take_owned(2) };
                if invalid.is_some() {
                    errors.push(Error::new_wline(
                        "you tried to make bits, but you provided a token at index 2",
                        17,
                        lnum,
                    ));
                    continue;
                }

                let bits = if let Some(Token::Immediate(n)) = bits {
                    match n.get_as_u64() {
                        16 | 32 | 64 => n.get_as_u64() as u8,
                        _ => {
                            errors.push(Error::new_wline("bits directive accepts only numbers 16, 32 and 64; found none of these", 17, lnum));
                            continue;
                        }
                    }
                } else {
                    if bits.is_none() {
                        errors.push(Error::new_wline(
                            "bits directive requires number 16/32/64 at index 1, found nothing",
                            17,
                            lnum,
                        ));
                    } else {
                        errors.push(Error::new_wline(
                            "you provided invalid token at index 1, expected number",
                            17,
                            lnum,
                        ));
                    }
                    continue;
                };

                if inroot {
                    root.push(RootNode::Bits(bits));
                } else {
                    body.push(BodyNode::Bits(bits));
                }
            }
            // possible layout:
            // assert that layout of line is something like: .visibility .type "label_name": <instruction>
            Token::Keyword(k) => {
                let mut v = Visibility::default();
                let mut t = SymbolType::default();
                unsafe { line.insert(0, Token::Keyword(k)) };
                let mut lname = RString::from("");
                let mut i = 0;
                for token in line.iter() {
                    match token {
                        Token::Keyword(Keyword::Protected) => v = Visibility::Protected,
                        Token::Keyword(Keyword::Public) => v = Visibility::Public,
                        Token::Keyword(Keyword::Local) => v = Visibility::Local,
                        Token::Keyword(Keyword::Weak) => v = Visibility::Weak,
                        Token::Keyword(Keyword::Anonymous) => v = Visibility::Anonymous,
                        Token::Keyword(Keyword::Function) => t = SymbolType::Func,
                        Token::Keyword(Keyword::Object) => t = SymbolType::Object,
                        Token::Label(l) => {
                            lname = l.clone();
                            break;
                        }
                        Token::Error(e) => errors.push(*e.clone()),
                        _ => {
                            errors.push(Error::new_wline(
                                "tried to use invalid inline label attribute",
                                20,
                                lnum,
                            ));
                            break;
                        }
                    }
                    i += 1;
                }
                if lname.is_empty() {
                    errors.push(Error::new_wline(
                        "tried to use inline attributes, but you did not provide label name",
                        20,
                        lnum,
                    ));
                    continue;
                }

                if let Some(Token::Mnemonic(m)) = unsafe { line.take_owned(i + 1) } {
                    unsafe { line.insert(i + 1, Token::Mnemonic(m)) };
                    match make_instruction(line, i + 1) {
                        Ok(mut i) => {
                            i.line = lnum;
                            body.push(BodyNode::Instruction(i))
                        }
                        Err(e) => errors.push(e),
                    }
                }

                body.push(BodyNode::Label(Label {
                    name: lname,
                    debug_line: lnum,
                    attributes: {
                        let mut a = LabelAttributes::default();
                        a.set_symbol_type(t);
                        a.set_visibility(v);
                        a
                    },
                    ..Default::default()
                }));
            }

            _ => errors.push(Error::new_wline(
                "tried to start line with unknown token",
                3,
                lnum,
            )),
        }
    }

    if errors.is_empty() {
        Ok(MergerResult { root, body })
    } else {
        Err(errors)
    }
}

pub fn make_operand(mut operand_buf: SmallVec<Token, 4>) -> Result<Operand, Error> {
    unsafe {
        match operand_buf.len() {
            0 => Err(Error::new("cannot make operand from nothing", 3)),
            1 => Ok(Operand::try_from(operand_buf.take_owned(0).unwrap_unchecked())?),
            2 => match (operand_buf.take_owned_unchecked(0), operand_buf.take_owned_unchecked(1)) {
                (Token::Keyword(Keyword::Ref), Token::SymbolRef(s)) => {
                    let mut s = s.clone();
                    s.deref(false);
                    Ok(Operand::SymbolRef(*s.clone()))
                },
                (Token::Keyword(Keyword::Deref), Token::SymbolRef(_)) =>
                    Err(Error::new("you cannot create dereference to a symbol without providing size directive", 8)),
                (Token::Keyword(sz), Token::SymbolRef(mut s)) | (Token::SymbolRef(mut s), Token::Keyword(sz)) => {
                    let size = match Size::try_from(sz){
                        Ok(s) => s,
                        Err(_) => return Err(Error::new(
                            "expected size specifier, found unknown directive", 8)),
                    };
                    s.set_size(size);
                    s.deref(true);
                    Ok(Operand::SymbolRef(*s))
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
                    let mut sref = *sref.clone();
                    sref.deref(true);
                    sref.set_size(sz);
                    Ok(Operand::SymbolRef(sref))
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
                        Ok(Operand::Mem(m))
                    } else {
                        Err(Error::new("expected modifier of size directive and bcst keyword", 8))
                    }
                }
                (Token::Closure(' ', m), Token::Keyword(k))
                |(Token::Keyword(k), Token::Closure(' ', m)) =>
                    Ok(Operand::Mem(Mem::new(&m, Size::try_from(k).unwrap_or(Size::Unknown))?)),
                (Token::Modifier(m), Token::Keyword(k))
                |(Token::Keyword(k), Token::Modifier(m)) => {
                    let size = match Size::try_from(k){
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
                    Ok(Operand::Mem(mem))
                }
                _ => Err(Error::new("you tried to make operand from two tokens, but ones you provided could not be parsed into one", 8))
            }
            3 => {
                match (operand_buf.take_owned_unchecked(0), operand_buf.take_owned_unchecked(1), operand_buf.take_owned_unchecked(2)) {
                    (Token::Keyword(Keyword::Deref), Token::SymbolRef(mut s), Token::Keyword(sz)) |
                    (Token::Keyword(sz), Token::SymbolRef(mut s), Token::Keyword(Keyword::Deref)) => {
                        let sz = match Size::try_from(sz) {
                            Ok(s) => s,
                            Err(_) => return Err(Error::new("you tried to use symbol dereference, but you did not provide valid size directive", 8))
                        };
                        s.set_size(sz);
                        s.deref(true);
                        Ok(Operand::SymbolRef(*s))
                    }
                    _ => Err(Error::new("you tried to make operand from three tokens, but ones you provided could not be parsed into one", 8))
                }
            }
            _ => Err(Error::new("you tried to make operand from more than 3 tokens",8))
        }
    }
}

pub fn make_instruction(
    mut line: SmallVec<Token, SMALLVEC_TOKENS_LEN>,
    mut start_idx: usize,
) -> Result<Instruction, Error> {
    let mut mnemonics: SmallVec<Mnemonic, 2> = SmallVec::new();
    let mut operands: SmallVec<Operand, 5> = SmallVec::new();
    let mut subexpr: Vec<RString> = Vec::new();

    let mut operand_buf: SmallVec<Token, 4> = SmallVec::new();
    unsafe {
        while start_idx < line.len() {
            match line.take_owned(start_idx) {
                Some(Token::Mnemonic(m)) => {
                    if mnemonics.can_push() {
                        mnemonics.push(m);
                    } else {
                        return Err(Error::new(
                            "you tried to make instruction from too many mnemonics (2+)",
                            19,
                        ));
                    }
                }
                Some(Token::Comma) => {
                    if operands.can_push() {
                        if !operand_buf.is_empty() {
                            operands.push(make_operand(operand_buf)?);
                            operand_buf = SmallVec::new();
                        }
                    } else {
                        return Err(Error::new(
                            "you tried to make instruction with too many (5+) operands",
                            19,
                        ));
                    }
                }
                Some(Token::SubExpr(s)) => subexpr.push(s),
                Some(t) => {
                    if operand_buf.can_push() {
                        operand_buf.push(t);
                    } else {
                        return Err(Error::new(
                            "you tried to make operand from more than 4 tokens",
                            19,
                        ));
                    }
                }
                None => break,
            }
            start_idx += 1;
        }
    }
    if !operand_buf.is_empty() {
        if operands.can_push() {
            operands.push(make_operand(operand_buf)?);
        } else {
            return Err(Error::new(
                "you tried to make instruction with too many (5+) operands",
                19,
            ));
        }
    }

    let (mnem, addt) = unsafe {
        match mnemonics.len() {
            0 => {
                return Err(Error::new(
                    "you tried to make instructions with no mnemonics",
                    19,
                ))
            }
            1 => (mnemonics.pop().unwrap(), None),
            2 => (mnemonics.take_owned(1).unwrap(), mnemonics.take_owned(0)),
            _ => panic!("too many mnems"),
        }
    };
    let mut ins = Instruction {
        operands,
        mnem,
        addt,
        line: 0,
        meta: 0,
    };
    for s in subexpr {
        match &*s {
            "k0" => ins.set_mask(0b000),
            "k1" => ins.set_mask(0b001),
            "k2" => ins.set_mask(0b010),
            "k3" => ins.set_mask(0b011),
            "k4" => ins.set_mask(0b100),
            "k5" => ins.set_mask(0b101),
            "k6" => ins.set_mask(0b110),
            "k7" => ins.set_mask(0b111),
            "sae" => ins.set_sae(),
            "er" => ins.set_er(),
            "z" => ins.set_z(),
            "m" => {}
            "evex" => ins.set_evex(),
            "vex" => ins.set_vex(),
            _ => {
                return Err(Error::new(
                    format!("you tried to use unknown/unsupported subexpression: \"{s}\""),
                    18,
                ))
            }
        }
    }
    Ok(ins)
}
