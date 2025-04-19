// rasmx86_64 - reloc.rs
// ---------------------
// made by matissoss
// licensed under MPL

use std::str::FromStr;
use crate::shr::{
    symbol::Symbol,
    error::{
        RASMError,
        ExceptionType as ExType
    },
    num::Number
};

#[derive(Debug, PartialEq, Clone)]
pub enum RType{
    Rel32  , // relative 32-bit ; jmp's and call's
    PCRel32, // same as previous, but relative to RIP/EIP
    Abs64  , // absolute 64-bit ; global vars and pointers
    None   , 
}

// idk how to name it
#[derive(Debug, PartialEq, Clone)]
pub enum RCategory{
    Jump,
    Lea,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Relocation {
    pub symbol: String,
    pub rtype: RType,
    pub offset: u32,
    pub addend: i32,
    pub catg  : RCategory,
    pub size  : u8,
}

pub fn relocate_addresses(buf: &mut [u8], relocs: Vec<Relocation>, symbols: &[Symbol]) -> Option<Vec<RASMError>>{ 
    let mut errors = Vec::new();
    for reloc in relocs{
        if reloc.rtype == RType::PCRel32{
            if let Some(symbol) = find(symbols, &reloc.symbol){
                //  rel32       = symb_addr - (inst_addr + inst_size);
                let rel32       = (symbol.offset as i32) - ((reloc.offset + reloc.size as u32) as i32);
                let rel32_bytes = rel32.to_le_bytes(); 
                let mut tmp : usize = 0;
                let offs = reloc.offset;
                if reloc.catg == RCategory::Jump{
                    while tmp < rel32_bytes.len(){
                        buf[offs as usize + tmp] = rel32_bytes[tmp];
                        tmp += 1;
                    }
                }
                else {
                    if let Some(str) = &symbol.content{
                        let immbytes = match Number::from_str(&str){
                            Ok(n) => n.split_into_bytes(),
                            Err(_) => {
                                errors.push(RASMError::new(
                                    None,
                                    ExType::Error,
                                    Some(format!("Tried to use string - forbidden in `baremetal`")),
                                    None
                                ));
                                break
                            }
                        };
                        while tmp < immbytes.len(){
                            buf[offs as usize + tmp] = immbytes[tmp];
                            tmp += 1;
                        }
                    }
                    else {
                        errors.push(RASMError::new(
                            None,
                            ExType::Error,
                            Some(format!("Tried to use unitialized variable (`!bss` one)")),
                            Some(format!("Unitialized variables currently cannot be used in `baremetal` target"))
                        ));
                    }
                }
            }
            else {
                errors.push(RASMError::new(
                    None,
                    ExType::Error,
                    Some(format!("couldn't find symbol {} in current file", reloc.symbol)),
                    Some(format!("consider creating symbol like e.g: label or variable in .bss/.data/.rodata section"))
                ))
            }
        }
        else {
            errors.push(RASMError::new(
                None,
                ExType::Error,
                Some(format!("tried to use currently unsupported relocation type: {:?}", reloc.rtype)),
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
fn find<'a>(table: &'a [Symbol], object: &'a str) -> Option<&'a Symbol>{
    for e in table{
        if &e.name == object{
            return Some(e);
        }
    }
    return None;
}
