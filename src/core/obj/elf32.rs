// rasmx86_64 - src/core/obj/elf32.rs
// ----------------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;

use crate::shr::{
    reloc::Relocation,
    symbol::{Symbol, SymbolType as SType, Visibility},
};
use std::path::Path;

type Elf32Addr = u32;
type Elf32Half = u16;
type Elf32Off = u32;
type Elf32Sword = i32;
type Elf32Word = u32;
type UnsignedChar = u8;

const EI_NIDENT: usize = 16;
const SHF_ALLOC: Elf32Word = 0x2;
const SHF_EXECINSTR: Elf32Word = 0x4;

// elf header
struct Elf32Ehdr {
    ident: [UnsignedChar; EI_NIDENT],
    etype: Elf32Half,
    machine: Elf32Half,
    version: Elf32Word,
    entry: Elf32Addr,
    phoff: Elf32Off,
    shoff: Elf32Off,
    flags: Elf32Word,
    ehsize: Elf32Half,
    phentsize: Elf32Half,
    phnum: Elf32Half,
    shentsize: Elf32Half,
    shnum: Elf32Half,
    shstrndx: Elf32Half,
}

// section header
#[derive(Debug, Clone, Copy)]
struct Elf32Shdr {
    name: Elf32Word,
    etype: Elf32Word,
    flags: Elf32Word,
    addr: Elf32Addr,
    offset: Elf32Off,
    size: Elf32Word,
    link: Elf32Word,
    info: Elf32Word,
    addralign: Elf32Word,
    entsize: Elf32Word,
}

#[derive(Debug, Clone, Copy)]
struct Elf32Sym {
    name: Elf32Word,
    value: Elf32Addr,
    size: Elf32Word,
    info: u8,
    other: u8,
    shndx: Elf32Half,
}

#[derive(Debug)]
struct Elf32Rel {
    offset: Elf32Addr,
    info: Elf32Word,
}

struct Elf32Rela {
    offset: Elf32Addr,
    info: Elf32Word,
    addend: Elf32Sword,
}

