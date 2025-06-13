// rasmx86_64 - src/obj/elf.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0
use std::io::{
    Write,
    Error
};

use crate::shr::{
    symbol::Symbol,
    reloc::Relocation,
};

// section constants
const SHT_NULL: u32 = 0;
// .text section
const SHT_PROGBITS: u32 = 1;

const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA  : u32 = 4;
const SHT_REL   : u32 = 9;
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
    machine: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElfSymbol {
    name: usize,
    // currently can be set to 0
    section_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElfRelocation {
    offset: usize,
    info: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElfSection {
    // index in shstrtab
    name: usize,
    size: usize,
    offset: usize,
    stype: u32,
    addralign: u32,
    entry_count: u32,
    info: u32,
    link: u32,
    flags: u32,
    entry_size: u32,
    // TODO
    //properties: ElfProperties,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Elf<'a> {
    code: &'a [u8],
    sections: Vec<ElfSection>,
    header: ElfHeader,
    shstrtab: Vec<u8>,
    symstrtab: Vec<u8>,
    symbols: Vec<ElfSymbol>,
    relocations: &'a [Relocation<'a>],
}

pub fn mk_ident(is_64bit: bool, is_le: bool) -> [u8; 16] {
    [0x7F, b'E', b'L', b'F', is_64bit as u8 + 1, is_le as u8 + 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]
}

impl<'a> Elf<'a> {
    pub fn add_section(&mut self, str: &str) {
        self.sections.push(ElfSection {
            name: self.shstrtab.len(),
            size: 0,
            flags: 0,
            offset: 0,
            stype: 0,
            addralign: 0,
            entry_count: 0,
            info: 0,
        });
        self.shstrtab.extend(str.as_bytes());
        self.shstrtab.push(0);
    }
    pub fn add_symbol(&'a mut self, symbol: Symbol<'a>) {
        self.symbols.push(ElfSymbol {
            name: self.symstrtab.len(),
            section_index: 0,
        });
        self.symstrtab.extend(symbol.name.as_bytes());
        self.symstrtab.push(0);
    }
}

// we currently will assert that there can only be three (or two) main sections:
// - .text
// - .rel.text
// - .rela.text
// (more coming soon, when i'll be able to)
pub fn write_elf<'a>(code: &'a [u8], rels: &'a [Relocation], symbols: &'a [Symbol<'a>], is_64bit: bool, writer: &mut impl Write) -> Result<(), Error> {
    // we have e_ident
    writer.write_all(&mk_ident(is_64bit, true))?;
    // now rest of header
    let mut elf = Elf::default();

    elf.header.machine = if is_64bit {
        EM_X86_64
    } else {
        EM_I386
    };

    // for now planned layout:
    // - .text
    // - .strtab
    // - .shstrtab
    // - .symtab
    // - .rel.text
    // - .rela.text
    elf.header.shstrtab_index = 2;
    elf.header.section_count = if rels.is_empty() {
        // .text + .shstrtab + .strtab + .symtab
        4
    } else {
        // .text + .shstrtab + .strtab + .rel.text + .symtab
        let mut r = 5;
        for rel in rels {
            if rel.addend != 0 {
                // .text + .shstrtab + .strtab + .rel.text + .rela.text + .symtab
                r = 6;
                break;
            }
        }
        r
    };

    Ok(())
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
        b.extend((e.addralign as u64) .to_le_bytes());
        b.extend((e.entry_size as u64).to_le_bytes());
    } else {
        b.extend((e.name as u32).to_le_bytes());
        b.extend((e.stype as u32).to_le_bytes());
        b.extend(e.flags.to_le_bytes());
        b.extend(&[0; 4]);
        b.extend((e.offset as u32).to_le_bytes());
        b.extend((e.size as u32).to_le_bytes());
        b.extend(e.link.to_le_bytes());
        b.extend(e.info.to_le_bytes());
        b.extend(e.addralign.to_le_bytes());
        b.extend(e.entry_size.to_le_bytes());
    }
    b
}
