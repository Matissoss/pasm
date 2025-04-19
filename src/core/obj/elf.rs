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
    core::reloc::{
        Relocation,
        //RType
    },
};

type Elf32Addr      = u32;
type Elf32Half      = u16;
type Elf32Off       = u32;
//type Elf32Sword     = i32;
type Elf32Word      = u32;
type UnsignedChar   = u8;

const EI_NIDENT     : usize     = 16;
//const SHF_WRITE     : Elf32Word = 0x1;
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

pub fn make_elf32(code: &[u8], _relocs: &[Relocation], symbols: &[Symbol], outpath: &PathBuf) -> Vec<u8>{
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
    let mut current_offset = size_of::<Elf32Ehdr>() as u32 + code.len() as u32;
    let mut shstrtab : Vec<UnsignedChar> = Vec::new();

    // =====================
    //  Elf section headers
    let null_shdr = Elf32Shdr{
        name:0,etype:0,flags:0,addr:0,offset:0,
        size:0,link:0,addralign:0,entsize:0,info:0
    };
    current_offset += size_of::<Elf32Shdr>() as u32;
    elf_header.shnum += 1;
    shstrtab.extend(b"\0");
    
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
    current_offset += size_of::<Elf32Shdr>() as u32;
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
    current_offset += size_of::<Elf32Shdr>() as u32;
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
    current_offset += size_of::<Elf32Shdr>() as u32;
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
    current_offset += size_of::<Elf32Shdr>() as u32;
    elf_header.shnum += 1;
    shstrtab.extend(b".strtab\0");
    

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
        for symbol in symbols{
            let elfsymbol = Elf32Sym{
                name: strtab.len() as u32,
                value: symbol.offset,
                size: if let Some(s) = symbol.size {s} else {0},
                info: ((symbol.visibility as u8) << 4) + symbol.stype as u8,
                other: 0,
                shndx: symbol.sindex,
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
        symtab_hdr.info = elf_symbs.len() as u32;
        for symb in global_symbols {
            elf_symbs.push(symb);
        }
        elf_symbs
    };

    let mut bytes = Vec::new();
    elf_header.shstrndx = 3;
    bytes.extend(elf_header.bytes());
    bytes.extend(code);
    bytes.extend(null_shdr.bytes());
    bytes.extend(text_shdr.bytes());
    symtab_hdr.offset = current_offset;
    symtab_hdr.size = (symbols.len() * size_of::<Elf32Sym>()) as u32;
    bytes.extend(symtab_hdr.bytes());
    current_offset += (symbols.len() * size_of::<Elf32Sym>()) as u32;
    shstrtab_hdr.offset = current_offset;
    shstrtab_hdr.size   = shstrtab.len() as u32;
    bytes.extend(shstrtab_hdr.bytes());
    current_offset += shstrtab.len() as u32;
    strtab_hdr.offset = current_offset;
    strtab_hdr.size   = strtab.len() as u32;
    bytes.extend(strtab_hdr.bytes());
    for s in symbols{
        bytes.extend(s.bytes());
    }
    bytes.extend(shstrtab);
    bytes.extend(strtab);
    bytes
}


// ====================================================
//
//                       Utils
//
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

#[allow(unused)]
fn calc_lsize(symbs: &[(String, u32)]) -> u32{
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
