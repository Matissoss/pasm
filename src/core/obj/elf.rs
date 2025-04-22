// rasmx86_64 - elf.rs
// -------------------
// made by matissoss
// licensed under MPL

use std::path::PathBuf;

use crate::{
    shr::symbol::{
        Symbol,
        SymbolType as SType,
        Visibility
    },
    shr::ast::Section,
    core::reloc::{
        Relocation,
        //RType,
    },
};

type Elf32Addr      = u32;
type Elf32Half      = u16;
type Elf32Off       = u32;
type Elf32Sword     = i32;
type Elf32Word      = u32;
type UnsignedChar   = u8;

const EI_NIDENT     : usize     = 16;
const SHF_WRITE     : Elf32Word = 0x1;
const SHF_ALLOC     : Elf32Word = 0x2;
const SHF_EXECINSTR : Elf32Word = 0x4;

// elf header
struct Elf32Ehdr{
    ident     : [UnsignedChar; EI_NIDENT],
    etype      : Elf32Half,
    machine   : Elf32Half,
    version   : Elf32Word,
    entry     : Elf32Addr,
    phoff     : Elf32Off,
    shoff     : Elf32Off,
    flags     : Elf32Word,
    ehsize    : Elf32Half,
    phentsize : Elf32Half,
    phnum     : Elf32Half,
    shentsize : Elf32Half,
    shnum     : Elf32Half,
    shstrndx  : Elf32Half,
}

// section header
#[derive(Debug, Clone, Copy)]
struct Elf32Shdr{
    name: Elf32Word,
    etype: Elf32Word,
    flags: Elf32Word,
    addr : Elf32Addr,
    offset : Elf32Off,
    size: Elf32Word,
    link: Elf32Word,
    info: Elf32Word,
    addralign: Elf32Word,
    entsize: Elf32Word
}

struct Elf32Sym{
    name: Elf32Word,
    value: Elf32Addr,
    size: Elf32Word,
    info: u8,
    other: u8,
    shndx: Elf32Half,
}

#[derive(Debug)]
struct Elf32Rel{
    offset: Elf32Addr,
    info  : Elf32Word,
}

struct Elf32Rela{
    offset: Elf32Addr,
    info  : Elf32Word,
    addend: Elf32Sword
}

