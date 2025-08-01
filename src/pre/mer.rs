// pasm - src/core/mer.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use std::mem::ManuallyDrop;

use crate::{
    pre::tok::Token,
    shr::{
        ast::{Instruction, OperandOwned},
        dir::Directive,
        error::Error,
        ins::Mnemonic,
        label::{Label, LabelAttributes},
        mem::Mem,
        num::Number,
        section::Section,
        size::Size,
        smallvec::SmallVec,
        symbol::SymbolType,
        visibility::Visibility,
    },
};

#[derive(Debug)]
pub enum RootNode<'a> {
    Format(&'a str),
    Extern(&'a str),
    Define(&'a str, Number),
    Bits(u8),
    Output(&'a str),
}

#[derive(Debug)]
pub enum BodyNode<'a> {
    Section(Section<'a>),

    Label(Label<'a>),
    Instruction(Instruction<'a>),

    Attributes(&'a str),
    Bits(u8),
    Align(u16),
    // section attributes
    Alloc,
    Exec,
    Write,
    NoBits,
}

pub enum MergerToken<'a> {
    Body(BodyNode<'a>),
    Root(RootNode<'a>),
}

pub fn mer(mut line: SmallVec<Token, 16>, lnum: usize) -> Result<MergerToken, Error> {
    let start = unsafe { line.take_owned_unchecked(0) };

    match start {
        Token::Error(mut e) => {
            e.set_line(lnum);
            Err(*e)
        }

        // attributes
        Token::Closure('#', attr) => {
            let garbage = unsafe { line.take_owned(1) };
            if let Some(garbage) = garbage {
                if let Token::Closure('#', _) = garbage {
                    return Err(Error::new_wline(
                        "only one #() attribute closure is allowed per line",
                        17,
                        lnum,
                    ));
                } else {
                    return Err(Error::new_wline(
                        "found garbage after #() attribute closure",
                        17,
                        lnum,
                    ));
                }
            }
            Ok(MergerToken::Body(BodyNode::Attributes(attr)))
        }
        // assert that layout of line is something like <label_name>: [instruction]
        Token::Label(lname) => {
            let mut l = Label {
                name: lname,
                base_line: lnum,
                ..Default::default()
            };
            match unsafe { line.take_owned(1) } {
                Some(Token::Mnemonic(m)) => {
                    unsafe { line.insert(1, Token::Mnemonic(m)) };
                    match make_instruction(line, 1) {
                        Ok(mut i) => {
                            i.line = lnum;
                            l.content.push(i);
                        }
                        Err(mut e) => {
                            e.set_line(lnum);
                            return Err(e);
                        }
                    }
                }
                Some(_) => {
                    return Err(Error::new_wline(
                        "you provided invalid token at index 1: expected instruction or nothing",
                        17,
                        lnum,
                    ));
                }
                None => {}
            }
            Ok(MergerToken::Body(BodyNode::Label(l)))
        }
        Token::Mnemonic(m) => {
            unsafe { line.insert(0, Token::Mnemonic(m)) };
            match make_instruction(line, 0) {
                Ok(mut i) => {
                    i.line = lnum;
                    Ok(MergerToken::Body(BodyNode::Instruction(i)))
                }
                Err(mut e) => {
                    e.set_line(lnum);
                    Err(e)
                }
            }
        }
        // assert that layout of line is: extern <path><!>
        Token::Directive(Directive::Extern) => {
            let name = unsafe { line.take_owned(1) };
            let garbage = unsafe { line.take_owned(2) };
            if garbage.is_some() {
                return Err(Error::new_wline(
                    "you tried to make extern, but you provided a token at index 2",
                    17,
                    lnum,
                ));
            }
            let name = if let Some(Token::String(s)) = name {
                s
            } else if name.is_none() {
                return Err(Error::new_wline(
                    "extern directive requires name at index 1, found nothing",
                    17,
                    lnum,
                ));
            } else {
                return Err(Error::new_wline(
                    "you provided invalid token at index 1, expected string",
                    17,
                    lnum,
                ));
            };
            Ok(MergerToken::Root(RootNode::Extern(name)))
        }
        // assert that layout of line is: output <path><!>
        Token::Directive(Directive::Output) => {
            let name = unsafe { line.take_owned(1) };
            let garbage = unsafe { line.take_owned(2) };
            if garbage.is_some() {
                return Err(Error::new_wline(
                    "you tried to make output, but you provided a token at index 2",
                    17,
                    lnum,
                ));
            }
            let name = if let Some(Token::String(s)) = name {
                s
            } else if name.is_none() {
                return Err(Error::new_wline(
                    "output directive requires name at index 1, found nothing",
                    17,
                    lnum,
                ));
            } else {
                return Err(Error::new_wline(
                    "you provided invalid token at index 1, expected string",
                    17,
                    lnum,
                ));
            };
            Ok(MergerToken::Root(RootNode::Output(name)))
        }
        // assert that layout of line is: format <name><!>
        Token::Directive(Directive::Format) => {
            let name = unsafe { line.take_owned(1) };
            let garbage = unsafe { line.take_owned(2) };
            if garbage.is_some() {
                return Err(Error::new_wline(
                    "you tried to make format, but you provided a token at index 2",
                    17,
                    lnum,
                ));
            }
            let name = if let Some(Token::String(s)) = name {
                s
            } else if name.is_none() {
                return Err(Error::new_wline(
                    "format directive requires name at index 1, found nothing",
                    17,
                    lnum,
                ));
            } else {
                return Err(Error::new_wline(
                    "you provided invalid token at index 1, expected string",
                    17,
                    lnum,
                ));
            };
            Ok(MergerToken::Root(RootNode::Format(name)))
        }
        Token::Directive(Directive::Writeable) => Ok(MergerToken::Body(BodyNode::Write)),
        Token::Directive(Directive::Executable) => Ok(MergerToken::Body(BodyNode::Exec)),
        Token::Directive(Directive::Alloc) => Ok(MergerToken::Body(BodyNode::Alloc)),
        Token::Directive(Directive::NoBits) => Ok(MergerToken::Body(BodyNode::NoBits)),
        // assert that layout of line is: section <name> [attributes]<!>
        Token::Directive(Directive::Section) => {
            let name = unsafe { line.take_owned(1) };
            let name = if let Some(Token::String(s)) = name {
                s
            } else if name.is_none() {
                return Err(Error::new_wline(
                    "section directive requires name at index 1, found nothing",
                    17,
                    lnum,
                ));
            } else {
                return Err(Error::new_wline(
                    "you provided invalid token at index 1, expected string",
                    17,
                    lnum,
                ));
            };
            let mut section = Section {
                name,
                ..Default::default()
            };
            let mut slice_idx = 2;
            while let Some(t) = unsafe { line.take_owned(slice_idx) } {
                match t {
                        Token::Directive(Directive::Executable) => section.attributes.set_exec(true),
                        Token::Directive(Directive::Writeable) => section.attributes.set_write(true),
                        Token::Directive(Directive::Alloc) => section.attributes.set_alloc(true),
                        Token::Directive(Directive::NoBits) => section.attributes.set_nobits(true),
                        _ => return Err(Error::new_wline("expected writeable, nobits, alloc and executable directives after section directive, found other garbage", 17, lnum))
                    }

                slice_idx += 1;
            }
            Ok(MergerToken::Body(BodyNode::Section(section)))
        }
        // assert that layout of line is align <align>
        Token::Directive(Directive::Align) => {
            let align = unsafe { line.take_owned(1) };
            // if is some, then error
            let invalid = unsafe { line.take_owned(2) };
            if invalid.is_some() {
                return Err(Error::new_wline(
                    "you tried to make bits, but you provided a token at index 2",
                    17,
                    lnum,
                ));
            }

            let align = if let Some(Token::Immediate(n)) = align {
                n.get_as_u64() as u16
            } else if align.is_none() {
                return Err(Error::new_wline(
                    "align directive requires 16-bit number at index 1, found nothing",
                    17,
                    lnum,
                ));
            } else {
                return Err(Error::new_wline(
                    "you provided invalid token at index 1, expected number",
                    17,
                    lnum,
                ));
            };
            Ok(MergerToken::Body(BodyNode::Align(align)))
        }
        // assert that layout of line is define <name> <value>
        Token::Directive(Directive::Define) => {
            let name = unsafe { line.take_owned(1) };
            let value = unsafe { line.take_owned(2) };
            // if is some, then error
            let invalid = unsafe { line.take_owned(3) };

            if invalid.is_some() {
                return Err(Error::new_wline(
                    "you tried to make define, but you provided a token at index 3",
                    17,
                    lnum,
                ));
            }

            let name = if let Some(Token::String(s)) = name {
                s
            } else if name.is_none() {
                return Err(Error::new_wline(
                    "define directive requires name at index 1, found nothing",
                    17,
                    lnum,
                ));
            } else {
                return Err(Error::new_wline(
                    "you provided invalid token at index 1, expected string",
                    17,
                    lnum,
                ));
            };
            let value = if let Some(Token::Immediate(n)) = value {
                n
            } else if value.is_none() {
                return Err(Error::new_wline(
                    "define directive requires value at index 2, found nothing",
                    17,
                    lnum,
                ));
            } else {
                return Err(Error::new_wline(
                    "you provided invalid token at index 2, expected number",
                    17,
                    lnum,
                ));
            };
            Ok(MergerToken::Root(RootNode::Define(name, value)))
        }
        // assert that layout of line is bits <16/32/64>
        Token::Directive(Directive::Bits) => {
            let bits = unsafe { line.take_owned(1) };
            // if is some, then error
            let invalid = unsafe { line.take_owned(2) };
            if invalid.is_some() {
                return Err(Error::new_wline(
                    "you tried to make bits, but you provided a token at index 2",
                    17,
                    lnum,
                ));
            }

            let bits = if let Some(Token::Immediate(n)) = bits {
                match n.get_as_u64() {
                    16 | 32 | 64 => n.get_as_u64() as u8,
                    _ => {
                        return Err(Error::new_wline("bits directive accepts only numbers 16, 32 and 64; found none of these", 17, lnum));
                    }
                }
            } else if bits.is_none() {
                return Err(Error::new_wline(
                    "bits directive requires number 16/32/64 at index 1, found nothing",
                    17,
                    lnum,
                ));
            } else {
                return Err(Error::new_wline(
                    "you provided invalid token at index 1, expected number",
                    17,
                    lnum,
                ));
            };

            Ok(MergerToken::Root(RootNode::Bits(bits)))
        }
        // possible layout:
        // assert that layout of line is something like: [label_attributes] <label_name>: [instruction]
        Token::Directive(k) => {
            let mut v = Visibility::default();
            let mut t = SymbolType::default();
            unsafe { line.insert(0, Token::Directive(k)) };
            let mut lname = "";
            let mut i = 0;
            for token in line.iter() {
                match token {
                    Token::Directive(Directive::Protected) => v = Visibility::Protected,
                    Token::Directive(Directive::Public) => v = Visibility::Public,
                    Token::Directive(Directive::Local) => v = Visibility::Local,
                    Token::Directive(Directive::Weak) => v = Visibility::Weak,
                    Token::Directive(Directive::Anonymous) => v = Visibility::Anonymous,
                    Token::Directive(Directive::Function) => t = SymbolType::Func,
                    Token::Directive(Directive::Object) => t = SymbolType::Object,
                    Token::Label(l) => {
                        lname = l;
                        break;
                    }
                    Token::Error(e) => return Err(*e.clone()),
                    _ => {
                        return Err(Error::new_wline(
                            "expected label declaration, found unknown directive",
                            20,
                            lnum,
                        ));
                    }
                }
                i += 1;
            }
            if lname.is_empty() {
                return Err(Error::new_wline(
                    "tried to use inline attributes, but you did not provide label name",
                    20,
                    lnum,
                ));
            }
            let mut label = Label {
                name: lname,
                base_line: lnum,
                attributes: {
                    let mut a = LabelAttributes::default();
                    a.set_symbol_type(t);
                    a.set_visibility(v);
                    a
                },
                ..Default::default()
            };

            if let Some(Token::Mnemonic(m)) = unsafe { line.take_owned(i + 1) } {
                unsafe { line.insert(i + 1, Token::Mnemonic(m)) };
                match make_instruction(line, i + 1) {
                    Ok(mut i) => {
                        i.line = lnum;
                        label.content.push(i);
                    }
                    Err(mut e) => {
                        e.set_line(lnum);
                        return Err(e);
                    }
                }
            }
            Ok(MergerToken::Body(BodyNode::Label(label)))
        }
        Token::SubExpr(s) => {
            unsafe { line.insert(0, Token::SubExpr(s)) };
            match make_instruction(line, 0) {
                Ok(mut i) => {
                    i.line = lnum;
                    Ok(MergerToken::Body(BodyNode::Instruction(i)))
                }
                Err(mut e) => {
                    e.set_line(lnum);
                    Err(e)
                }
            }
        }

        _ => Err(Error::new_wline(
            "tried to start line with unknown token",
            3,
            lnum,
        )),
    }
}

pub fn make_operand(mut operand_buf: SmallVec<Token, 2>) -> Result<OperandOwned, Error> {
    unsafe {
        match operand_buf.len() {
            0 => Err(Error::new("cannot make operand from nothing", 3)),
            1 => Ok(OperandOwned::try_from(operand_buf.take_owned(0).unwrap_unchecked())?),
            2 => match (operand_buf.take_owned_unchecked(0), operand_buf.take_owned_unchecked(1)) {
                (Token::Directive(sz), Token::SymbolRef(mut s)) | (Token::SymbolRef(mut s), Token::Directive(sz)) => {
                    let size = match Size::try_from(sz){
                        Ok(s) => s,
                        Err(_) => return Err(Error::new(
                            "expected size specifier, found unknown directive", 8)),
                    };
                    s.set_size(size);
                    s.deref(true);
                    Ok(OperandOwned::Symbol(ManuallyDrop::new(s)))
                }
                (Token::Closure(' ', m), Token::Modifier(mods)) |
                (Token::Modifier(mods), Token::Closure(' ', m)) => {
                    if mods.len() == 2 {
                        let sz = if let Token::Directive(modk) = mods[0] {
                            match Size::try_from(modk){
                                Ok(s) => s,
                                Err(_) => return Err(Error::new(
                                    "expected size specifier, found unknown directive", 8)),
                            }
                        } else {
                            return Err(Error::new("expected size directive at index 0 of modifier", 8));
                        };
                        if let Token::Directive(Directive::Bcst) = mods[1] {} else {
                            return Err(Error::new("expected bcst directive at index 1 of modifier", 8));
                        }
                        let mut m = Mem::new(m, sz)?;
                        m.set_bcst(true);
                        Ok(OperandOwned::Mem(m))
                    } else {
                        Err(Error::new("expected modifier of size directive and bcst keyword", 8))
                    }
                }
                (Token::Closure(' ', m), Token::Directive(k))
                |(Token::Directive(k), Token::Closure(' ', m)) =>
                    Ok(OperandOwned::Mem(Mem::new(m, Size::try_from(k).unwrap_or(Size::Unknown))?)),
                (Token::Modifier(m), Token::Directive(k))
                |(Token::Directive(k), Token::Modifier(m)) => {
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
                    Ok(OperandOwned::Mem(mem))
                }
                _ => {
                    Err(Error::new("you tried to make operand from two tokens, but ones you provided could not be parsed into one", 8))
                }
            }
            _ => Err(Error::new("you tried to make operand from more than 2 tokens",8))
        }
    }
}

pub fn make_instruction(
    mut line: SmallVec<Token, 16>,
    mut start_idx: usize,
) -> Result<Instruction, Error> {
    let mut ins = Instruction::default();
    let mut mnemonics: SmallVec<Mnemonic, 2> = SmallVec::new();
    let mut subexpr: Vec<&str> = Vec::new();

    let mut operand_buf: SmallVec<Token, 2> = SmallVec::new();
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
                    if ins.len() < 4 {
                        if !operand_buf.is_empty() {
                            ins.push(make_operand(operand_buf)?);
                            operand_buf = SmallVec::new();
                        }
                    } else {
                        return Err(Error::new(
                            "you tried to make instruction with too many (4+) operands",
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
        if ins.len() < 4 {
            ins.push(make_operand(operand_buf)?);
        } else {
            return Err(Error::new(
                "you tried to make instruction with too many (4+) operands",
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
            1 => (mnemonics.take_owned_unchecked(0), None),
            2 => (mnemonics.take_owned_unchecked(1), mnemonics.take_owned(0)),
            _ => panic!("too many mnems"),
        }
    };

    ins.mnemonic = mnem;
    if let Some(addt) = addt {
        ins.set_addt(addt);
    }
    for s in subexpr {
        match s {
            // APX
            "apx" => ins.apx_set_default(),
            "apx-evex" => ins.apx_evex_set_apx_extension(true),
            "of" => ins.apx_eevex_cond_set_of(),
            "cf" => ins.apx_eevex_cond_set_cf(),
            "zf" => ins.apx_eevex_cond_set_zf(),
            "sf" => ins.apx_eevex_cond_set_sf(),
            "rex2" => ins.apx_set_rex2(),
            "eevex" => ins.apx_set_eevex(),
            "nf" => ins.apx_set_leg_nf(),
            "vex-nf" => ins.apx_set_vex_nf(),

            "k0" => ins.set_evex_mask(0b000),
            "k1" => ins.set_evex_mask(0b001),
            "k2" => ins.set_evex_mask(0b010),
            "k3" => ins.set_evex_mask(0b011),
            "k4" => ins.set_evex_mask(0b100),
            "k5" => ins.set_evex_mask(0b101),
            "k6" => ins.set_evex_mask(0b110),
            "k7" => ins.set_evex_mask(0b111),
            "sae" => ins.set_evex_sae(),
            "er" => ins.set_evex_er(0b001),
            "rn-sae" => ins.set_evex_er(0b001),
            "rd-sae" => ins.set_evex_er(0b010),
            "ru-sae" => ins.set_evex_er(0b011),
            "rz-sae" => ins.set_evex_er(0b100),
            "z" => ins.set_evex_z(),
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
