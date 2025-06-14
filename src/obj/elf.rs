// rasmx86_64 - src/obj/elf.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0
#![allow(unused)]

use crate::shr::{
    error::RASMError as Error,
    reloc::{RelType, Relocation},
    section::Section,
    symbol::Symbol,
};

// section constants
const SHT_NULL: u32 = 0;
// .text section
const SHT_PROGBITS: u32 = 1;

const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_REL: u32 = 9;
// .bss
const SHT_NOBITS: u32 = 8;

const EHDR_SIZE_64: u32 = 64;
const EHDR_SIZE_32: u32 = 52;

const SHDR_SIZE_64: u32 = 64;
const SHDR_SIZE_32: u32 = 40;

// flags
const SHF_WRITE: u32 = 0x01;
const SHF_ALLOC: u32 = 0x02;
const SHF_EXECINSTR: u32 = 0x04;

const EM_I386: u8 = 3;
const EM_X86_64: u8 = 62;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElfHeader {
    shstrtab_index: usize,
    section_count: usize,
    section_offset: u32,
    machine: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElfSymbol {
    name: usize,
    // always address
    value: u32,
    size: u32,
    // currently can be set to 1
    section_index: u32,
    info: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElfRelocation {
    offset: u32,
    info: u32,
    addend: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElfSection {
    // index in shstrtab
    name: usize,
    size: u32,
    offset: u32,
    stype: u32,
    addralign: u32,
    entry_count: u32,
    info: u32,
    link: u32,
    flags: u32,
    entry_size: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Elf<'a> {
    code: &'a [u8],
    sections: Vec<ElfSection>,
    header: ElfHeader,
    shstrtab: Vec<u8>,
    strtab: Vec<u8>,
    symbols: Vec<ElfSymbol>,
    relocations: Vec<ElfRelocation>,
}

pub fn mk_ident(is_64bit: bool, is_le: bool) -> [u8; 16] {
    [
        0x7F,
        b'E',
        b'L',
        b'F',
        is_64bit as u8 + 1,
        is_le as u8 + 1,
        1,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]
}

impl Elf<'_> {
    pub fn compile(self, is_64bit: bool) -> Vec<u8> {
        compile(self, is_64bit)
    }
    fn push_reloc(&mut self, reloc: &TmpRelocation, is_64bit: bool) {
        self.relocations.push(ElfRelocation {
            offset: reloc.offset,
            info: reloc.symbol.wrapping_shr(8)
                + if is_64bit {
                    reloc.reltype.to_elf64_rtype() as u32
                } else {
                    reloc.reltype.to_elf32_rtype()
                },
            addend: reloc.addend,
        });
    }
    pub fn push_symbol(&mut self, symbol: &Symbol) {
        let name = self.push_strtab(symbol.name);
        self.symbols.push(ElfSymbol {
            name,
            value: symbol.offset,
            size: symbol.size,
            section_index: symbol.sindex as u32,
            info: (symbol.visibility as u8).wrapping_shr(4) | (symbol.stype as u8 & 0x0F),
        });
    }
    pub fn push_strtab(&mut self, str: &str) -> usize {
        let len = self.strtab.len();
        self.strtab.extend(str.as_bytes());
        self.strtab.push(0);
        len
    }
    pub fn push_shstrtab(&mut self, str: &str) -> usize {
        let len = self.shstrtab.len();
        self.shstrtab.extend(str.as_bytes());
        self.shstrtab.push(0);
        len
    }
    pub fn add_section(&mut self, section: Section) {
        self.sections.push(ElfSection {
            name: self.shstrtab.len(),
            size: 0,
            flags: 0,
            offset: 0,
            stype: 0,
            addralign: 0,
            entry_count: 0,
            info: 0,
            link: 0,
            entry_size: 16,
        });
        self.shstrtab.extend(section.name.as_bytes());
        self.shstrtab.push(0);
    }
    pub fn push_section(&mut self, section: ElfSection) {
        self.sections.push(section);
    }
}

struct TmpRelocation {
    pub symbol: u32,
    pub offset: u32,
    pub addend: i32,
    pub reltype: RelType,
}

pub fn make_elf<'a>(
    sections: &'a [&'a Section],
    code: &'a [u8],
    relocs: &'a [Relocation<'a>],
    symbols: &'a [Symbol<'a>],
    is_64bit: bool,
) -> Result<Elf<'a>, Error> {
    let mut elf = Elf::default();
    let header_size = if is_64bit { EHDR_SIZE_64 } else { EHDR_SIZE_32 };
    for section in sections {
        let idx = elf.push_shstrtab(&section.name);
        elf.push_section(ElfSection {
            name: idx,
            size: section.size,
            offset: section.offset + header_size,
            // might want to change
            stype: SHT_PROGBITS,

            flags: {
                let w = if section.attributes.write() {
                    SHF_WRITE
                } else {
                    0
                };
                let a = if section.attributes.alloc() {
                    SHF_ALLOC
                } else {
                    0
                };
                let e = if section.attributes.exec() {
                    SHF_EXECINSTR
                } else {
                    0
                };
                w + a + e
            },
            entry_count: 0,

            // we assert that every section currently is code
            // (will be later specified with addralign parameter)
            addralign: 16,

            entry_size: 0,
            // idk
            link: 0,
            info: 0,
        });
    }
    elf.code = code;
    for reloc in relocs {
        if let Some(symbol) = find_index(reloc, symbols) {
            elf.push_reloc(
                &TmpRelocation {
                    symbol: symbol as u32,
                    offset: reloc.offset,
                    addend: reloc.addend,
                    reltype: reloc.reltype,
                },
                is_64bit,
            );
        } else {
            return Err(Error::msg(format!(
                "Could not find symbol \"{}\" which you tried to use. Consider making it extern.",
                reloc.symbol
            )));
        }
    }
    for symbol in symbols {
        elf.push_symbol(symbol);
    }
    Ok(elf)
}

