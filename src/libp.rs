// pasm - src/libp.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};

use crate::{
    core::{api::AssembleResult, comp},
    obj::Elf,
    pre::{
        chk,
        par::{par, LineResult},
    },
    shr::{
        error::Error as PasmError,
        reloc::{relocate_addresses, RelType, Relocation},
        section::{Section, SectionAttributes},
        symbol::{Symbol, SymbolType},
        visibility::Visibility,
    },
    utils::LineIter,
};

pub fn assemble(ipath: &Path, opath: &Path) -> Result<(), PasmError> {
    // fetch input file
    let ifile = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(ipath);
    let mut ifile = match ifile {
        Ok(f) => f,
        Err(e) => {
            return Err(PasmError::new(e.to_string(), 6));
        }
    };
    let mut ibuf = Vec::new();
    if let Err(e) = ifile.read_to_end(&mut ibuf) {
        return Err(PasmError::new(e.to_string(), 6));
    }

    // get output from assembler
    let mut obuf: Vec<u8> = Vec::new();
    let mut rels: Vec<Relocation> = Vec::new();
    let mut symbols: Vec<Symbol> = Vec::new();
    let mut line_iter = LineIter::new(&ibuf);

    let mut sections: Vec<Section> = Vec::new();
    let mut current_section = Section {
        name: ".text",
        size: 0,
        offset: 0,
        align: 0,
        attributes: SectionAttributes::new(),
        bits: 16,
    };
    let mut current_label = 0usize;
    let mut sindex: u16 = 0u16;

    let mut target: Option<&str> = None;
    let mut bits: u8 = 16u8;

    while let Some((lnum, line)) = line_iter.next() {
        let line = line.trim();
        match par(line) {
            LineResult::Error(mut e) => {
                e.set_line(lnum + 1);
                return Err(e);
            }
            LineResult::Instruction(mut i) => {
                let e = if bits == 64 {
                    chk::check_ins64bit(&i)
                } else {
                    chk::check_ins32bit(&i)
                };
                if let Err(mut e) = e {
                    e.set_line(lnum + 1);
                    return Err(e);
                }
                // i hate Rust's borrow checker sometimes tbh
                let ins_ptr = std::ptr::from_mut(&mut i);
                std::mem::forget(i);
                let (res, mut rel_a) = comp::get_genapi(unsafe { &*ins_ptr }, bits).assemble(
                    unsafe { &*ins_ptr },
                    bits,
                    RelType::REL32,
                );
                for r in rel_a.iter_mut() {
                    r.offset += obuf.len();
                }
                match res {
                    AssembleResult::WLargeImm(d) => obuf.extend(d.into_iter()),
                    AssembleResult::NoLargeImm(d) => obuf.extend(d.iter()),
                }
                rels.extend(rel_a.into_iter());
                unsafe {
                    std::ptr::drop_in_place(ins_ptr);
                }
            }
            LineResult::Section(s) => {
                for sym in &symbols {
                    if sym.name == s {
                        if sym.stype == SymbolType::Section {
                            return Err(PasmError::new(
                                format!("there is already a section with name {s}"),
                                8,
                            ));
                        } else {
                            return Err(PasmError::new(
                                format!("there is already a symbol with name {s}"),
                                8,
                            ));
                        }
                    }
                }
                current_section.size = obuf.len() - current_section.offset;
                if sindex != 0 {
                    symbols.push(Symbol {
                        name: current_section.name,
                        offset: current_section.offset,
                        size: current_section.size,
                        sindex,
                        visibility: Visibility::Local,
                        stype: SymbolType::Section,
                        valid: true,
                    });
                    sindex += 1;
                }
                current_section = Section {
                    name: s,
                    size: 0,
                    offset: obuf.len(),
                    align: 0,
                    attributes: SectionAttributes::new(),
                    bits,
                };
            }

            LineResult::Label(l) => {
                let mut found = None;
                for (i, s) in symbols.iter().enumerate() {
                    if s.name == l {
                        if !s.valid {
                            found = Some(i);
                        } else {
                            return Err(PasmError::new(
                                format!("symbol \"{l}\" is redeclared twice or more in this file"),
                                8,
                            ));
                        }
                        break;
                    }
                }
                if let Some(i) = found {
                    symbols[i].valid = true;
                    symbols[current_label].size = obuf.len() - symbols[current_label].offset;
                    current_label = i;
                } else {
                    symbols.push(Symbol {
                        name: l,
                        offset: obuf.len(),
                        size: 0,
                        sindex,
                        visibility: Visibility::Local,
                        stype: SymbolType::NoType,
                        valid: true,
                    });
                    symbols[current_label].size = obuf.len() - symbols[current_label].offset;
                    current_label = symbols.len() - 1;
                }
            }
            LineResult::Directive("target", t) => target = Some(t),
            LineResult::Directive("public", t) => {
                let mut found = false;
                for s in &mut symbols {
                    if s.name == t {
                        s.visibility = Visibility::Public;
                        found = true;
                        break;
                    }
                }
                if !found {
                    symbols.push(Symbol {
                        name: t,
                        stype: SymbolType::NoType,
                        size: 0,
                        offset: 0,
                        sindex: 0,
                        visibility: Visibility::Public,
                        valid: false,
                    })
                }
            }
            LineResult::Directive("private", t) => {
                let mut found = false;
                for s in &mut symbols {
                    if s.name == t {
                        s.visibility = Visibility::Local;
                        found = true;
                        break;
                    }
                }
                if !found {
                    symbols.push(Symbol {
                        name: t,
                        stype: SymbolType::NoType,
                        size: 0,
                        offset: 0,
                        sindex: 0,
                        visibility: Visibility::Local,
                        valid: false,
                    })
                }
            }
            LineResult::Directive("weak", t) => {
                let mut found = false;
                for s in &mut symbols {
                    if s.name == t {
                        s.visibility = Visibility::Weak;
                        found = true;
                        break;
                    }
                }
                if !found {
                    symbols.push(Symbol {
                        name: t,
                        stype: SymbolType::NoType,
                        size: 0,
                        offset: 0,
                        sindex: 0,
                        visibility: Visibility::Weak,
                        valid: false,
                    })
                }
            }
            LineResult::Directive("protected", t) => {
                let mut found = false;
                for s in &mut symbols {
                    if s.name == t {
                        s.visibility = Visibility::Protected;
                        found = true;
                        break;
                    }
                }
                if !found {
                    symbols.push(Symbol {
                        name: t,
                        stype: SymbolType::NoType,
                        size: 0,
                        offset: 0,
                        sindex: 0,
                        visibility: Visibility::Protected,
                        valid: false,
                    })
                }
            }
            LineResult::Directive("function", t) => {
                let mut found = false;
                for s in &mut symbols {
                    if s.name == t {
                        s.stype = SymbolType::Func;
                        found = true;
                        break;
                    }
                }
                if !found {
                    symbols.push(Symbol {
                        name: t,
                        stype: SymbolType::Func,
                        size: 0,
                        offset: 0,
                        sindex: 0,
                        visibility: Visibility::Local,
                        valid: false,
                    })
                }
            }
            LineResult::Directive("extern", t) => symbols.push(Symbol {
                name: t,
                offset: 0,
                size: 0,
                sindex: 0,
                visibility: Visibility::Extern,
                stype: SymbolType::NoType,
                valid: true,
            }),
            LineResult::Directive("object", t) => {
                let mut found = false;
                for s in &mut symbols {
                    if s.name == t {
                        s.stype = SymbolType::Object;
                        found = true;
                        break;
                    }
                }
                if !found {
                    symbols.push(Symbol {
                        name: t,
                        stype: SymbolType::Object,
                        size: 0,
                        offset: 0,
                        sindex: 0,
                        visibility: Visibility::Local,
                        valid: false,
                    })
                }
            }
            LineResult::Directive("bits", b) => {
                if let Ok(b) = b.parse::<u8>() {
                    if b == 16 || b == 32 || b == 64 {
                        bits = b;
                    }
                }
            }
            LineResult::Directive("nobits", _) => current_section.attributes.set_nobits(true),
            LineResult::Directive("writeable", _) => current_section.attributes.set_write(true),
            LineResult::Directive("align", c) => {
                if let Ok(c) = c.parse::<u16>() {
                    current_section.align = c;
                }
            }
            LineResult::Directive("alloc", _) => current_section.attributes.set_alloc(true),
            _ => {}
        }
    }
    current_section.size = obuf.len() - current_section.offset;
    symbols.push(Symbol {
        name: current_section.name,
        offset: current_section.offset,
        size: current_section.size,
        sindex,
        visibility: Visibility::Local,
        stype: SymbolType::Section,
        valid: true,
    });
    sections.push(current_section);

    for s in &symbols {
        if !s.valid {
            return Err(PasmError::new(
                format!(
                    "you tried to use directive on invalid symbol named \"{}\"",
                    s.name
                ),
                8,
            ));
        }
    }

    match target.unwrap_or("bin") {
        #[cfg(feature = "target_elf")]
        "elf64" | "ELF64" => {
            let elf = Elf::new(&sections, opath, &obuf, rels, &symbols, true)?;
            obuf = elf.compile(true);
        }
        #[cfg(feature = "target_elf")]
        "elf32" | "ELF32" => {
            let elf = Elf::new(&sections, opath, &obuf, rels, &symbols, false)?;
            obuf = elf.compile(false);
        }
        "bin" => {
            relocate_addresses(&mut obuf, rels, &symbols)?;
        }
        t => return Err(PasmError::new(format!("unknown target {t}"), 7)),
    }

    // now write content to a file
    let ofile = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(opath);
    let mut ofile = match ofile {
        Ok(f) => f,
        Err(e) => {
            return Err(PasmError::new(e.to_string(), 6));
        }
    };
    if let Err(err) = ofile.write_all(&obuf) {
        return Err(PasmError::new(err.to_string(), 6));
    }
    Ok(())
}
