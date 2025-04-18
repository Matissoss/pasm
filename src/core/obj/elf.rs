// rasmx86_64 - elf.rs
// -------------------
// made by matissoss
// licensed under MPL

use crate::{
    shr::ast::Section,
    core::reloc::{
        Relocation,
        //RType
    }
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
    e_ident     : [UnsignedChar; EI_NIDENT],
    e_type      : Elf32Half,
    e_machine   : Elf32Half,
    e_version   : Elf32Word,
    e_entry     : Elf32Addr,
    e_phoff     : Elf32Off,
    e_shoff     : Elf32Off,
    e_flags     : Elf32Word,
    e_ehsize    : Elf32Half,
    e_phentsize : Elf32Half,
    e_phnum     : Elf32Half,
    e_shentsize : Elf32Half,
    e_shnum     : Elf32Half,
    e_shstrndx  : Elf32Half,
}

// section header
struct Elf32Shdr{
    sh_name: Elf32Word,
    sh_type: Elf32Word,
    sh_flags: Elf32Word,
    sh_addr : Elf32Addr,
    sh_offset : Elf32Off,
    sh_size: Elf32Word,
    sh_link: Elf32Word,
    sh_info: Elf32Word,
    sh_addralign: Elf32Word,
    sh_entsize: Elf32Word
}

type Sections = Vec<(Section, Vec<u8>, Vec<(String, u32, u32)>)>;
#[allow(unused)]
pub fn make_elf32(sections: Sections, code: &Vec<u8>, relocs: Vec<Relocation>) -> Vec<u8>{
    // elf class
    // 0 - invalid
    // 1 - 32-bit object
    // 2 - 64-bit object
    let ei_class = 1;
    // data encoding
    // 0 - invalid
    // 1 - little endian
    // 2 - big endian
    let ei_data = 2;
    // elf version
    // 0 - invalid
    // 1 - current
    let ei_version = 1;
    // os abi
    // 0 - System V
    // [...]
    // 3 - Linux
    // [...]
    let ei_osabi = 0;
    // os abi version
    let ei_osabiversion = 0;
    // elf type
    // 0 - unknown
    // 1 - relocatable (.o)
    // 2 - executable
    // 3 - shared object
    // 4 - core dump
    let e_type = 1;
    // machine
    // 0 - none
    // [...]
    // 3 - i386 (x86?)
    // [...]
    // 62 - x86_64
    let e_machine = 3;
    let mut elfhdr = Elf32Ehdr{
        e_ident: [0x7F, 'E' as u8, 'L' as u8, 'F' as u8, 
            ei_class, ei_data, ei_version, ei_osabi, ei_osabiversion, 
            0, 0, 0, 0, 0, 0, 0
        ],
        e_type,
        e_machine,
        e_version   : 1,
        e_entry     : 0, // to set
        e_phoff     : 0, // progam header offset
        e_shoff     : 0, // section header offset
        e_flags     : 0,
        e_ehsize    : size_of::<Elf32Ehdr>() as u16, // elf header size
        e_phentsize : 0,  // program header size
        e_phnum     : 0,  // program header entries
        e_shentsize : size_of::<Elf32Shdr>() as u16, // section header size
        e_shnum     : 0,  // section header entries
        e_shstrndx  : 2,  // section header string index?
    };

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

    let mut text_shdr = Elf32Shdr{
        sh_name: 1,
        sh_type: 1,
        sh_flags: SHF_EXECINSTR | SHF_ALLOC,
        sh_addr: 0,
        sh_offset: (size_of::<Elf32Ehdr>() + 1) as u32,
        sh_size: code.len() as u32,
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 16,
        sh_entsize: 0,
    };
    /*
    let lsize_bssdata = calc_lsize(&data_data.1);
    let mut bss_shdr = Elf32Shdr{
        sh_name: 0,
        sh_type: 8,
        sh_flags: SHF_ALLOC | SHF_WRITE,
        sh_addr: 0,
        sh_offset: 0,
        sh_size: bss_data.0.len() as u32,
        sh_link: 0,
        sh_info: 0,
        sh_addralign: lsize_bssdata,
        sh_entsize: 0,
    };
    let mut data_shdr = Elf32Shdr{
        sh_name: 0,
        sh_type: 1,
        sh_flags: SHF_ALLOC | SHF_WRITE,
        sh_addr: 0,
        sh_offset: 0,
        sh_size: data_data.0.len() as u32,
        sh_link: 0,
        sh_info: 0,
        sh_addralign: lsize_bssdata,
        sh_entsize: 0,
    };
    */

    let null_shdr = Elf32Shdr{
        sh_name: 0,
        sh_type: 0,
        sh_flags: 0,
        sh_addr: 0,
        sh_offset: 0,
        sh_size: 0,
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 0,
        sh_entsize: 0
    };

    let string_table : Vec<u8> = vec![
        0, '.' as u8, 't' as u8, 'e' as u8, 'x' as u8, 't' as u8, 0, '.' as u8, 's' as u8, 'h' as u8, 
        's' as u8, 't' as u8, 'r' as u8, 't' as u8, 'a' as u8, 'b' as u8, 0
    ];
    let mut string_table_hdr = Elf32Shdr{
        sh_name: 7,
        sh_type: 3,
        sh_flags: 0,
        sh_addr: 0,
        sh_offset: (52 + code.len() + 40 + 40 + 40) as u32,
        sh_size: string_table.len() as u32,
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 1,
        sh_entsize: 0,
    };

    elfhdr.e_shnum = 3;
    elfhdr.e_shoff = (size_of::<Elf32Ehdr>() + code.len()) as u32;

    let mut bytes = elfhdr.bytes();
    bytes.extend(code);
    bytes.extend(null_shdr.bytes());
    bytes.extend(text_shdr.bytes());
    bytes.extend(string_table_hdr.bytes());
    bytes.extend(string_table);
    bytes
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


impl Elf32Shdr{
    pub fn bytes(&self) -> Vec<u8>{
        let mut bytes = Vec::new();
        bytes.extend(self.sh_name.to_be_bytes());
        bytes.extend(self.sh_type.to_be_bytes());
        bytes.extend(self.sh_flags.to_be_bytes());
        bytes.extend(self.sh_addr.to_be_bytes());
        bytes.extend(self.sh_offset.to_be_bytes());
        bytes.extend(self.sh_size.to_be_bytes());
        bytes.extend(self.sh_link.to_be_bytes());
        bytes.extend(self.sh_info.to_be_bytes());
        bytes.extend(self.sh_addralign.to_be_bytes());
        bytes.extend(self.sh_entsize.to_be_bytes());
        bytes
    }
}

impl Elf32Ehdr{
    pub fn bytes(&self) -> Vec<u8>{
        let mut bytes = Vec::new();
        bytes.extend(self.e_ident);
        bytes.extend(self.e_type.to_be_bytes());
        bytes.extend(self.e_machine.to_be_bytes());
        bytes.extend(self.e_version.to_be_bytes());
        bytes.extend(self.e_entry.to_be_bytes());
        bytes.extend(self.e_phoff.to_be_bytes());
        bytes.extend(self.e_shoff.to_be_bytes());
        bytes.extend(self.e_flags.to_be_bytes());
        bytes.extend(self.e_ehsize.to_be_bytes());
        bytes.extend(self.e_phentsize.to_be_bytes());
        bytes.extend(self.e_phnum.to_be_bytes());
        bytes.extend(self.e_shentsize.to_be_bytes());
        bytes.extend(self.e_shnum.to_be_bytes());
        bytes.extend(self.e_shstrndx.to_be_bytes());
        bytes
    }
}
