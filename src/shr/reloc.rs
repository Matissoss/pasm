// rasmx86_64 - src/shr/reloc.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    error::RASMError,
    symbol::{Symbol, VarContent},
};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RType {
    S32,
    PCRel32, // relative 32-bit ; jmp's and call's
    Abs64,   // absolute 64-bit ; global vars and pointers
    None,
}

impl RType {
    pub fn to_elf64_rtype(&self) -> u64 {
        match self {
            Self::S32 => 11,
            Self::PCRel32 => 2,
            Self::Abs64 => 1,
            Self::None => 0,
        }
    }
    pub fn to_elf32_rtype(&self) -> u32 {
        match self {
            Self::S32 => 1,
            Self::PCRel32 => 2,
            Self::Abs64 => 1,
            Self::None => 0,
        }
    }
}

// idk how to name it
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RCategory {
    Jump,
    Lea,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Relocation<'a> {
    pub symbol: Cow<'a, &'a String>,
    pub rtype: RType,
    pub offset: u64,
    pub addend: i32,
    pub catg: RCategory,
    pub size: u8,
}

pub fn relocate_addresses<'a>(
    buf: &mut [u8],
    relocs: Vec<Relocation<'a>>,
    symbols: &'a [Symbol<'a>],
) -> Option<Vec<RASMError>> {
    let mut errors = Vec::new();
    for reloc in relocs {
        if reloc.rtype == RType::PCRel32 {
            if let Some(symbol) = find(symbols, &reloc.symbol) {
                //  rel32       = S + A - P
                let rel32 = (symbol.offset as i32) + (symbol.addend as i32)
                    - ((reloc.offset + reloc.size as u64) as i32);
                let rel32_bytes = rel32.to_le_bytes();
                let mut tmp: usize = 0;
                let offs = reloc.offset;

                #[allow(clippy::collapsible_else_if)]
                if reloc.catg == RCategory::Jump {
                    while tmp < rel32_bytes.len() {
                        buf[offs as usize + tmp] = rel32_bytes[tmp];
                        tmp += 1;
                    }
                } else {
                    if let Some(con) = &symbol.content {
                        let immbytes = match **con {
                            VarContent::Number(n) => n.split_into_bytes(),
                            VarContent::String(_) => {
                                errors.push(RASMError::no_tip(
                                    None,
                                    Some("Tried to use string - forbidden in `bin`"),
                                ));
                                break;
                            }
                            VarContent::Uninit => {
                                errors.push(RASMError::no_tip(
                                    None,
                                    Some(
                                        "Tried to use uninitialized variable - forbidden in `bin`",
                                    ),
                                ));
                                break;
                            }
                        };
                        while tmp < immbytes.len() {
                            buf[offs as usize + tmp] = immbytes[tmp];
                            tmp += 1;
                        }
                    } else {
                        errors.push(RASMError::with_tip(
                            None,
                            Some("Tried to use unitialized variable (`!bss` one)"),
                            Some("Unitialized variables currently cannot be used in `baremetal` target")
                        ));
                    }
                }
            } else {
                errors.push(RASMError::with_tip(
                    None,
                    Some(format!("couldn't find symbol {} in current file", reloc.symbol)),
                    Some("consider creating symbol like e.g: label or variable in .bss/.data/.rodata section")
                ))
            }
        } else if let RType::S32 = reloc.rtype {
            if let Some(symbol) = find(symbols, &reloc.symbol) {
                let _s32 = symbol.offset;
                if let Some(con) = &symbol.content {
                    let immbytes = match **con {
                        VarContent::Number(n) => n.split_into_bytes(),
                        VarContent::String(_) => {
                            errors.push(RASMError::no_tip(
                                None,
                                Some("Tried to use string - forbidden in `bin`"),
                            ));
                            break;
                        }
                        VarContent::Uninit => {
                            errors.push(RASMError::no_tip(
                                None,
                                Some("Tried to use uninitialized variable - forbidden in `bin`"),
                            ));
                            break;
                        }
                    };
                    let mut tmp = 0;
                    while tmp < immbytes.len() {
                        buf[reloc.offset as usize + tmp] = immbytes[tmp];
                        tmp += 1;
                    }
                } else {
                    errors.push(RASMError::with_tip(
                        None,
                        Some("Tried to use unitialized variable (`!bss` one)"),
                        Some(
                            "Unitialized variables currently cannot be used in `baremetal` target",
                        ),
                    ));
                }
            } else {
                errors.push(RASMError::with_tip(
                    None,
                    Some(format!("Couldn't find symbol {} in current file", reloc.symbol)),
                    Some("Consider creating symbol like e.g: label or variable in .bss/.data/.rodata section")
                ))
            }
        } else {
            errors.push(RASMError::no_tip(
                None,
                Some(format!(
                    "Tried to use currently unsupported relocation type: {:?}",
                    reloc.rtype
                )),
            ))
        }
    }
    if errors.is_empty() {
        None
    } else {
        Some(errors)
    }
}

#[allow(clippy::manual_find)]
#[inline]
fn find<'a>(table: &'a [Symbol<'a>], object: &'a str) -> Option<&'a Symbol<'a>> {
    for e in table {
        if e.name == Cow::Borrowed(object) {
            return Some(e);
        }
    }
    None
}
