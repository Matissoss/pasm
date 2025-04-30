// rasmx86_64 - src/core/obj/elf64.rs
// ----------------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;
use std::path::PathBuf;

use crate::{
    core::reloc::Relocation,
    shr::{
        var::VarContent,
        symbol::{
            Symbol, 
            SymbolType as SType, 
            Visibility
        }
    },
};

type Elf64Addr = u64;
type Elf64Half = u16;
type Elf64Off = u64;
type Elf64Word = u32;
type UnsignedChar = u8;

const EI_NIDENT: usize = 16;
const SHF_WRITE: u64 = 0x1;
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
    name: Elf64Word,
    etype: Elf64Word,
    flags: u64,
    addr: Elf64Addr,
    offset: Elf64Off,
    size: u64,
    link: Elf64Word,
    info: Elf64Word,
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
    offset  : u64,
    info    : u64,
    addend  : i64,
}

pub fn make_elf64(
    code: &[u8],
    relocs: Vec<Relocation>,
    symbols: &[Symbol],
    outpath: &PathBuf,
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
            'E' as u8,
            'L' as u8,
            'F' as u8,
            ei_class,
            ei_data,
            ei_version,
            ei_osabi,
            ei_osabiversion,
            0,0,0,0,0,0,0,
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

    // ==========================
    //  .data, .rodata and .bss
    // ==========================

    let mut rodata_shdr = Elf64Shdr {
        name: 0,
        etype: 1,
        flags: SHF_ALLOC,
        addr: 0,
        offset: 0,
        size: 0,
        link: 0,
        info: 0,
        addralign: 0,
        entsize: 0,
    };
    let mut bss_shdr = Elf64Shdr {
        name: 0,
        etype: 8,
        flags: SHF_ALLOC | SHF_WRITE,
        addr: 0,
        offset: 0,
        size: 0,
        link: 0,
        info: 0,
        addralign: 0,
        entsize: 0,
    };

    let mut data_shdr = Elf64Shdr {
        name: 0,
        etype: 1,
        flags: SHF_ALLOC | SHF_WRITE,
        addr: 0,
        offset: (size_of::<Elf64Ehdr>() + code.len()) as u64,
        size: 0,
        link: 0,
        info: 0,
        addralign: 0,
        entsize: 0,
    };

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
                rel_text_symbref.push(Cow::Owned::<&String>(&reloc.symbol));
                rels.push(reloc);
            } else {
                rela_text_symbref.push(Cow::Owned::<&String>(&reloc.symbol));
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
                offset: rel.offset,
                info: rel.rtype as u64,
            });
        }
        for rela in relas {
            rela_text_symb.push(Elf64Rela {
                offset: rela.offset,
                info: rela.rtype as u64,
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

    let (mut bss, mut data, mut rodata) = (false, false, false);
    let (mut bss_r, mut data_r, mut rodata_r): (Vec<usize>, Vec<usize>, Vec<usize>) =
        (Vec::new(), Vec::new(), Vec::new());
    let (mut bss_b, mut data_b, mut rodata_b): (Vec<u8>, Vec<u8>, Vec<u8>) =
        (Vec::new(), Vec::new(), Vec::new());

    // symbols
    let mut strtab: Vec<UnsignedChar> = vec![0];
    let mut symbols: Vec<Elf64Sym> = {
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

        let elfsymlen = elf_symbs.len();
        let mut glob_num = 0;
        for (index, symbol) in symbols.iter().enumerate() {
            if symbol.visibility == Visibility::Local {
                match symbol.addt {
                    /*  .data  */
                    0x1 => {
                        data_r.push(index + elfsymlen - glob_num);
                        data_b.extend(Cow::Owned::<Option<&Cow<VarContent>>>(symbol.content.as_ref()).unwrap().bytes());
                        data = true;
                    }
                    /* .rodata */
                    0x2 => {
                        rodata_r.push(index + elfsymlen - glob_num);
                        rodata_b.extend(Cow::Owned::<Option<&Cow<VarContent>>>(symbol.content.as_ref()).unwrap().bytes());
                        rodata = true;
                    }
                    /*   .bss  */
                    0x3 => {
                        bss_r.push(index + elfsymlen - glob_num);
                        bss_b.extend(vec![0; symbol.size.unwrap() as usize]);
                        bss = true;
                    }
                    _ => {}
                }
            }
            if symbol.visibility == Visibility::Global {
                glob_num += 1;
                global_symbols.push(symbol);
            } else {
                elf_symbs.push(Elf64Sym {
                    name: strtab.len() as Elf64Word,
                    value: symbol.offset,
                    size: if let Some(s) = symbol.size {
                        s as u64
                    } else {
                        0
                    },
                    info: ((symbol.visibility as u8) << 4) + symbol.stype as u8,
                    other: 0,
                    shndx: symbol.sindex,
                });
                strtab.extend(symbol.name.as_bytes());
                strtab.extend(b"\0");
            }
        }
        symtab_hdr.info = elf_symbs.len() as Elf64Word;

        let base_len = elf_symbs.len();
        for (index, symbol) in global_symbols.iter().enumerate() {
            match symbol.addt {
                /*  .data  */
                0x1 => {
                    data_r.push(index + base_len);
                    data_b.extend(Cow::Owned::<Option<&Cow<VarContent>>>(symbol.content.as_ref()).unwrap().bytes());
                    data = true;
                }
                /* .rodata */
                0x2 => {
                    rodata_r.push(index + base_len);
                    rodata_b.extend(Cow::Owned::<Option<&Cow<VarContent>>>(symbol.content.as_ref()).unwrap().bytes());
                    rodata = true;
                }
                /*   .bss  */
                0x3 => {
                    bss_r.push(index + base_len);
                    bss_b.extend(vec![0; symbol.size.unwrap() as usize]);
                    bss = true;
                }
                _ => {}
            }
            elf_symbs.push(Elf64Sym {
                name: strtab.len() as Elf64Word,
                value: symbol.offset,
                size: if let Some(s) = symbol.size {
                    s as u64
                } else {
                    0
                },
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
    let mut symbols_offset = (size_of::<Elf64Ehdr>()
        + code.len()
        + (elf_header.shnum as usize * size_of::<Elf64Shdr>()))
        as Elf64Off;

    let mut bytes = Vec::new();

    let modf = {
        let mut modf = 0;
        if bss {
            modf += 1
        }
        if data {
            modf += 1
        }
        if rodata {
            modf += 1
        }
        modf
    };
    if bss {
        symbols_offset += size_of::<Elf64Shdr>() as Elf64Off;
        elf_header.shnum += 1;
    }
    if data {
        elf_header.shnum += 1;
        elf_header.shoff += data_b.len() as u64;
        symbols_offset += data_b.len() as u64 + size_of::<Elf64Shdr>() as Elf64Off;
    }
    if rodata {
        elf_header.shnum += 1;
        elf_header.shoff += rodata_b.len() as u64;
        symbols_offset += rodata_b.len() as u64 + size_of::<Elf64Shdr>() as Elf64Off;
    }

    elf_header.shstrndx += modf;
    bytes.extend(elf_header.bytes());
    bytes.extend(code);
    if data {
        bytes.extend(&data_b);
    }
    if rodata {
        bytes.extend(&rodata_b);
    }
    bytes.extend(null_shdr.bytes());
    bytes.extend(text_shdr.bytes());

    let mut addt_section = 0;
    if data {
        data_shdr.name = shstrtab.len() as Elf64Word;
        data_shdr.offset = (size_of::<Elf64Ehdr>() + code.len()) as u64;
        data_shdr.size = data_b.len() as u64;

        let mut data_symb = Vec::new();
        for i in data_r {
            let symb = &mut symbols[i];
            symb.shndx = 2 + addt_section as u16;
            let name = collect_asciiz(&strtab, symb.name as usize).unwrap();
            data_symb.push((name, symb.value, symb.size, None));
        }

        data_shdr.addralign = calc_lsize(&data_symb);
        shstrtab.extend(b".data\0");
        bytes.extend(data_shdr.bytes());
        addt_section += 1;
    }
    if rodata {
        rodata_shdr.name = shstrtab.len() as Elf64Word;
        rodata_shdr.offset = (size_of::<Elf64Ehdr>() + code.len() + data_b.len()) as u64;
        rodata_shdr.size = data_b.len() as u64;

        let mut rodata_symb = Vec::new();
        for i in rodata_r {
            let symb = &mut symbols[i];
            symb.shndx = 2 + addt_section as u16;
            let name = collect_asciiz(&strtab, symb.name as usize).unwrap();
            rodata_symb.push((name, symb.value, symb.size, None));
        }
        rodata_shdr.addralign = calc_lsize(&rodata_symb);
        shstrtab.extend(b".rodata\0");
        bytes.extend(rodata_shdr.bytes());

        addt_section += 1;
    }
    if bss {
        bss_shdr.name = shstrtab.len() as Elf64Word;
        bss_shdr.offset = (size_of::<Elf64Ehdr>() + code.len()) as Elf64Off;
        bss_shdr.size = bss_b.len() as u64;

        let mut bss_symb = Vec::new();
        for i in bss_r {
            let symb = &mut symbols[i];
            symb.shndx = 2 + addt_section as u16;
            let name = collect_asciiz(&strtab, symb.name as usize).unwrap();

            bss_symb.push((name, symb.value, symb.size, None));
        }

        bss_shdr.addralign = calc_lsize(&bss_symb);
        shstrtab.extend(b".bss\0");
        bytes.extend(bss_shdr.bytes());
    }

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

    if let Some(_) = rel_text_shdr {
        let mut index = 0;
        for mut rel in rel_text_symb {
            let mut symb_index = 0;
            for s in &symbols {
                let s_name = collect_asciiz(&strtab, s.name as usize).unwrap();
                if **rel_text_symbref[index] == *Cow::Borrowed(&s_name) {
                    rel.info = ((symb_index as u64) << 32) as u64 + rel.info;
                    break;
                }
                symb_index += 1;
            }
            bytes.extend(rel.bytes());
            index += 1;
        }
    }
    if let Some(_) = rela_text_shdr {
        let mut index = 0;
        for mut rela in rela_text_symb {
            let mut symb_index = 0;
            for s in &symbols {
                let s_name = collect_asciiz(&strtab, s.name as usize).unwrap();
                if **rela_text_symbref[index] == *Cow::Borrowed(&s_name) {
                    rela.info += ((symb_index as u64) << 32) as u64;
                    break;
                }
                symb_index += 1;
            }
            bytes.extend(rela.bytes());
            index += 1;
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

fn calc_lsize(symbs: &[(String, u64, u64, Option<VarContent>)]) -> u64 {
    let mut iter = symbs.iter();
    let mut lsize = 1;
    let mut prvoff = 0;
    while let Some(i) = iter.next() {
        let tsize = i.1 - prvoff;
        if let Some(VarContent::String(_)) = i.3 {
            lsize = lsize.max(1);
        }
        else {
            if tsize % 4 == 0 {
                lsize = lsize.max(4);
            } else if tsize % 8 == 0 {
                lsize = lsize.max(8);
            }
        }
        prvoff = i.1;
    }
    return lsize;
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
