// rasmx86_64 - reloc.rs
// ---------------------
// made by matissoss
// licensed under MPL

use crate::shr::error::{
    RASMError,
    ExceptionType as ExType
};

#[derive(Debug, PartialEq, Clone)]
pub enum RType{
    Rel32  , // relative 32-bit ; jmp's and call's
    PCRel32, // same as previous, but relative to RIP/EIP
    Abs64  , // absolute 64-bit ; global vars and pointers
    None   , 
}

#[derive(PartialEq, Debug, Clone)]
pub struct Relocation {
    pub r_type: RType,
    pub symbol: String,
    pub offset: u32,
    pub addend: i32,
    pub size  : u8,
}

// currently without support for constants that would be found in .bss/.data section :(
pub fn relocate_addresses(buf: &mut [u8], relocs: Vec<Relocation>, symbols: &[(String, u32, u32)]) -> Option<Vec<RASMError>>{
    let mut errors = Vec::new();
    for reloc in relocs{
        if reloc.r_type == RType::PCRel32{
            if let Some(offset) = find(symbols, &reloc.symbol){
                //  rel32       = symb_addr - (inst_addr + inst_size);
                let rel32       = (offset as i32) - ((reloc.offset + reloc.size as u32) as i32);
                let rel32_bytes = rel32.to_le_bytes(); 
                let mut tmp : usize = 0;
                let offs = reloc.offset;
                while tmp < rel32_bytes.len(){
                    buf[offs as usize + tmp] = rel32_bytes[tmp];
                    tmp += 1;
                }
            }
            else {
                errors.push(RASMError::new(
                    None,
                    ExType::Error,
                    None,
                    Some(format!("couldn't find symbol {} in current file", reloc.symbol)),
                    Some(format!("consider creating symbol like e.g: label or variable in .bss/.data/.rodata section"))
                ))
            }
        }
        else {
            errors.push(RASMError::new(
                None,
                ExType::Error,
                None,
                Some(format!("tried to use currently unsupported relocation type: {:?}", reloc.r_type)),
                None
            ))
        }
    }
    if errors.is_empty(){
        return None;
    }
    else {
        return Some(errors);
    }
}

#[inline]
fn find(table: &[(String, u32, u32)], object: &str) -> Option<u32>{
    for e in table{
        if &e.0 == object{
            return Some(e.1);
        }
    }
    return None;
}
