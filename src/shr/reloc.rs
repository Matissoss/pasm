// pasm - src/shr/reloc.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{error::Error, symbol::Symbol};

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

impl std::str::FromStr for RelType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "abs32" => Ok(Self::ABS32),
            "rel32" => Ok(Self::REL32),
            "rel16" => Ok(Self::REL16),
            "rel8" => Ok(Self::REL8),
            _ => Err(()),
        }
    }
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
pub struct Relocation<'a> {
    pub symbol: &'a str,
    pub offset: usize,
    pub addend: i32,
    pub shidx: u16,
    pub reltype: RelType,
}

impl Relocation<'_> {
    pub const fn is_rel(&self) -> bool {
        self.reltype.is_rel()
    }
    pub const fn size(&self) -> usize {
        self.reltype.size()
    }
    pub fn lea(&self, addr: usize) -> usize {
        let addend: i64 = self.addend.into();
        // we can cast offset as i64, because it would be currently impossible (?) to utilize 63-bit address (?)
        let offset: i64 = self.offset as i64;
        if self.is_rel() {
            // S + A - P
            //
            // very important thing (because i forgor earlier):
            // S = Symbol, A = Addend, P = Offset
            //                NOT
            // S = Offset, A = Addend, P = Symbol
            (addr as i64 + addend - offset) as usize
        } else {
            (offset + addend).abs_diff(0) as usize
        }
    }
}

pub fn relocate_addresses(
    buf: &mut [u8],
    rels: Vec<Relocation>,
    symbols: &[Symbol],
) -> Result<(), Error> {
    for rel in rels {
        relocate(buf, rel, symbols)?;
    }
    Ok(())
}

pub fn relocate(buf: &mut [u8], rel: Relocation, symbols: &[Symbol]) -> Result<(), Error> {
    let symbol = if let Some(symbol) = symbols.iter().find(|e| e.name == rel.symbol) {
        symbol
    } else {
        return Err(Error::new(
            format!(
                "you tried to use relocation on undeclared symbol \"{}\"",
                rel.symbol
            ),
            4,
        ));
    };
    let addr = rel.lea(symbol.offset);

    let addr = addr.to_le_bytes();
    let buf_offset = rel.offset;

    if buf.len() + rel.size() < buf_offset {
        return Err(Error::new(
            "src/shr/rel.rs: tried to perform relocation, but we tried to write out of bounds",
            500,
        ));
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
        use crate::shr::symbol::SymbolType;
        use crate::shr::visibility::Visibility;
        // we assert here that Symbol is defined as second (idx 1)
        // byte.
        //                          0     1     2     3     4     5     6     7
        let mut bytes: [u8; 8] = [0x00, 0x71, 0x00, 0x00, 0x00, 0x00, 0x81, 0x91];
        let symbol = Symbol {
            name: "Symbol",
            offset: 0x01,
            stype: SymbolType::NoType,
            size: 0,
            sindex: 0,
            visibility: Visibility::Local,
            valid: true,
        };
        let relocation = Relocation {
            symbol: "Symbol",
            offset: 0x02,
            addend: 0,
            reltype: RelType::REL32,
            shidx: 0,
        };
        assert_eq!(relocation.lea(0x01), (-1i64) as usize);
        assert_eq!(relocate(&mut bytes, relocation, &[symbol.clone()]), Ok(()));
        //                                       -1
        //                              +-----+--++--+----+
        //                              |     |      |    |
        assert_eq!(bytes, [0x00, 0x71, 0xFF, 0xFF, 0xFF, 0xFF, 0x81, 0x91]);
        let relocation = Relocation {
            symbol: "Symbol",
            offset: 0x03,
            addend: -1,
            reltype: RelType::REL32,
            shidx: 0,
        };
        assert_eq!(relocate(&mut bytes, relocation, &[symbol]), Ok(()));
        assert_eq!(bytes, [0x00, 0x71, 0xFF, 0xFD, 0xFF, 0xFF, 0xFF, 0x91]);
    }
}
