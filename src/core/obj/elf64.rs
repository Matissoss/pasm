// rasmx86_64 - src/core/obj/elf64.rs
// ----------------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;
use std::path::Path;

use crate::shr::{
    reloc::Relocation,
    symbol::{Symbol, SymbolType as SType, Visibility},
};

type Elf64Addr = u64;
type Elf64Half = u16;
type Elf64Off = u64;
type Elf64Word = u32;
type UnsignedChar = u8;

const EI_NIDENT: usize = 16;
const SHF_ALLOC: u64 = 0x2;
const SHF_EXECINSTR: u64 = 0x4;

// elf header
struct Elf64Ehdr {
    ident: [UnsignedChar; EI_NIDENT],
    etype: Elf64Half,
    machine: Elf64Half,
    version: Elf64Word,
    entry: Elf64Addr,
    phoff: Elf64Off,
    shoff: Elf64Off,
    flags: Elf64Word,
    ehsize: Elf64Half,
    phentsize: Elf64Half,
    phnum: Elf64Half,
    shentsize: Elf64Half,
    shnum: Elf64Half,
    shstrndx: Elf64Half,
}

// section header
#[derive(Debug, Clone, Copy)]
struct Elf64Shdr {
    name: u32,
    etype: u32,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entsize: u64,
}

#[derive(Debug, Clone, Copy)]
struct Elf64Sym {
    name: Elf64Word,
    info: u8,
    other: u8,
    shndx: Elf64Half,
    value: Elf64Addr,
    size: u64,
}

#[derive(Debug)]
struct Elf64Rel {
    offset: Elf64Addr,
    info: u64,
}

struct Elf64Rela {
    offset: u64,
    info: u64,
    addend: i64,
}