pub fn make_elf32(
    code: &[u8],
    relocs: Vec<Relocation>,
    symbols: &[Symbol],
    outpath: &Path,
) -> Vec<u8> {
    // elf class
    // 1 - 32-bit object
    // 2 - 64-bit object
    let ei_class = 1;
    // data encoding
    // 1 - little endian
    // 2 - big endian
    let ei_data = 1;
    // elf version
    // 1 - current
    let ei_version = 1;
    // os abi
    // 0 - System V
    let ei_osabi = 0;
    // os abi version
    let ei_osabiversion = 0;
    // elf type
    // 1 - relocatable (.o)
    // 2 - executable
    // 3 - shared object
    let etype = 1;
    // machine
    // 3 - i386 (x86?)
    // 62 - x86_64
    let machine = 3;
    let mut elf_header = Elf32Ehdr {
        ident: [
            0x7F,
            b'E',
            b'L',
            b'F',
            ei_class,
            ei_data,
            ei_version,
            ei_osabi,
            ei_osabiversion,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        etype,
        machine,
        version: 1,
        entry: 0,                                            // not used
        phoff: 0,                                            // not used
        shoff: (size_of::<Elf32Ehdr>() + code.len()) as u32, // section header offset
        flags: 0,                                            // not used
        ehsize: size_of::<Elf32Ehdr>() as u16,               // elf header size
        phentsize: 0,                                        // not used
        phnum: 0,                                            // not used
        shentsize: size_of::<Elf32Shdr>() as u16,            // section header size
        shnum: 0,                                            // section header entries
        shstrndx: 3, // section header string table (.shstrtab) index
    };
    let mut shstrtab: Vec<UnsignedChar> = vec![0];

    // =====================
    //  .rel.* and .rela.* (for now only .rel.* and only for .text section)
    // =====================

    let (mut rel_text_shdr, mut rela_text_shdr) = (None, None);
    let (mut rel_text_symb, mut rela_text_symb) = (Vec::new(), Vec::new());
    let (mut rel_text_symbref, mut rela_text_symbref) = (Vec::new(), Vec::new());

    if !relocs.is_empty() {
        let mut relas = Vec::new();
        let mut rels = Vec::new();
        #[allow(clippy::needless_range_loop)]
        for index in 0..relocs.len() {
            if relocs[index].addend == 0 {
                rel_text_symbref.push(Cow::Owned::<&String>(&relocs[index].symbol));
                rels.push(Cow::Borrowed(&relocs[index]));
            } else {
                rela_text_symbref.push(Cow::Borrowed(&relocs[index].symbol));
                relas.push(Cow::Borrowed(&relocs[index]));
            }
        }
        if !rels.is_empty() {
            elf_header.shnum += 1;
            rel_text_shdr = Some(Elf32Shdr {
                name: shstrtab.len() as u32,
                etype: 9,
                info: 1,
                link: 0,
                flags: 0,
                addr: 0,
                addralign: 0,
                offset: 0,
                size: (size_of::<Elf32Rel>() * rels.len()) as u32,
                entsize: size_of::<Elf32Rel>() as u32,
            });
            shstrtab.extend(b".rel.text\0");
        }
        if !relas.is_empty() {
            elf_header.shnum += 1;
            rela_text_shdr = Some(Elf32Shdr {
                name: shstrtab.len() as u32,
                etype: 4,
                info: 1,
                link: 0,
                flags: 0,
                addr: 0,
                addralign: 0,
                offset: 0,
                size: (size_of::<Elf32Rela>() * relas.len()) as u32,
                entsize: size_of::<Elf32Rela>() as u32,
            });
            shstrtab.extend(b".rela.text\0");
        }
        for rel in rels {
            rel_text_symb.push(Elf32Rel {
                offset: rel.offset as u32,
                info: rel.rtype.to_elf32_rtype(),
            });
        }
        for rela in relas {
            rela_text_symb.push(Elf32Rela {
                offset: rela.offset as u32,
                info: rela.rtype.to_elf32_rtype(),
                addend: rela.addend,
            });
        }
    }

    // =====================
    //  Elf section headers
    // =====================

    let null_shdr = Elf32Shdr {
        name: 0,
        etype: 0,
        flags: 0,
        addr: 0,
        offset: 0,
        size: 0,
        link: 0,
        addralign: 0,
        entsize: 0,
        info: 0,
    };
    elf_header.shnum += 1;

    let text_shdr = Elf32Shdr {
        name: shstrtab.len() as u32,
        etype: 1,
        flags: SHF_EXECINSTR | SHF_ALLOC,
        addr: 0,
        offset: size_of::<Elf32Ehdr>() as u32,
        size: code.len() as u32,
        link: 0,
        info: 0,
        addralign: 16,
        entsize: 0,
    };
    elf_header.shnum += 1;
    shstrtab.extend(b".text\0");

    let mut symtab_hdr = Elf32Shdr {
        name: shstrtab.len() as u32,
        etype: 2,
        flags: 0,
        offset: 0, // to set - symbols
        addr: 0,
        size: 0,
        link: 4,
        info: 0,
        addralign: 0,
        entsize: size_of::<Elf32Sym>() as u32,
    };
    elf_header.shnum += 1;
    shstrtab.extend(b".symtab\0");

    let mut shstrtab_hdr = Elf32Shdr {
        name: shstrtab.len() as u32,
        etype: 3,
        flags: 0,
        addr: 0,
        offset: 0, // to set - .shstrtab section
        size: 0,
        link: 0,
        info: 0,
        addralign: 1,
        entsize: 0,
    };
    elf_header.shnum += 1;
    shstrtab.extend(b".shstrtab\0");

    // string table, but for symbols
    let mut strtab_hdr = Elf32Shdr {
        name: shstrtab.len() as u32,
        etype: 3,
        flags: 0,
        addr: 0,
        size: 0,   // to set - strtab size
        offset: 0, // to set - strtab offset
        addralign: 0,
        link: 0,
        info: 0,
        entsize: 0,
    };
    elf_header.shnum += 1;
    shstrtab.extend(b".strtab\0");

    // symbols
    let mut strtab: Vec<UnsignedChar> = vec![0];
    let symbols: Vec<Elf32Sym> = {
        let mut elf_symbs = Vec::new();
        // null symbol
        elf_symbs.push(Elf32Sym {
            name: 0,
            value: 0,
            size: 0,
            info: 0,
            other: 0,
            shndx: 0,
        });
        // file symbol
        elf_symbs.push(Elf32Sym {
            name: strtab.len() as u32,
            value: 0,
            size: 0,
            info: SType::File as u8,
            other: 0,
            shndx: 0xFFF1,
        });
        strtab.extend(outpath.to_string_lossy().as_bytes());
        strtab.extend(b"\0");

        // text section symbol
        elf_symbs.push(Elf32Sym {
            name: strtab.len() as u32,
            value: 0,
            size: 0,
            info: SType::Section as u8,
            other: 0,
            shndx: 1,
        });
        strtab.extend(b".text\0");

        let mut global_symbols = Vec::new();

        for symbol in symbols.iter() {
            if symbol.visibility == Visibility::Global {
                global_symbols.push(symbol);
            } else {
                elf_symbs.push(Elf32Sym {
                    name: strtab.len() as u32,
                    value: symbol.offset as u32,
                    size: symbol.size.unwrap_or(0),
                    info: ((symbol.visibility as u8) << 4) + symbol.stype as u8,
                    other: 0,
                    shndx: symbol.sindex,
                });
                strtab.extend(symbol.name.as_bytes());
                strtab.extend(b"\0");
            }
        }
        symtab_hdr.info = elf_symbs.len() as u32;

        for symbol in global_symbols.iter() {
            elf_symbs.push(Elf32Sym {
                name: strtab.len() as u32,
                value: symbol.offset as u32,
                size: symbol.size.unwrap_or(0),
                info: ((symbol.visibility as u8) << 4) + symbol.stype as u8,
                other: 0,
                shndx: symbol.sindex,
            });
            strtab.extend(symbol.name.as_bytes());
            strtab.extend(b"\0");
        }
        elf_symbs
    };

    // offset to end of shdr's and start of symbols (.symtab), .shstrtab and .strtab
    let symbols_offset = (size_of::<Elf32Ehdr>()
        + code.len()
        + (elf_header.shnum as usize * size_of::<Elf32Shdr>())) as u32;

    let mut bytes = Vec::new();

    let modf = 0;

    elf_header.shstrndx += modf;
    bytes.extend(elf_header.bytes());
    bytes.extend(code);
    bytes.extend(null_shdr.bytes());
    bytes.extend(text_shdr.bytes());

    symtab_hdr.offset = symbols_offset;
    symtab_hdr.size = (symbols.len() * size_of::<Elf32Sym>()) as u32;
    symtab_hdr.link += modf as u32;
    bytes.extend(symtab_hdr.bytes());

    // .shstrtab section header
    shstrtab_hdr.offset = symbols_offset + (symbols.len() * size_of::<Elf32Sym>()) as u32;
    shstrtab_hdr.size = shstrtab.len() as u32;
    bytes.extend(shstrtab_hdr.bytes());

    // .strtab section header
    strtab_hdr.offset =
        symbols_offset + (symbols.len() * size_of::<Elf32Sym>()) as u32 + shstrtab.len() as u32;
    strtab_hdr.size = strtab.len() as u32;
    bytes.extend(strtab_hdr.bytes());

    if let Some(mut shdr) = rel_text_shdr {
        shdr.link = (modf + 2) as u32;
        shdr.offset = symbols_offset
            + (symbols.len() * size_of::<Elf32Sym>()) as u32
            + shstrtab.len() as u32
            + strtab.len() as u32;
        bytes.extend(shdr.bytes());
    }
    if let Some(mut shdr) = rela_text_shdr {
        shdr.link = (modf + 2) as u32;
        // what is this
        shdr.offset = symbols_offset
            + (symbols.len() * size_of::<Elf32Sym>()) as u32
            + shstrtab.len() as u32
            + strtab.len() as u32
            + (rel_text_symb.len() * size_of::<Elf32Rel>()) as u32;
        bytes.extend(shdr.bytes());
    }

    // .symtab
    for s in &symbols {
        bytes.extend(s.bytes());
    }

    bytes.extend(&shstrtab);
    bytes.extend(&strtab);

    if rel_text_shdr.is_some() {
        for (index, rel) in rel_text_symb.iter_mut().enumerate() {
            for (symb_index, s) in symbols.iter().enumerate() {
                let s_name = collect_asciiz(&strtab, s.name as usize).unwrap();
                // what is that?
                if ***rel_text_symbref[index] == *Cow::Borrowed(&s_name) {
                    rel.info += (symb_index << 8) as u32;
                    break;
                }
            }
            bytes.extend(rel.bytes());
        }
    }
    if rela_text_shdr.is_some() {
        for (index, rela) in rela_text_symb.iter_mut().enumerate() {
            for (symb_index, s) in symbols.iter().enumerate() {
                let s_name = collect_asciiz(&strtab, s.name as usize).unwrap();
                if ***rel_text_symbref[index] == *Cow::Borrowed(&s_name) {
                    rela.info += (symb_index << 8) as u32;
                    break;
                }
            }
            bytes.extend(rela.bytes());
        }
    }
    bytes
}

// ====================================================
//                       Utils
// ====================================================
impl Elf32Shdr {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.name.to_le_bytes());
        bytes.extend(self.etype.to_le_bytes());
        bytes.extend(self.flags.to_le_bytes());
        bytes.extend(self.addr.to_le_bytes());
        bytes.extend(self.offset.to_le_bytes());
        bytes.extend(self.size.to_le_bytes());
        bytes.extend(self.link.to_le_bytes());
        bytes.extend(self.info.to_le_bytes());
        bytes.extend(self.addralign.to_le_bytes());
        bytes.extend(self.entsize.to_le_bytes());
        bytes
    }
}