fn shdr_collect(e: ElfSection, is_64bit: bool) -> Vec<u8> {
    let mut b = Vec::new();
    if is_64bit {
        b.extend((e.name as u32).to_le_bytes());
        b.extend(e.stype.to_le_bytes());
        b.extend((e.flags as u64).to_le_bytes());
        b.extend(&[0; 8]);
        b.extend((e.offset as u64).to_le_bytes());
        b.extend((e.size as u64).to_le_bytes());
        b.extend(e.link.to_le_bytes());
        b.extend(e.info.to_le_bytes());
        b.extend((e.addralign as u64).to_le_bytes());
        b.extend((e.entry_size as u64).to_le_bytes());
    } else {
        b.extend((e.name as u32).to_le_bytes());
        b.extend(e.stype.to_le_bytes());
        b.extend(e.flags.to_le_bytes());
        b.extend(&[0; 4]);
        b.extend(e.offset.to_le_bytes());
        b.extend(e.size.to_le_bytes());
        b.extend(e.link.to_le_bytes());
        b.extend(e.info.to_le_bytes());
        b.extend(e.addralign.to_le_bytes());
        b.extend(e.entry_size.to_le_bytes());
    }
    b
}

fn ehdr_collect(e: ElfHeader, is_64bit: bool) -> Vec<u8> {
    let mut b = Vec::new();
    if is_64bit {
        b.extend(1u16.to_le_bytes());
        b.extend((e.machine as u16).to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(&[0; 8]);
        b.extend(&[0; 8]);
        b.extend((e.section_offset as u64).to_le_bytes());
        b.extend(&[0; 4]);
        // ehdr_size
        b.extend((EHDR_SIZE_64 as u16).to_le_bytes());
        b.extend(&[0; 2]);
        b.extend(&[0; 2]);
        b.extend((SHDR_SIZE_64 as u16).to_le_bytes());
        b.extend((e.section_count as u16).to_le_bytes());
        b.extend((e.shstrtab_index as u16).to_le_bytes());
    } else {
        b.extend(1u16.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(&[0; 4]);
        b.extend(&[0; 4]);
        b.extend(e.section_offset.to_le_bytes());
        b.extend(&[0; 4]);
        b.extend((EHDR_SIZE_32 as u16).to_le_bytes());
        b.extend(&[0; 2]);
        b.extend(&[0; 2]);
        b.extend((SHDR_SIZE_32 as u16).to_le_bytes());
        b.extend((e.section_count as u16).to_le_bytes());
        b.extend((e.shstrtab_index as u16).to_le_bytes());
    }
    b
}

fn find_index<'a>(reloc: &'a Relocation<'a>, symbols: &'a [Symbol<'a>]) -> Option<usize> {
    for (idx, s) in symbols.iter().enumerate() {
        if s.name == reloc.symbol {
            return Some(idx);
        }
    }
    None
}

// planned rasm's ELF layout:
// - Elf Header (obviously)
// - Section headers (in order):
//      - .shstrtab
//      - .strtab
//      - .symtab
//      - other sections:
//          - x
//          - .rel.x
//          - .rela.x
// - Content:
//      - .shstrtab
//      - .strtab
//      - .symtab
//      - code
//      - other sections:
//          - x
//          - .rel.x
//          - .rela.x
//
fn compile(mut elf: Elf, is_64bit: bool) -> Vec<u8> {
    // ELF header:
    // [...]
    // Section headers:
    // [...]
    // Content:
    // [...]
    Vec::new()
}
