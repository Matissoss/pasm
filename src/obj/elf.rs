// rasmx86_64 - src/obj/elf.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    error::RASMError as Error,
    reloc::{RelType, Relocation},
    section::Section,
    symbol::{Symbol, SymbolType},
};

// section constants
#[allow(unused)]
const SHT_NULL: u32 = 0;
// .text section
const SHT_PROGBITS: u32 = 1;

const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_REL: u32 = 9;

// .bss
#[allow(unused)]
const SHT_NOBITS: u32 = 8;

const REL_SIZE_64: u32 = 16;
const REL_SIZE_32: u32 = 8;

const RELA_SIZE_64: u32 = 24;
const RELA_SIZE_32: u32 = 12;

const SYM_SIZE_64: u32 = 24;
const SYM_SIZE_32: u32 = 16;

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
    offset: u64,
    addend: i64,
    info: u64,
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
        !is_le as u8 + 1,
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
            offset: reloc.offset.into(),
            info: (reloc.symbol as u64) << 32
                | if is_64bit {
                    (reloc.reltype.to_elf64_rtype() & 0xFF) as u32
                } else {
                    reloc.reltype.to_elf32_rtype() & 0xFF
                } as u64,
            addend: reloc.addend.into(),
        });
    }
    pub fn push_symbols(&mut self, symbols: &[Symbol]) {
        let mut delayed = Vec::with_capacity(symbols.len());
        for symbol in symbols {
            if symbol.is_global() {
                delayed.push(symbol);
            } else {
                self.push_symbol(symbol);
            }
        }
        for symbol in delayed {
            self.push_symbol(symbol);
        }
    }
    pub fn push_symbol(&mut self, symbol: &Symbol) {
        let name = self.push_strtab(symbol.name);
        self.symbols.push(ElfSymbol {
            name,
            value: symbol.offset,
            size: symbol.size,
            section_index: symbol.sindex as u32 + 3,
            info: (symbol.visibility as u8) << 4 | (symbol.stype as u8 & 0x0F),
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
    elf.shstrtab.push(0);
    elf.strtab.push(0);
    elf.symbols.push(ElfSymbol::default());

    let header_size = if is_64bit { EHDR_SIZE_64 } else { EHDR_SIZE_32 };
    for (idx, section) in sections.iter().enumerate() {
        let symb_name = elf.push_strtab(&section.name);
        elf.symbols.push(ElfSymbol {
            name: symb_name,
            section_index: idx as u32 + 4,
            value: 0,
            size: 0,
            info: SymbolType::Section as u8,
        });
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
    elf.push_symbols(symbols);
    Ok(elf)
}

fn shdr_collect(e: ElfSection, is_64bit: bool) -> Vec<u8> {
    let mut b = Vec::with_capacity(SHDR_SIZE_32 as usize);
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
    let mut b = Vec::with_capacity(EHDR_SIZE_32 as usize);
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
        b.extend((e.machine as u16).to_le_bytes());
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

fn reloc_collect(rel: ElfRelocation, is_64bit: bool) -> Vec<u8> {
    let mut b = Vec::with_capacity(REL_SIZE_32 as usize);
    if is_64bit {
        b.extend(rel.offset.to_le_bytes());
        b.extend(rel.info.to_le_bytes());
        if rel.addend != 0 {
            b.extend(rel.addend.to_le_bytes());
        }
    } else {
        b.extend((rel.offset as u32).to_le_bytes());
        let rel_info_high = rel.info & 0xFFFF_FFFF_0000_0000;
        let rel_info_new = (rel_info_high >> 24) as u32 | rel.info as u32;
        b.extend(rel_info_new.to_le_bytes());
        if rel.addend != 0 {
            b.extend((rel.addend.try_into().unwrap_or(0i32)).to_le_bytes());
        }
    }
    b
}

fn sym_collect(symb: ElfSymbol, is_64bit: bool) -> Vec<u8> {
    let mut b = Vec::with_capacity(SYM_SIZE_32 as usize);
    if is_64bit {
        b.extend((symb.name as u32).to_le_bytes());
        b.extend(symb.info.to_le_bytes());
        b.extend([0; 1]);
        b.extend((symb.section_index as u16).to_le_bytes());
        b.extend((symb.value as u64).to_le_bytes());
        b.extend((symb.size as u64).to_le_bytes());
    } else {
        b.extend((symb.name as u32).to_le_bytes());
        b.extend(symb.value.to_le_bytes());
        b.extend(symb.size.to_le_bytes());
        b.extend(symb.info.to_le_bytes());
        b.extend([0; 1]);
        b.extend((symb.section_index as u16).to_le_bytes());
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

// rasm's ELF layout:
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
//      - .rel.*
//      - .rela.*
//
const NULL_SHDR: ElfSection = ElfSection {
    name: 0,
    entry_count: 0,
    info: 0,
    addralign: 0,
    link: 0,
    stype: 0,
    flags: 0,
    offset: 0,
    size: 0,
    entry_size: 0,
};
fn compile(mut elf: Elf, is_64bit: bool) -> Vec<u8> {
    let mut rel = Vec::new();
    let mut rela = Vec::new();

    let rela_size = if is_64bit { RELA_SIZE_64 } else { RELA_SIZE_32 };
    let rel_size = if is_64bit { REL_SIZE_64 } else { REL_SIZE_32 };
    let sym_size = if is_64bit { SYM_SIZE_64 } else { SYM_SIZE_32 };
    let shdr_size = if is_64bit { SHDR_SIZE_64 } else { SHDR_SIZE_32 };
    let ehdr_size = if is_64bit { EHDR_SIZE_64 } else { EHDR_SIZE_32 };

    let mut bytes = Vec::with_capacity(16);
    // ELF header:
    bytes.extend(mk_ident(is_64bit, true));

    // we add .shstrtab, .strtab, .symtab and NULL section
    elf.header.section_count += elf.sections.len() + 4;
    elf.header.shstrtab_index = 1;
    elf.header.machine = if is_64bit { EM_X86_64 } else { EM_I386 };
    elf.header.section_offset = ehdr_size;

    // for now we assert that there is only
    // one section for relocs (wout addend): .rel.text
    if !elf.relocations.is_empty() {
        elf.header.section_count += 1;
        for reloc in &elf.relocations {
            if reloc.addend != 0 {
                rela.push(*reloc);
            } else {
                rel.push(*reloc);
            }
        }
    }
    let uses_rel = !rel.is_empty();
    let uses_rela = !rela.is_empty();

    bytes.extend(ehdr_collect(elf.header, is_64bit));
    // reserved:
    let shstrtab_name = elf.push_shstrtab(".shstrtab");
    let strtab_name = elf.push_shstrtab(".strtab");
    let symtab_name = elf.push_shstrtab(".symtab");

    let (rel_name, rela_name) = {
        if rel.is_empty() && rela.is_empty() {
            (0, 0)
        } else {
            match (uses_rel, uses_rela) {
                (true, true) => (
                    elf.push_shstrtab(".rel.text"),
                    elf.push_shstrtab(".rela.text"),
                ),
                (true, false) => (elf.push_shstrtab(".rel.text"), 0),
                (false, true) => (0, elf.push_shstrtab(".rela.text")),
                (false, false) => (0, 0),
            }
        }
    };

    let content_offset = ehdr_size + (elf.header.section_count as u32 * shdr_size);
    let strtab_offset = content_offset + elf.shstrtab.len() as u32;
    let symtab_offset = strtab_offset + elf.strtab.len() as u32;
    let code_offset = symtab_offset + (sym_size * elf.symbols.len() as u32);
    let rel_offset = code_offset + elf.code.len() as u32;
    let rela_offset = rel_offset + (rel_size * rel.len() as u32);
    // Section headers:
    bytes.extend(shdr_collect(NULL_SHDR, is_64bit));
    // .shstrtab
    bytes.extend(shdr_collect(
        ElfSection {
            name: shstrtab_name,
            stype: SHT_STRTAB,
            addralign: 1,
            info: 0,
            link: 0,
            size: elf.shstrtab.len() as u32,
            entry_count: 0,
            entry_size: 0,
            flags: 0,
            offset: content_offset,
        },
        is_64bit,
    ));
    // .strtab
    bytes.extend(shdr_collect(
        ElfSection {
            name: strtab_name,
            offset: strtab_offset,
            size: elf.strtab.len() as u32,
            stype: SHT_STRTAB,
            entry_count: 0,
            entry_size: 1,
            info: 0,
            addralign: 1,
            link: 0,
            flags: 0,
        },
        is_64bit,
    ));
    // .symtab
    bytes.extend(shdr_collect(
        ElfSection {
            name: symtab_name,
            stype: SHT_SYMTAB,
            flags: 0,
            offset: symtab_offset,
            entry_count: 0,
            link: 2, // ref to .strtab(?)
            info: elf.symbols.len() as u32,
            entry_size: sym_size,
            addralign: 0,
            size: elf.symbols.len() as u32 * sym_size,
        },
        is_64bit,
    ));
    // other sections
    for section in elf.sections {
        bytes.extend(shdr_collect(section, is_64bit));
    }
    if uses_rel {
        bytes.extend(shdr_collect(
            ElfSection {
                name: rel_name,
                stype: SHT_REL,
                info: 4,
                link: 3,
                size: rel.len() as u32 * rel_size,
                entry_size: rel_size,
                addralign: 0,
                offset: rel_offset,
                entry_count: rel.len() as u32,
                flags: 0,
            },
            is_64bit,
        ));
    }
    if uses_rela {
        bytes.extend(shdr_collect(
            ElfSection {
                name: rela_name,
                stype: SHT_RELA,
                info: 4,
                link: 3,
                size: rela.len() as u32 * rela_size,
                entry_size: rela_size,
                addralign: 0,
                offset: rela_offset,
                entry_count: rela.len() as u32,
                flags: 0,
            },
            is_64bit,
        ));
    }

    // Content:
    bytes.extend(elf.shstrtab);
    bytes.extend(elf.strtab);
    for symbol in elf.symbols {
        bytes.extend(sym_collect(symbol, is_64bit));
    }
    bytes.extend(elf.code);
    for rel in rel {
        bytes.extend(reloc_collect(rel, is_64bit));
    }
    for rel in rela {
        bytes.extend(reloc_collect(rel, is_64bit));
    }
    bytes
}
