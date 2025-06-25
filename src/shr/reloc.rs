// pasm - src/shr/reloc.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{error::RASMError, symbol::Symbol};

impl RelType {
    pub fn to_elf64_rtype(&self) -> u64 {
        match self {
            Self::ABS32 => 11,
            Self::REL32 => 2,
            Self::REL16 => 13,
            Self::REL8 => 15,
        }
    }
    pub fn to_elf32_rtype(&self) -> u32 {
        match self {
            Self::ABS32 => 1,
            Self::REL32 => 2,
            Self::REL16 => 21,
            Self::REL8 => 23,
        }
    }
}

#[derive(PartialEq, Default, Clone, Debug, Copy)]
pub enum RelType {
    ABS32,
    #[default]
    REL32,
    REL16,
    REL8,
}

impl RelType {
    pub const fn size(&self) -> usize {
        match self {
            Self::ABS32 => 4,
            Self::REL32 => 4,
            Self::REL16 => 2,
            Self::REL8 => 1,
        }
    }
    pub const fn is_rel(&self) -> bool {
        !matches!(self, Self::ABS32)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Relocation {
    pub symbol: crate::RString,
    pub offset: u32,
    pub addend: i32,
    pub shidx: u16,
    pub reltype: RelType,
}

impl Relocation {
    pub const fn is_rel(&self) -> bool {
        self.reltype.is_rel()
    }
    pub const fn size(&self) -> usize {
        self.reltype.size()
    }
    pub fn lea(&self, addr: u32) -> u32 {
        // this might not work very well with larger numbers, so
        // later i might need to cast as i64/u64.
        let addend: i64 = self.addend.into();
        let offset: i64 = self.offset.into();
        if self.is_rel() {
            // S + A - P
            (offset + addend - <u32 as Into<i64>>::into(addr)) as u32
        } else {
            (offset + addend).abs_diff(0) as u32
        }
    }
}

pub fn relocate_addresses(
    buf: &mut [u8],
    rels: Vec<Relocation>,
    symbols: &[Symbol],
) -> Result<(), RASMError> {
    for rel in rels {
        relocate(buf, rel, symbols)?;
    }
    Ok(())
}

pub fn relocate(buf: &mut [u8], rel: Relocation, symbols: &[Symbol]) -> Result<(), RASMError> {
    let symbol = if let Some(symbol) = symbols.iter().find(|e| e.name == rel.symbol) {
        symbol
    } else {
        return Err(RASMError::msg(
            "Tried to do relocation with non-existent symbol",
        ));
    };
    let addr = rel.lea(symbol.offset);

    let rs = match rel.reltype.size() {
        1 => u8::MAX as u32,
        2 => u16::MAX as u32,
        _ => u32::MAX,
    };
    if addr > rs {
        return Err(RASMError::msg(format!(
            "Tried to perform {}-bit relocation on too large address",
            rs << 3
        )));
    }
    let addr = addr.to_le_bytes();
    let buf_offset = rel.offset as usize;

    if buf.len() + rel.size() < buf_offset {
        return Err(RASMError::msg("Tried to access field outside of buffer"));
    }
    let mut idx = 0;
    while idx < rel.size() {
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
            name: "Symbol".to_string().into(),
            offset: 0x01,
            stype: SymbolType::NoType,
            size: 0,
            sindex: 0,
            visibility: Visibility::Local,
            is_extern: false,
        };
        let relocation = Relocation {
            symbol: "Symbol".to_string().into(),
            offset: 0x02,
            addend: 0,
            reltype: RelType::REL32,
            shidx: 0,
        };
        assert_eq!(relocation.lea(0x01), 1);
        assert_eq!(relocate(&mut bytes, relocation, &[symbol.clone()]), Ok(()));
        assert_eq!(bytes, [0x00, 0x71, 0x01, 0x00, 0x00, 0x00, 0x81, 0x91]);
        let relocation = Relocation {
            symbol: "Symbol".to_string().into(),
            offset: 0x03,
            addend: -1,
            reltype: RelType::REL32,
            shidx: 0,
        };
        assert_eq!(relocate(&mut bytes, relocation, &[symbol]), Ok(()));
        assert_eq!(bytes, [0x00, 0x71, 0x01, 0x01, 0x00, 0x00, 0x00, 0x91]);
    }
}