type Sections = Vec<(Section, Vec<u8>, Vec<(String, u32, u32, Option<String>)>)>;
pub fn make_elf32(code: &[u8], relocs: Vec<Relocation>, symbols: &[Symbol], outpath: &PathBuf, sections: Sections) -> Vec<u8>{
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
    let mut elf_header = Elf32Ehdr{
        ident: [0x7F, 'E' as u8, 'L' as u8, 'F' as u8, 
            ei_class, ei_data, ei_version, ei_osabi, ei_osabiversion, 
            0, 0, 0, 0, 0, 0, 0
        ],
        etype,
        machine,
        version   : 1,
        entry     : 0, // not used
        phoff     : 0, // not used
        shoff     : (size_of::<Elf32Ehdr>() + code.len()) as u32, // section header offset
        flags     : 0, // not used
        ehsize    : size_of::<Elf32Ehdr>() as u16, // elf header size
        phentsize : 0,  // not used
        phnum     : 0,  // not used
        shentsize : size_of::<Elf32Shdr>() as u16, // section header size
        shnum     : 0,  // section header entries
        shstrndx  : 3,  // section header string table (.shstrtab) index
    };
    let mut shstrtab : Vec<UnsignedChar> = vec![0];

    // =====================
    //    .data and .bss
    // =====================
    let (data_data, bss_data) = {
        let mut data = (Vec::new(), vec![]);
        let mut bss  = (Vec::new(), vec![]);
        for section in sections{
            if section.0 == Section::Data {
                data.0 = section.1;
                data.1 = section.2.to_vec();
            }
            else if section.0 == Section::Bss{
                bss.0 = section.1;
                bss.1 = section.2.to_vec();
            }
        }
        (data, bss)
    };

    let bss_shdr = {
        if bss_data != (Vec::new(), vec![]){
            elf_header.shnum += 1;
            let lsize_bssdata = calc_lsize(&bss_data.1);
            let name_index = shstrtab.len() as u32;
            shstrtab.extend(b".bss\0");
            Some(Elf32Shdr{
                name: name_index,
                etype: 8,
                flags: SHF_ALLOC | SHF_WRITE,
                addr: 0,
                offset: 0,
                size: bss_data.0.len() as u32,
                link: 0,
                info: 0,
                addralign: lsize_bssdata,
                entsize: 0,
            })
        }
        else {
            None
        }
    };
 
    let data_shdr = {
        if data_data != (Vec::new(), vec![]){
            elf_header.shnum += 1;
            elf_header.shoff += data_data.0.len() as u32;
            let lsize_data = calc_lsize(&data_data.1);
            let name_index = shstrtab.len() as u32;
            shstrtab.extend(b".data\0");
            Some(Elf32Shdr{
                name: name_index,
                etype: 1,
                flags: SHF_ALLOC | SHF_WRITE,
                addr: 0,
                offset: (size_of::<Elf32Ehdr>() + code.len()) as u32,
                size: data_data.0.len() as u32,
                link: 0,
                info: 0,
                addralign: lsize_data,
                entsize: 0,
            })
        }
        else {
            None
        }
    };
    
    // =====================
    //  .rel.* and .rela.* (for now only .rel.* and only for .text section)
    // =====================

    let (mut rel_text_shdr, mut rela_text_shdr) = (None, None);
    let (mut rel_text_symb, mut rela_text_symb) = (Vec::new(), Vec::new());
    let (mut rel_text_symbref, mut rela_text_symbref) = (Vec::new(), Vec::new());

    if !relocs.is_empty(){
        let mut relas = Vec::new();
        let mut rels  = Vec::new();
        for reloc in relocs{       
            if reloc.addend == 0 {
                rel_text_symbref.push(reloc.symbol.clone());
                rels.push(reloc);
            }
            else {
                rela_text_symbref.push(reloc.symbol.clone());
                relas.push(reloc);
            }
        }
        if !rels.is_empty(){
            elf_header.shnum += 1;
            rel_text_shdr = Some(Elf32Shdr{
                name        : shstrtab.len() as u32,
                etype       : 9,
                info        : 1,
                link        : 0,
                flags       : 0,
                addr        : 0,
                addralign   : 0,
                offset      : 0,
                size        : (size_of::<Elf32Rel>() * rels.len()) as u32,
                entsize     : 0,
            });
            shstrtab.extend(b".rel.text\0");
        }
        if !relas.is_empty(){
            elf_header.shnum += 1;
            rela_text_shdr = Some(Elf32Shdr{
                name: shstrtab.len() as u32,
                etype: 4,
                info: 1,
                link: 0,
                flags: 0,
                addr: 0,
                addralign: 0,
                offset: 0,
                size: (size_of::<Elf32Rela>() * relas.len()) as u32,
                entsize: 0,
            });
            shstrtab.extend(b".rela.text\0");
        }
        for rel in rels{
            rel_text_symb.push(Elf32Rel{
                offset: rel.offset,
                info  : rel.rtype.clone() as u32
            });
        }
        for rela in relas{
            rela_text_symb.push(Elf32Rela{
                offset: rela.offset,
                info  : rela.rtype.clone() as u32,
                addend: rela.addend
            });
        }
    }

    // =====================
    //  Elf section headers
    // =====================
    
    let null_shdr = Elf32Shdr{
        name:0,etype:0,flags:0,addr:0,offset:0,
        size:0,link:0,addralign:0,entsize:0,info:0
    };
    elf_header.shnum += 1;
    
    let text_shdr = Elf32Shdr{
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
    
    let mut symtab_hdr = Elf32Shdr{
        name        : shstrtab.len() as u32,
        etype       : 2,
        flags       : 0,
        offset      : 0, // to set - symbols
        addr        : 0,
        size        : 0,
        link        : 4,
        info        : 0,
        addralign   : 0,
        entsize     : 16
    };
    elf_header.shnum += 1;
    shstrtab.extend(b".symtab\0");
    
    let mut shstrtab_hdr = Elf32Shdr{
        name        : shstrtab.len() as u32,
        etype       : 3,
        flags       : 0,
        addr        : 0,
        offset      : 0, // to set - .shstrtab section
        size        : 0,
        link        : 0,
        info        : 0,
        addralign   : 1,
        entsize     : 0,
    };
    elf_header.shnum += 1;
    shstrtab.extend(b".shstrtab\0");

    // string table, but for symbols
    let mut strtab_hdr = Elf32Shdr{
        name        : shstrtab.len() as u32,
        etype       : 3,
        flags       : 0,
        addr        : 0,
        size        : 0, // to set - strtab size
        offset      : 0, // to set - strtab offset
        addralign   : 0,
        link        : 0,
        info        : 0,
        entsize     : 0,
    };
    elf_header.shnum += 1;
    shstrtab.extend(b".strtab\0");
    
    let mut data_section_index = 0;

    // symbols
    let mut strtab : Vec<UnsignedChar> = vec![0];
    let symbols : Vec<Elf32Sym> = {
        let mut elf_symbs = Vec::new();
        // null symbol
        elf_symbs.push(Elf32Sym{
            name:0,value:0,size:0,
            info:0,other:0,shndx:0
        });
        // file symbol
        elf_symbs.push(Elf32Sym{
            name    : strtab.len() as u32,
            value   : 0,
            size    : 0,
            info    : SType::File as u8,
            other   : 0,
            shndx   : 0xFFF1
        });
        strtab.extend(outpath.to_string_lossy().as_bytes());
        strtab.extend(b"\0");

        // text section symbol
        elf_symbs.push(Elf32Sym{
            name    : strtab.len() as u32,
            value   : 0,
            size    : 0,
            info    : SType::Section as u8,
            other   : 0,
            shndx   : 1
        });
        strtab.extend(b".text\0");
        let mut global_symbols = Vec::new();
        let plus = {
            let mut p = if let Some(_) = data_shdr{
                1
            } else {0};
            if let Some(_) = bss_shdr {
                p += 1
            } else {};
            p
        };
        for symbol in symbols{
            let elfsymbol = Elf32Sym{
                name: strtab.len() as u32,
                value: symbol.offset,
                size: if let Some(s) = symbol.size {s} else {0},
                info: ((symbol.visibility as u8) << 4) + symbol.stype as u8,
                other: 0,
                shndx: symbol.sindex + plus,
            }; 
            if symbol.visibility == Visibility::Global{
                global_symbols.push(elfsymbol);
            }
            else{
                elf_symbs.push(elfsymbol);
            }
            strtab.extend(symbol.name.as_bytes());
            strtab.extend(b"\0");
        }
        if let Some(_) = data_shdr{
            data_section_index = 2;
            for d in &data_data.1{
                elf_symbs.push(Elf32Sym{
                    name: strtab.len() as u32,
                    value: d.1,
                    size: d.2,
                    info: SType::Object as u8,
                    other: 0,
                    shndx: data_section_index,
                });
                strtab.extend(d.0.as_bytes());
                strtab.extend(b"\0");
            }
        }
        if let Some(_) = bss_shdr{
            if data_section_index == 0{
                data_section_index = 2;
            }
            else {
                data_section_index = 3;
            }
            for d in &bss_data.1{
                elf_symbs.push(Elf32Sym{
                    name: strtab.len() as u32,
                    value: d.1,
                    size: d.2,
                    info: SType::Object as u8,
                    other: 0,
                    shndx: data_section_index,
                });
                strtab.extend(d.0.as_bytes());
                strtab.extend(b"\0");
            }
        }
        symtab_hdr.info = elf_symbs.len() as u32;
        for symb in global_symbols {
            elf_symbs.push(symb);
        }
        elf_symbs
    };

    // offset to end of shdr's and start of symbols (.symtab), .shstrtab and .strtab
    let symbols_offset = 
        (size_of::<Elf32Ehdr>() + code.len() + data_data.0.len() + (elf_header.shnum as usize * size_of::<Elf32Shdr>())) as u32;
    let mut bytes = Vec::new();

    let modf = {
        let mut modf = 0;
        if let Some(_) = &data_shdr{
            modf += 1;
        }
        if let Some(_) = &bss_shdr{
            modf += 1;
        }
        modf
    };
    elf_header.shstrndx += modf;
    bytes.extend(elf_header.bytes());
    bytes.extend(code);
    if let Some(_) = data_shdr{
        bytes.extend(data_data.0);
    }
    bytes.extend(null_shdr.bytes());
    bytes.extend(text_shdr.bytes());

    if let Some(dsh) = data_shdr{
        bytes.extend(dsh.bytes());
        symtab_hdr.link += 1;
    }
    if let Some(bsh) = bss_shdr{
        bytes.extend(bsh.bytes());
        symtab_hdr.link += 1;
    }

    symtab_hdr.offset = symbols_offset;
    symtab_hdr.size = (symbols.len() * size_of::<Elf32Sym>()) as u32;
    bytes.extend(symtab_hdr.bytes());

    // .shstrtab section header
    shstrtab_hdr.offset = symbols_offset + (symbols.len() * size_of::<Elf32Sym>()) as u32;
    shstrtab_hdr.size   = shstrtab.len() as u32;
    bytes.extend(shstrtab_hdr.bytes());
    
    // .strtab section header
    strtab_hdr.offset = symbols_offset + (symbols.len() * size_of::<Elf32Sym>()) as u32 + shstrtab.len() as u32;
    strtab_hdr.size   = strtab.len() as u32;
    bytes.extend(strtab_hdr.bytes());

    if let Some(mut shdr) = rel_text_shdr{
        shdr.link = (modf + 2) as u32;
        shdr.offset = symbols_offset + (symbols.len() * size_of::<Elf32Sym>()) as u32 + shstrtab.len() as u32 + 
            strtab.len() as u32;
        bytes.extend(shdr.bytes());
    }
    if let Some(mut shdr) = rela_text_shdr{
        shdr.link = (modf + 2) as u32;
        // what is this
        shdr.offset = symbols_offset + (symbols.len() * size_of::<Elf32Sym>()) as u32 + shstrtab.len() as u32 + 
            strtab.len() as u32 + (rel_text_symb.len() * size_of::<Elf32Rel>()) as u32;
        bytes.extend(shdr.bytes());
    }

    // .symtab
    for s in &symbols{
        bytes.extend(s.bytes());
    }

    bytes.extend(shstrtab);
    bytes.extend(&strtab);
    if let Some(_) = rel_text_shdr{
        let mut index = 0;
        for mut rel in rel_text_symb{
            let mut symb_index = 0;
            for s in &symbols{
                let mut iter = strtab[s.name as usize..].iter();
                let mut s_name = String::new();
                while let Some(c) = iter.next(){
                    if c != &('\0' as u8){
                        s_name.push(*c as char);
                    }
                    else {
                        break;
                    }
                }
                if rel_text_symbref[index] == s_name{
                    rel.info = (symb_index << 8) as u32 + rel.info;
                    break;
                }
                symb_index += 1;
            }
            bytes.extend(rel.bytes());
            index += 1;
        }
    }
    if let Some(_) = rela_text_shdr{
        let mut index = 0;
        for mut rela in rela_text_symb{
            let mut symb_index = 0;
            for s in &symbols{
                let mut iter = strtab[s.name as usize..].iter();
                let mut s_name = String::new();
                while let Some(c) = iter.next(){
                    if c != &('\0' as u8){
                        s_name.push(*c as char);
                    }
                    else {
                        break;
                    }
                }
                if rel_text_symbref[index] == s_name{
                    rela.info += (symb_index << 8) as u32;
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
impl Elf32Shdr{
    pub fn bytes(&self) -> Vec<u8>{
        let mut bytes = Vec::new();
        bytes.extend(self.name       .to_le_bytes());
        bytes.extend(self.etype      .to_le_bytes());
        bytes.extend(self.flags      .to_le_bytes());
        bytes.extend(self.addr       .to_le_bytes());
        bytes.extend(self.offset     .to_le_bytes());
        bytes.extend(self.size       .to_le_bytes());
        bytes.extend(self.link       .to_le_bytes());
        bytes.extend(self.info       .to_le_bytes());
        bytes.extend(self.addralign  .to_le_bytes());
        bytes.extend(self.entsize    .to_le_bytes());
        bytes
    }
}

impl Elf32Ehdr{
    pub fn bytes(&self) -> Vec<u8>{
        let mut bytes = Vec::new();
        bytes.extend(self.ident);
        bytes.extend(self.etype       .to_le_bytes());
        bytes.extend(self.machine     .to_le_bytes());
        bytes.extend(self.version     .to_le_bytes());
        bytes.extend(self.entry       .to_le_bytes());
        bytes.extend(self.phoff       .to_le_bytes());
        bytes.extend(self.shoff       .to_le_bytes());
        bytes.extend(self.flags       .to_le_bytes());
        bytes.extend(self.ehsize      .to_le_bytes());
        bytes.extend(self.phentsize   .to_le_bytes());
        bytes.extend(self.phnum       .to_le_bytes());
        bytes.extend(self.shentsize   .to_le_bytes());
        bytes.extend(self.shnum       .to_le_bytes());
        bytes.extend(self.shstrndx    .to_le_bytes());
        bytes
    }
}

impl Elf32Sym{
    pub fn bytes(&self) -> Vec<u8>{
        let mut bytes = Vec::new();
        bytes.extend(self.name   .to_le_bytes());
        bytes.extend(self.value  .to_le_bytes());
        bytes.extend(self.size   .to_le_bytes());
        bytes.extend(self.info   .to_le_bytes());
        bytes.push(self.other);
        bytes.extend(self.shndx  .to_le_bytes());
        bytes
    }
}

impl Elf32Rel{
    pub fn bytes(&self) -> Vec<u8>{
        let mut bytes = Vec::new();
        bytes.extend(self.offset.to_le_bytes());
        bytes.extend(self.info  .to_le_bytes());
        bytes
    }
}
impl Elf32Rela{
    pub fn bytes(&self) -> Vec<u8>{
        let mut bytes = Vec::new();
        bytes.extend(self.offset.to_le_bytes());
        bytes.extend(self.info  .to_le_bytes());
        bytes.extend(self.addend.to_le_bytes());
        bytes
    }
}

fn calc_lsize(symbs: &[(String, u32, u32, Option<String>)]) -> u32{
    let mut iter   = symbs.iter();
    let mut lsize  = 1;
    let mut prvoff = 0;
    while let Some(i) = iter.next(){
        let tsize = i.1 - prvoff;
        if i.0.starts_with("str") {
            lsize = lsize.max(1);
        }
        else {
            if tsize % 4 == 0 {
                lsize = lsize.max(4);
            }
            else if tsize % 8 == 0 {
                lsize = lsize.max(8);
            }
        }
        prvoff = i.1;
    }
    return lsize;
}