pub fn make_elf64(
    code: &[u8],
    relocs: Vec<Relocation>,
    symbols: &[Symbol],
    outpath: &Path,
) -> Vec<u8> {
    // elf class
    // 1 - 32-bit object
    // 2 - 64-bit object
    let ei_class = 2;
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
    let machine = 62;
    let mut elf_header = Elf64Ehdr {
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
        shoff: (size_of::<Elf64Ehdr>() + code.len()) as u64, // section header offset
        flags: 0,                                            // not used
        ehsize: size_of::<Elf64Ehdr>() as u16,               // elf header size
        phentsize: 0,                                        // not used
        phnum: 0,                                            // not used
        shentsize: size_of::<Elf64Shdr>() as u16,            // section header size
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
        for reloc in relocs {
            if reloc.addend == 0 {
                rel_text_symbref.push(Cow::Owned::<String>(reloc.symbol.clone()));
                rels.push(reloc);
            } else {
                rela_text_symbref.push(Cow::Owned::<String>(reloc.symbol.clone()));
                relas.push(reloc);
            }
        }
        if !rels.is_empty() {
            elf_header.shnum += 1;
            rel_text_shdr = Some(Elf64Shdr {
                name: shstrtab.len() as Elf64Word,
                etype: 9,
                info: 1,
                link: 0,
                flags: 0,
                addr: 0,
                addralign: 0,
                offset: 0,
                size: (size_of::<Elf64Rel>() * rels.len()) as u64,
                entsize: size_of::<Elf64Rel>() as u64,
            });
            shstrtab.extend(b".rel.text\0");
        }
        if !relas.is_empty() {
            elf_header.shnum += 1;
            rela_text_shdr = Some(Elf64Shdr {
                name: shstrtab.len() as Elf64Word,
                etype: 4,
                info: 1,
                link: 0,
                flags: 0,
                addr: 0,
                addralign: 0,
                offset: 0,
                size: (size_of::<Elf64Rela>() * relas.len()) as u64,
                entsize: size_of::<Elf64Rela>() as u64,
            });
            shstrtab.extend(b".rela.text\0");
        }
        for rel in rels {
            rel_text_symb.push(Elf64Rel {
                offset: rel.offset as u64,
                info: rel.reltype.to_elf64_rtype(),
            });
        }
        for rela in relas {
            rela_text_symb.push(Elf64Rela {
                offset: rela.offset as u64,
                info: rela.reltype.to_elf64_rtype(),
                addend: rela.addend as i64,
            });
        }
    }

    // =====================
    //  Elf section headers
    // =====================

    let null_shdr = Elf64Shdr {
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

    let text_shdr = Elf64Shdr {
        name: shstrtab.len() as Elf64Word,
        etype: 1,
        flags: SHF_EXECINSTR | SHF_ALLOC,
        addr: 0,
        offset: size_of::<Elf64Ehdr>() as u64,
        size: code.len() as u64,
        link: 0,
        info: 0,
        addralign: 16,
        entsize: 0,
    };
    elf_header.shnum += 1;
    shstrtab.extend(b".text\0");

    let mut symtab_hdr = Elf64Shdr {
        name: shstrtab.len() as Elf64Word,
        etype: 2,
        flags: 0,
        offset: 0, // to set - symbols
        addr: 0,
        size: 0,
        link: 4,
        info: 0,
        addralign: 0,
        entsize: size_of::<Elf64Sym>() as u64,
    };
    elf_header.shnum += 1;
    shstrtab.extend(b".symtab\0");

    let mut shstrtab_hdr = Elf64Shdr {
        name: shstrtab.len() as Elf64Word,
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
    let mut strtab_hdr = Elf64Shdr {
        name: shstrtab.len() as Elf64Word,
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
    let symbols: Vec<Elf64Sym> = {
        let mut elf_symbs = Vec::new();
        // null symbol
        elf_symbs.push(Elf64Sym {
            name: 0,
            value: 0,
            size: 0,
            info: 0,
            other: 0,
            shndx: 0,
        });
        // file symbol
        elf_symbs.push(Elf64Sym {
            name: strtab.len() as Elf64Word,
            value: 0,
            size: 0,
            info: SType::File as u8,
            other: 0,
            shndx: 0xFFF1,
        });
        strtab.extend(outpath.to_string_lossy().as_bytes());
        strtab.extend(b"\0");

        // text section symbol
        elf_symbs.push(Elf64Sym {
            name: strtab.len() as Elf64Word,
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
                elf_symbs.push(Elf64Sym {
                    name: strtab.len() as Elf64Word,
                    value: symbol.offset as u64,
                    size: symbol.size as u64,
                    info: ((symbol.visibility as u8) << 4) + symbol.stype as u8,
                    other: 0,
                    shndx: symbol.sindex,
                });
                strtab.extend(symbol.name.as_bytes());
                strtab.extend(b"\0");
            }
        }
        symtab_hdr.info = elf_symbs.len() as Elf64Word;

        for symbol in global_symbols.iter() {
            elf_symbs.push(Elf64Sym {
                name: strtab.len() as Elf64Word,
                value: symbol.offset as u64,
                size: symbol.size as u64,
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
    let symbols_offset = (size_of::<Elf64Ehdr>()
        + code.len()
        + (elf_header.shnum as usize * size_of::<Elf64Shdr>()))
        as Elf64Off;

    let mut bytes = Vec::new();

    let modf = 0;

    elf_header.shstrndx += modf;
    bytes.extend(elf_header.bytes());
    bytes.extend(code);
    bytes.extend(null_shdr.bytes());
    bytes.extend(text_shdr.bytes());

    symtab_hdr.offset = symbols_offset;
    symtab_hdr.size = (symbols.len() * size_of::<Elf64Sym>()) as u64;
    symtab_hdr.link += modf as Elf64Word;
    bytes.extend(symtab_hdr.bytes());

    // .shstrtab section header
    shstrtab_hdr.offset = symbols_offset + (symbols.len() * size_of::<Elf64Sym>()) as u64;
    shstrtab_hdr.size = shstrtab.len() as u64;
    bytes.extend(shstrtab_hdr.bytes());

    // .strtab section header
    strtab_hdr.offset =
        symbols_offset + (symbols.len() * size_of::<Elf64Sym>()) as u64 + shstrtab.len() as u64;
    strtab_hdr.size = strtab.len() as u64;
    bytes.extend(strtab_hdr.bytes());

    if let Some(mut shdr) = rel_text_shdr {
        shdr.link = (modf + 2) as Elf64Word;
        shdr.offset = symbols_offset
            + (symbols.len() * size_of::<Elf64Sym>()) as u64
            + shstrtab.len() as u64
            + strtab.len() as u64;
        bytes.extend(shdr.bytes());
    }
    if let Some(mut shdr) = rela_text_shdr {
        shdr.link = (modf + 2) as Elf64Word;
        // what is this
        shdr.offset = symbols_offset
            + (symbols.len() * size_of::<Elf64Sym>()) as u64
            + shstrtab.len() as u64
            + strtab.len() as u64
            + (rel_text_symb.len() * size_of::<Elf64Rel>()) as u64;
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
                if **rel_text_symbref[index] == *Cow::Borrowed(&s_name) {
                    rel.info += (symb_index as u64) << 32;
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
                if **rela_text_symbref[index] == *Cow::Borrowed(&s_name) {
                    rela.info += (symb_index as u64) << 32;
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
impl Elf64Shdr {
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

impl Elf64Ehdr {
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

impl Elf64Sym {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.name.to_le_bytes());
        bytes.extend(self.info.to_le_bytes());
        bytes.extend(self.other.to_le_bytes());
        bytes.extend(self.shndx.to_le_bytes());
        bytes.extend(self.value.to_le_bytes());
        bytes.extend(self.size.to_le_bytes());
        bytes
    }
}

impl Elf64Rel {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.offset.to_le_bytes());
        bytes.extend(self.info.to_le_bytes());
        bytes
    }
}
impl Elf64Rela {
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
