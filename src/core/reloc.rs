// rasmx86_64 - src/core/reloc.rs
// ------------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;
use crate::shr::{
    symbol::Symbol,
    error::RASMError,
    var::VarContent
};

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RType{
    S32       = 11,
    PCRel32   = 2, // relative 32-bit ; jmp's and call's
    Abs64     = 1,   // absolute 64-bit ; global vars and pointers
    None      = 0, 
}

// idk how to name it
#[derive(Debug, PartialEq, Clone)]
pub enum RCategory{
    Jump,
    Lea,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Relocation<'a> {
    pub symbol: Cow<'a, &'a String>,
    pub rtype : RType,
    pub offset: u64,
    pub addend: i32,
    pub catg  : RCategory,
    pub size  : u8,
}

pub fn relocate_addresses(buf: &mut [u8], relocs: Vec<Relocation>, symbols: &[Symbol]) -> Option<Vec<RASMError>>{ 
    let mut errors = Vec::new();
    for reloc in relocs{
        if reloc.rtype == RType::PCRel32{
            if let Some(symbol) = find(symbols, &reloc.symbol){
                //  rel32       = S + A - P
                let rel32       = (symbol.offset as i32) + (symbol.addend as i32) - ((reloc.offset + reloc.size as u64) as i32);
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
                    if let Some(con) = &symbol.content{
                        let immbytes = match con{
                            VarContent::Number(n) => n.split_into_bytes(),
                            VarContent::String(_) => {
                                errors.push(RASMError::new(
                                    None,
                                    Some(format!("Tried to use string - forbidden in `baremetal`")),
                                    None
                                ));
                                break;
                            },
                            VarContent::Uninit => {
                                errors.push(RASMError::new(
                                    None,
                                    Some(format!("Tried to use uninitialized variable - forbidden in `baremetal`")),
                                    None
                                ));
                                break;
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
                            Some(format!("Tried to use unitialized variable (`!bss` one)")),
                            Some(format!("Unitialized variables currently cannot be used in `baremetal` target"))
                        ));
                    }
                }
            }
            else {
                errors.push(RASMError::new(
                    None,
                    Some(format!("couldn't find symbol {} in current file", reloc.symbol)),
                    Some(format!("consider creating symbol like e.g: label or variable in .bss/.data/.rodata section"))
                ))
            }
        }
        else if let RType::S32 = reloc.rtype{
            if let Some(symbol) = find(&symbols, &reloc.symbol){
                let _s32 = symbol.offset;
                if let Some(con) = &symbol.content{
                    let immbytes = match con{
                        VarContent::Number(n) => n.split_into_bytes(),
                        VarContent::String(_) => {
                            errors.push(RASMError::new(
                                None,
                                Some(format!("Tried to use string - forbidden in `baremetal`")),
                                None
                            ));
                            break;
                        },
                        VarContent::Uninit => {
                            errors.push(RASMError::new(
                                None,
                                Some(format!("Tried to use uninitialized variable - forbidden in `baremetal`")),
                                None
                            ));
                            break;
                        }
                    };
                    let mut tmp = 0;
                    while tmp < immbytes.len(){
                        buf[reloc.offset as usize + tmp] = immbytes[tmp];
                        tmp += 1;
                    }
                }
                else {
                    errors.push(RASMError::new(
                        None,
                        Some(format!("Tried to use unitialized variable (`!bss` one)")),
                        Some(format!("Unitialized variables currently cannot be used in `baremetal` target"))
                    ));
                }
            }
            else {
                errors.push(RASMError::new(
                    None,
                    Some(format!("couldn't find symbol {} in current file", reloc.symbol)),
                    Some(format!("consider creating symbol like e.g: label or variable in .bss/.data/.rodata section"))
                ))
            }
        }
        else {
            errors.push(RASMError::new(
                None,
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
fn find<'a>(table: &'a [Symbol], object: &'a str) -> Option<&'a Symbol<'a>>{
    for e in table{
        if e.name == Cow::Borrowed(object){
            return Some(e);
        }
    }
    return None;
}
