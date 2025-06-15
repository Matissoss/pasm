// rasmx86_64 - src/shr/reloc.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{error::RASMError, symbol::Symbol};

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
    pub shidx: u16,
    pub reltype: RelType,
}

impl Relocation<'_> {
    pub fn lea(&self, addr: u32) -> u32 {
        // this might not work very well with larger numbers, so
        // later i might need to cast as i64/u64.
        if self.reltype == RelType::REL32 {
            // S + A - P
            (self.offset as i32 + self.addend) as u32 - addr
        } else {
            (self.offset as i32 + self.addend) as u32
        }
    }
}

pub fn relocate_addresses<'a>(
    buf: &mut [u8],
    rels: Vec<Relocation<'a>>,
    symbols: &'a [Symbol<'a>],
) -> Result<(), RASMError> {
    for rel in rels {
        relocate(buf, rel, symbols)?;
    }
    Ok(())
}

pub fn relocate<'a>(
    buf: &mut [u8],
    rel: Relocation<'a>,
    symbols: &'a [Symbol<'a>],
) -> Result<(), RASMError> {
    let symbol = if let Some(symbol) = symbols.iter().find(|e| e.name == rel.symbol) {
        symbol
    } else {
        return Err(RASMError::msg(
            "Tried to do relocation with non-existent symbol",
        ));
    };
    let addr = rel.lea(symbol.offset).to_le_bytes();
    let buf_offset = rel.offset as usize;

    if buf.len() < buf_offset {
        return Err(RASMError::msg("Tried to access field outside of buffer"));
    }
    let mut idx = 0;
    while idx < rel.reltype.size() {
        buf[buf_offset + idx] = addr[idx];
        idx += 1;
    }

    Ok(())
}

// i hope this works :)
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rel_test() {
        use crate::shr::symbol::{SymbolType, Visibility};
        // we assert here that Symbol is defined as second (idx 1)
        // byte.
        //                0     1     2     3     4     5     6     7
        let mut bytes = [0x00, 0x71, 0x00, 0x00, 0x00, 0x00, 0x81, 0x91];
        let symbol = Symbol {
            name: &"Symbol".to_string(),
            offset: 0x01,
            stype: SymbolType::NoType,
            size: 0,
            sindex: 0,
            visibility: Visibility::Local,
            is_extern: false,
        };
        let relocation = Relocation {
            symbol: &"Symbol".to_string(),
            offset: 0x02,
            addend: 0,
            reltype: RelType::REL32,
            shidx: 0,
        };
        assert_eq!(relocation.lea(0x01), 1);
        assert_eq!(relocate(&mut bytes, relocation, &[symbol.clone()]), Ok(()));
        assert_eq!(bytes, [0x00, 0x71, 0x01, 0x00, 0x00, 0x00, 0x81, 0x91]);
        let relocation = Relocation {
            symbol: &"Symbol".to_string(),
            offset: 0x03,
            addend: -1,
            reltype: RelType::REL32,
            shidx: 0,
        };
        assert_eq!(relocate(&mut bytes, relocation, &[symbol]), Ok(()));
        assert_eq!(bytes, [0x00, 0x71, 0x01, 0x01, 0x00, 0x00, 0x00, 0x91]);
    }
}