impl Elf32Ehdr {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.ident);
        bytes.extend(self.etype.to_le_bytes());
        bytes.extend(self.machine.to_le_bytes());
        bytes.extend(self.version.to_le_bytes());
        bytes.extend(self.entry.to_le_bytes());
        bytes.extend(self.phoff.to_le_bytes());
        bytes.extend(self.shoff.to_le_bytes());
        bytes.extend(self.flags.to_le_bytes());
        bytes.extend(self.ehsize.to_le_bytes());
        bytes.extend(self.phentsize.to_le_bytes());
        bytes.extend(self.phnum.to_le_bytes());
        bytes.extend(self.shentsize.to_le_bytes());
        bytes.extend(self.shnum.to_le_bytes());
        bytes.extend(self.shstrndx.to_le_bytes());
        bytes
    }
}

impl Elf32Sym {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.name.to_le_bytes());
        bytes.extend(self.value.to_le_bytes());
        bytes.extend(self.size.to_le_bytes());
        bytes.extend(self.info.to_le_bytes());
        bytes.push(self.other);
        bytes.extend(self.shndx.to_le_bytes());
        bytes
    }
}

impl Elf32Rel {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.offset.to_le_bytes());
        bytes.extend(self.info.to_le_bytes());
        bytes
    }
}
impl Elf32Rela {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.offset.to_le_bytes());
        bytes.extend(self.info.to_le_bytes());
        bytes.extend(self.addend.to_le_bytes());
        bytes
    }
}

#[inline]
fn collect_asciiz(tab: &[u8], start: usize) -> Option<String> {
    if tab.len() < start {
        return None;
    }
    let mut string = String::new();

    for n in &tab[start..] {
        if n != &0x0 {
            string.push(*n as char);
        } else {
            break;
        }
    }
    Some(string)
}
