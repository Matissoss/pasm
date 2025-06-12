// rasmx86_64 - src/shr/reloc.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    error::RASMError,
    symbol::Symbol,
};

impl RelType {
    pub fn to_elf64_rtype(&self) -> u64 {
        match self {
            Self::ABS32 => 11,
            Self::REL32 => 2,
        }
    }
    pub fn to_elf32_rtype(&self) -> u32 {
        match self {
            Self::ABS32 => 1,
            Self::REL32 => 2,
        }
    }
}

pub fn relocate_addresses<'a>(
    buf: &mut [u8],
    relocs: Vec<Relocation>,
    symbols: &'a [Symbol<'a>],
) -> Option<Vec<RASMError>> {
    let mut errors = Vec::new();
    for reloc in relocs {
        if reloc.reltype == RelType::REL32 {
            if let Some(symbol) = symbols.iter().find(|&e| e.name == reloc.symbol) {
                //  rel32       = S + A - P
                let rel32 = (symbol.offset as i32)
                    - ((reloc.offset + reloc.reltype.size() as u32) as i32);
                let rel32_bytes = rel32.to_le_bytes();
                let mut tmp: usize = 0;
                let offs = reloc.offset;

                while tmp < rel32_bytes.len() {
                    buf[offs as usize + tmp] = rel32_bytes[tmp];
                    tmp += 1;
                }
            } else {
                errors.push(RASMError::with_tip(
                    None,
                    Some(format!("couldn't find symbol {} in current file", reloc.symbol)),
                    Some("consider creating symbol like e.g: label or variable in .bss/.data/.rodata section")
                ))
            }
        } else if let RelType::ABS32 = reloc.reltype {
            if let Some(symbol) = symbols.iter().find(|&e| e.name == reloc.symbol) {
                let s32 = symbol.offset.to_le_bytes();
                let offs = reloc.offset;
                let mut tmp = 0;

                while tmp < s32.len() {
                    buf[offs as usize + tmp] = s32[tmp];
                    tmp += 1;
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
                    reloc.reltype
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

// new relocation types
#[derive(PartialEq, Clone, Debug, Copy)]
pub enum RelType {
    ABS32,
    REL32,
}

impl RelType {
    pub const fn size(&self) -> usize {
        match self {
            Self::ABS32 => 4,
            Self::REL32 => 4,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Relocation<'a> {
    pub symbol: &'a String,
    pub offset: u32,
    pub addend: i32,
    pub reltype: RelType, 
}
