// pasm - src/obj/elf.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use std::path::Path;

use crate::utils;

use crate::shr::{
    error::Error,
    reloc::{RelType, Relocation},
    section::Section,
    symbol::{Symbol, SymbolType},
    visibility::Visibility,
};

// section constants
const SHT_PROGBITS: u32 = 1;

const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_NOBITS: u32 = 8;

const RELA_SIZE_64: usize = 24;
const RELA_SIZE_32: usize = 12;

const SYM_SIZE_64: usize = 24;
const SYM_SIZE_32: usize = 16;

const EHDR_SIZE_64: usize = 64;
const EHDR_SIZE_32: usize = 52;

const SHDR_SIZE_64: usize = 64;
const SHDR_SIZE_32: usize = 40;

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
    section_offset: usize,
    machine: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElfSymbol {
    name: usize,
    // always address
    value: usize,
    size: usize,
    section_index: u32,

    visibility: u8,

    info: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElfRelocation {
    // elf info
    offset: u64,
    addend: i64,
    info: u64,
    // rasm's added
    sindex: u16,
    iglob: bool,
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

fn mk_ident(is_64bit: bool, is_le: bool) -> [u8; 16] {
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

type Sections<'a> = &'a [Section<'a>];

impl<'a> Elf<'a> {
    pub fn new(
        sections: Sections<'a>,
        opath: &'a Path,
        code: &'a [u8],
        relocs: Vec<Relocation>,
        symbols: &'a [Symbol],
        is_64bit: bool,
    ) -> Result<Self, Error> {
        make_elf(sections, opath, code, relocs, symbols, is_64bit)
    }
    pub fn compile(self, is_64bit: bool) -> Vec<u8> {
        compile(self, is_64bit)
    }
    fn get_local_symbol_count(&self) -> usize {
        self.symbols.len() - self.get_global_count()
    }
    fn find_symbol(&self, name: &str) -> Option<usize> {
        let buf = &self.strtab;
        for (i, s) in self.symbols.iter().enumerate() {
            let sname = unsafe { utils::cstring(buf.as_ptr().add(s.name)) };
            if sname == name {
                return Some(i);
            }
        }
        None
    }
    fn push_reloc(&mut self, reloc: &TmpRelocation, is_64bit: bool) {
        let symb = reloc.symbol as u64;
        self.relocations.push(ElfRelocation {
            offset: reloc.offset as u64,
            info: symb << 32
                | if is_64bit {
                    (reloc.reltype.to_elf64_rtype() & 0xFF) as u32
                } else {
                    reloc.reltype.to_elf32_rtype() & 0xFF
                } as u64,
            addend: reloc.addend,
            sindex: reloc.sindex,
            iglob: false,
        });
    }
    fn push_symbols(&mut self, symbols: &[Symbol]) {
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
    fn push_symbol(&mut self, symbol: &Symbol) {
        let name = self.push_strtab(symbol.name);
        self.symbols.push(ElfSymbol {
            name,
            value: symbol.offset,
            size: symbol.size,
            section_index: symbol.sindex as u32 + 4,
            info: (match symbol.visibility {
                Visibility::Public => 1,
                Visibility::Local => 0,
                Visibility::Weak => 2,
                _ => 0,
            }) << 4
                | (symbol.stype as u8 & 0x0F),
            visibility: match symbol.visibility {
                Visibility::Anonymous => 2,
                Visibility::Protected => 3,
                _ => 0,
            },
        });
    }
    fn push_strtab(&mut self, str: &str) -> usize {
        let len = self.strtab.len();
        self.strtab.extend(str.as_bytes());
        self.strtab.push(0);
        len
    }
    fn push_shstrtab(&mut self, str: &str) -> usize {
        let len = self.shstrtab.len();
        self.shstrtab.extend(str.as_bytes());
        self.shstrtab.push(0);
        len
    }
    fn get_global_count(&self) -> usize {
        let mut idx = 0;
        for symb in &self.symbols {
            // check if symbol is global
            if (symb.info & 0b1111000) == const { 1 << 4 } {
                break;
            }
            idx += 1;
        }
        self.symbols.len() - idx
    }
    fn push_section(&mut self, section: ElfSection) {
        self.sections.push(section);
    }
}

struct TmpRelocation {
    symbol: usize,
    offset: usize,
    addend: i64,
    sindex: u16,
    reltype: RelType,
}

fn make_elf<'a>(
    sections: Sections<'a>,
    outpath: &'a Path,
    code: &'a [u8],
    relocs: Vec<Relocation>,
    symbols: &'a [Symbol],
    is_64bit: bool,
) -> Result<Elf<'a>, Error> {
    let mut elf = Elf::default();
    elf.shstrtab.push(0);
    elf.strtab.push(0);
    elf.symbols.push(ElfSymbol::default());

    // push file symbol
    let file_name = elf.push_strtab(&outpath.to_string_lossy());
    elf.symbols.push(ElfSymbol {
        name: file_name,
        value: 0,
        info: SymbolType::File as u8,
        section_index: 0xFFF1,
        size: 0,
        visibility: 0,
    });
    let iter = sections.iter();
    for section in iter {
        let idx = elf.push_shstrtab(section.name);
        elf.push_section(ElfSection {
            name: idx,
            size: section.size,
            offset: if section.attributes.get_nobits() {
                0
            } else {
                section.offset
            },
            stype: if section.attributes.get_nobits() {
                SHT_NOBITS
            } else {
                SHT_PROGBITS
            },

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

            addralign: section.align as u32,
            entry_size: 0,
            // idk
            link: 0,
            info: 0,
        });
    }
    elf.code = code;
    elf.push_symbols(symbols);
    for reloc in relocs {
        if let Some(idx) = elf.find_symbol(reloc.symbol) {
            elf.push_reloc(
                &TmpRelocation {
                    symbol: idx,
                    offset: reloc.offset,
                    addend: reloc.addend.into(),
                    reltype: reloc.reltype,
                    sindex: reloc.shidx,
                },
                is_64bit,
            );
        } else {
            return Err(Error::new(
                format!("usage of undefined symbol \"{}\"", reloc.symbol),
                4,
            ));
        }
    }
    Ok(elf)
}

fn shdr_collect(e: ElfSection, is_64bit: bool) -> Vec<u8> {
    let mut b = Vec::with_capacity(SHDR_SIZE_32);
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
        b.extend((e.offset as u32).to_le_bytes());
        b.extend((e.size as u32).to_le_bytes());
        b.extend(e.link.to_le_bytes());
        b.extend(e.info.to_le_bytes());
        b.extend(e.addralign.to_le_bytes());
        b.extend(e.entry_size.to_le_bytes());
    }
    b
}

fn ehdr_collect(e: ElfHeader, is_64bit: bool) -> Vec<u8> {
    let mut b = Vec::with_capacity(EHDR_SIZE_32);
    if is_64bit {
        b.extend(1u16.to_le_bytes());
        b.extend((e.machine as u16).to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(&[0; 8]);
        b.extend(&[0; 8]);
        b.extend((e.section_offset as u64).to_le_bytes());
        b.extend(&[0; 4]);
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
        b.extend((e.section_offset as u32).to_le_bytes());
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
    let mut b = Vec::with_capacity(RELA_SIZE_32);
    if is_64bit {
        b.extend(rel.offset.to_le_bytes());
        b.extend(rel.info.to_le_bytes());
        b.extend(rel.addend.to_le_bytes());
    } else {
        b.extend((rel.offset as u32).to_le_bytes());
        let rel_info_high = rel.info & 0xFFFF_FFFF_0000_0000;
        let rel_info_new = (rel_info_high >> 24) as u32 | rel.info as u32;
        b.extend(rel_info_new.to_le_bytes());
        b.extend((rel.addend.try_into().unwrap_or(0i32)).to_le_bytes());
    }
    b
}

fn sym_collect(symb: ElfSymbol, is_64bit: bool) -> Vec<u8> {
    let mut b = Vec::with_capacity(SYM_SIZE_32);
    if is_64bit {
        b.extend((symb.name as u32).to_le_bytes());
        b.extend(symb.info.to_le_bytes());
        b.extend(symb.visibility.to_le_bytes());
        b.extend((symb.section_index as u16).to_le_bytes());
        b.extend((symb.value as u64).to_le_bytes());
        b.extend((symb.size as u64).to_le_bytes());
    } else {
        b.extend((symb.name as u32).to_le_bytes());
        b.extend((symb.value as u32).to_le_bytes());
        b.extend((symb.size as u32).to_le_bytes());
        b.extend(symb.info.to_le_bytes());
        b.extend(symb.visibility.to_le_bytes());
        b.extend((symb.section_index as u16).to_le_bytes());
    }
    b
}

// pasm's ELF layout:
// - Elf Header (obviously)
// - Section headers (in order):
//      - .shstrtab
//      - .strtab
//      - .symtab
//      - other sections:
//          - x
//          - .rela.x
// - Content:
//      - .shstrtab
//      - .strtab
//      - .symtab
//      - code
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
    let rela_size = if is_64bit { RELA_SIZE_64 } else { RELA_SIZE_32 };
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

    let mut rela_info: Vec<RelInfo> = Vec::new();
    let mut rela_shidx = 0;
    let mut rela_count = 0;
    if !elf.relocations.is_empty() {
        for reloc in &elf.relocations {
            if reloc.sindex == rela_shidx {
                rela_count += 1;
            } else {
                rela_info.push(RelInfo {
                    name: 0,
                    relcount: rela_count,
                });
                rela_count = 1;
                rela_shidx += 1;
            }
        }
    }
    if rela_count != 0 {
        rela_info.push(RelInfo {
            name: 0,
            relcount: rela_count,
        });
    }

    for r in &rela_info {
        if r.relcount != 0 {
            elf.header.section_count += 1;
        }
    }

    bytes.extend(ehdr_collect(elf.header, is_64bit));
    // reserved:
    let shstrtab_name = elf.push_shstrtab(".shstrtab");
    let strtab_name = elf.push_shstrtab(".strtab");
    let symtab_name = elf.push_shstrtab(".symtab");

    for idx in 0..rela_info.len() {
        if rela_info[idx].relcount != 0 {
            let cstr = format!(".rela{}", unsafe {
                utils::cstring(elf.shstrtab.as_ptr().add(elf.sections[idx].name))
            });
            rela_info[idx].name = elf.push_shstrtab(&cstr);
        }
    }

    let content_offset = ehdr_size + (elf.header.section_count * shdr_size);
    let strtab_offset = content_offset + elf.shstrtab.len();
    let symtab_offset = strtab_offset + elf.strtab.len();
    let code_offset = symtab_offset + (sym_size * elf.symbols.len());
    let rela_offset = code_offset + elf.code.len();
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
            size: elf.shstrtab.len(),
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
            size: elf.strtab.len(),
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
            info: elf.get_local_symbol_count() as u32,
            entry_size: sym_size as u32,
            addralign: 0,
            size: elf.symbols.len() * sym_size,
        },
        is_64bit,
    ));
    // other sections
    for mut section in elf.sections {
        if section.stype != SHT_NOBITS {
            section.offset += code_offset;
        }
        bytes.extend(shdr_collect(section, is_64bit));
    }
    if !elf.relocations.is_empty() {
        let mut offs = 0;
        for (idx, relc) in rela_info.iter().enumerate() {
            if relc.relcount != 0 {
                bytes.extend(shdr_collect(
                    ElfSection {
                        name: relc.name,
                        stype: SHT_RELA,
                        info: 4 + idx as u32,
                        link: 3,
                        size: relc.relcount * rela_size,
                        entry_size: rela_size as u32,
                        addralign: 0,
                        offset: rela_offset + offs,
                        entry_count: relc.relcount as u32,
                        flags: 0,
                    },
                    is_64bit,
                ));
                offs += relc.relcount * rela_size;
            }
        }
    }

    // Content:
    bytes.extend(elf.shstrtab);
    bytes.extend(elf.strtab);
    for symbol in elf.symbols {
        bytes.extend(sym_collect(symbol, is_64bit));
    }
    bytes.extend(elf.code);

    for rel in elf.relocations {
        bytes.extend(reloc_collect(rel, is_64bit));
    }
    bytes
}

#[derive(Clone, Copy)]
struct RelInfo {
    name: usize,
    relcount: usize,
}
