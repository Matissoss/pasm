// pasm - src/core/mer.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    conf::{RString, SMALLVEC_TOKENS_LEN},
    pre::tok::Token,
    shr::{
        ast::{Instruction, Operand},
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

use crate::shr::location::Location;

#[derive(Debug)]
pub struct RootNode {
    pub location: Location,
    pub node: RootNodeEnum,
}

#[derive(Debug)]
pub enum RootNodeEnum {
    Format(RString),
    Include(RString),
    Extern(RString),
    Define(RString, Number),
    Bits(u8),
    Output(RString),
}

#[derive(Debug)]
pub struct BodyNode {
    pub location: Location,
    pub node: BodyNodeEnum,
}

#[derive(Debug)]
pub enum BodyNodeEnum {
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

#[derive(Debug)]
pub struct MergerResult {
    pub root: Vec<RootNode>,
    pub body: Vec<BodyNode>,
}

#[allow(unused_assignments)]
pub fn mer(lines: Vec<SmallVec<Token, SMALLVEC_TOKENS_LEN>>) -> Result<MergerResult, Vec<Error>> {
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
                body.push(BodyNode {
                    location: Location {
                        line: lnum,
                        char: 0,
                        ..Default::default()
                    },
                    node: BodyNodeEnum::Attributes(attr),
                });
            }
            // assert that layout of line is something like label: <instruction>
            Token::Label(lname) => {
                inroot = false;
                let l = Label {
                    name: lname,
                    debug_line: lnum,
                    ..Default::default()
                };
                body.push(BodyNode {
                    location: Location {
                        line: lnum,
                        char: 0,
                        ..Default::default()
                    },
                    node: BodyNodeEnum::Label(l),
                });
                match unsafe { line.take_owned(1) } {
                    Some(Token::Mnemonic(m)) => {
                        unsafe { line.insert(1, Token::Mnemonic(m)) };
                        match make_instruction(line, 0) {
                            Ok(mut i) => {
                                i.line = lnum;
                                body.push(BodyNode {
                                    location: Location {
                                        line: lnum,
                                        char: 0,
                                        ..Default::default()
                                    },
                                    node: BodyNodeEnum::Instruction(i),
                                });
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
                        body.push(BodyNode {
                            location: Location {
                                line: lnum,
                                char: 0,
                                ..Default::default()
                            },
                            node: BodyNodeEnum::Instruction(i),
                        });
                    }
                    Err(e) => errors.push(e),
                }
            }
            // assert that layout of line is: .extern "path" and nothing past that
            Token::Keyword(Keyword::Extern) => {
                if !inroot {
                    errors.push(Error::new_wline(
                        "you tried to use output directive outside of root",
                        15,
                        lnum,
                    ));
                }
                let name = unsafe { line.take_owned(1) };
                let garbage = unsafe { line.take_owned(2) };
                if garbage.is_some() {
                    errors.push(Error::new_wline(
                        "you tried to make output, but you provided a token at index 2",
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
                            "output directive requires name at index 1, found nothing",
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
                root.push(RootNode {
                    location: Location {
                        line: lnum,
                        char: 0,
                        ..Default::default()
                    },
                    node: RootNodeEnum::Extern(name),
                });
            }
            // assert that layout of line is: .output "path" and nothing past that
            Token::Keyword(Keyword::Output) => {
                if !inroot {
                    errors.push(Error::new_wline(
                        "you tried to use output directive outside of root",
                        15,
                        lnum,
                    ));
                }
                let name = unsafe { line.take_owned(1) };
                let garbage = unsafe { line.take_owned(2) };
                if garbage.is_some() {
                    errors.push(Error::new_wline(
                        "you tried to make output, but you provided a token at index 2",
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
                            "output directive requires name at index 1, found nothing",
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
                root.push(RootNode {
                    location: Location {
                        line: lnum,
                        char: 0,
                        ..Default::default()
                    },
                    node: RootNodeEnum::Output(name),
                });
            }
            // assert that layout of line is: .format "name" and nothing past that
            Token::Keyword(Keyword::Format) => {
                if !inroot {
                    errors.push(Error::new_wline(
                        "you tried to use format directive outside of root",
                        15,
                        lnum,
                    ));
                }
                let name = unsafe { line.take_owned(1) };
                let garbage = unsafe { line.take_owned(2) };
                if garbage.is_some() {
                    errors.push(Error::new_wline(
                        "you tried to make format, but you provided a token at index 2",
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
                            "format directive requires name at index 1, found nothing",
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
                root.push(RootNode {
                    location: Location {
                        line: lnum,
                        char: 0,
                        ..Default::default()
                    },
                    node: RootNodeEnum::Format(name),
                });
            }
            Token::Keyword(Keyword::Writeable) => body.push(BodyNode {
                location: Location {
                    line: lnum,
                    char: 0,
                    ..Default::default()
                },
                node: BodyNodeEnum::Write,
            }),
            Token::Keyword(Keyword::Executable) => body.push(BodyNode {
                location: Location {
                    line: lnum,
                    char: 0,
                    ..Default::default()
                },
                node: BodyNodeEnum::Exec,
            }),
            Token::Keyword(Keyword::Alloc) => body.push(BodyNode {
                location: Location {
                    line: lnum,
                    char: 0,
                    ..Default::default()
                },
                node: BodyNodeEnum::Alloc,
            }),
            // assert that layout of line is: .section "name" and nothing past that
            Token::Keyword(Keyword::Section) => {
                inroot = false;
                let name = unsafe { line.take_owned(1) };
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
                body.push(BodyNode {
                    location: Location {
                        line: lnum,
                        char: 0,
                        ..Default::default()
                    },
                    node: BodyNodeEnum::Section(name),
                });
                let mut slice_idx = 2;
                while let Some(t) = unsafe { line.take_owned(slice_idx) } {
                    match t {
                        Token::Keyword(Keyword::Executable) => body.push(BodyNode {
                            location: Location {
                                line: lnum,
                                char: 0,
                                ..Default::default()
                            },
                            node: BodyNodeEnum::Exec,
                        }),
                        Token::Keyword(Keyword::Writeable) => body.push(BodyNode {
                            location: Location {
                                line: lnum,
                                char: 0,
                                ..Default::default()
                            },
                            node: BodyNodeEnum::Write,
                        }),
                        Token::Keyword(Keyword::Alloc) => body.push(BodyNode {
                            location: Location {
                                line: lnum,
                                char: 0,
                                ..Default::default()
                            },
                            node: BodyNodeEnum::Alloc,
                        }),
                        _ => errors.push(Error::new_wline("expected writeable, alloc and executable directives after section directive, found other garbage", 17, lnum))
                    }

                    slice_idx += 1;
                }
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
                body.push(BodyNode {
                    location: Location {
                        line: lnum,
                        char: 0,
                        ..Default::default()
                    },
                    node: BodyNodeEnum::Align(align),
                });
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
                    root.push(RootNode {
                        location: Location {
                            line: lnum,
                            char: 0,
                            ..Default::default()
                        },
                        node: RootNodeEnum::Define(name, value),
                    });
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
                    root.push(RootNode {
                        location: Location {
                            line: lnum,
                            char: 0,
                            ..Default::default()
                        },
                        node: RootNodeEnum::Include(path),
                    });
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
                    root.push(RootNode {
                        location: Location {
                            line: lnum,
                            char: 0,
                            ..Default::default()
                        },
                        node: RootNodeEnum::Bits(bits),
                    });
                } else {
                    body.push(BodyNode {
                        location: Location {
                            line: lnum,
                            char: 0,
                            ..Default::default()
                        },
                        node: BodyNodeEnum::Bits(bits),
                    });
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
                body.push(BodyNode {
                    location: Location {
                        line: lnum,
                        char: 0,
                        ..Default::default()
                    },
                    node: BodyNodeEnum::Label(Label {
                        name: lname,
                        debug_line: lnum,
                        attributes: {
                            let mut a = LabelAttributes::default();
                            a.set_symbol_type(t);
                            a.set_visibility(v);
                            a
                        },
                        ..Default::default()
                    }),
                });

                if let Some(Token::Mnemonic(m)) = unsafe { line.take_owned(i + 1) } {
                    unsafe { line.insert(i + 1, Token::Mnemonic(m)) };
                    match make_instruction(line, i + 1) {
                        Ok(mut i) => {
                            i.line = lnum;
                            body.push(BodyNode {
                                location: Location {
                                    line: lnum,
                                    char: 0,
                                    ..Default::default()
                                },
                                node: BodyNodeEnum::Instruction(i),
                            });
                        }
                        Err(e) => errors.push(e),
                    }
                }
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
