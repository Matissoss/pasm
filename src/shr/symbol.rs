// pasm - src/shr/symbol.rs
// ------------------------
// made by matissoss
// licensed under MPL 2.0

use std::str::FromStr;

use crate::{
    shr::{booltable::BoolTable8, num::Number, reloc::RelType, size::Size, visibility::Visibility},
    utils::split_once_intelligent,
};

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum SymbolType {
    #[default]
    NoType = 0,
    Object = 1,
    Func = 2,
    Section = 3,
    File = 4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol<'a> {
    pub name: &'a str,
    pub offset: usize,
    pub size: usize,
    pub sindex: u16,
    pub visibility: Visibility,
    pub stype: SymbolType,
}

impl Symbol<'_> {
    pub fn is_global(&self) -> bool {
        self.visibility == Visibility::Public || self.visibility == Visibility::Weak
    }
}

const SIZE_GUARDIAN: u8 = 0x0;
const RELT_GUARDIAN: u8 = 0x1;
const ADED_GUARDIAN: u8 = 0x2;
const IS_DEREF: u8 = 0x3;

#[derive(PartialEq, Clone, Debug)]
pub struct SymbolRef<'a> {
    pub symbol: &'a str,
    addend: i32,
    size: Size,
    reltype: RelType,
    guardians: BoolTable8,
}

impl Default for SymbolRef<'_> {
    fn default() -> Self {
        Self {
            symbol: "",
            addend: 0,
            size: Size::Unknown,
            reltype: RelType::REL32,
            guardians: BoolTable8::new(),
        }
    }
}

impl<'a> SymbolRef<'a> {
    // because rust's trait implementation does not allow for this kind of stuff
    // (lifetime in `s: &'a str`)
    //
    /// expected input: in format @[<SYMBOL REF>] already without any semicolons or other trash.
    #[allow(clippy::result_unit_err)]
    pub fn from_str(s: &'a str) -> Result<SymbolRef<'a>, ()> {
        if let Some(s) = s.strip_prefix("@[") {
            if let Some(s) = s.strip_suffix("]") {
                let mut symbolref = SymbolRef::default();
                let mut symbol_str = s;
                while let Some((str, rest)) = split_once_intelligent(symbol_str, ',') {
                    if let Ok(reltype) = RelType::from_str(str) {
                        symbolref.set_reltype(reltype);
                    } else if let Ok(n) = Number::from_str(str) {
                        symbolref.set_addend(n.get_as_i32());
                    } else if symbolref.symbol.is_empty() {
                        symbolref.symbol = str;
                    } else {
                        // because we already have the referenced symbol
                        // TODO: add support for specyfying section
                        return Err(());
                    }

                    symbol_str = rest;
                }
                if let Ok(reltype) = RelType::from_str(symbol_str) {
                    symbolref.set_reltype(reltype);
                } else if let Ok(n) = Number::from_str(symbol_str) {
                    symbolref.set_addend(n.get_as_i32());
                } else if symbolref.symbol.is_empty() {
                    symbolref.symbol = symbol_str;
                } else {
                    // because we already have the referenced symbol
                    // TODO: add support for specyfying section
                    return Err(());
                }

                Ok(symbolref)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
    pub fn deref(&mut self, bool: bool) {
        self.guardians.set(IS_DEREF, bool)
    }
    pub fn new(
        symb: &'a str,
        addend: Option<i32>,
        is_deref: bool,
        sz: Option<Size>,
        reltype: Option<RelType>,
    ) -> Self {
        Self {
            symbol: symb,
            addend: addend.unwrap_or(0),
            size: sz.unwrap_or(Size::Unknown),
            reltype: reltype.unwrap_or(RelType::REL32),
            guardians: BoolTable8::new()
                .setc(IS_DEREF, is_deref)
                .setc(ADED_GUARDIAN, addend.is_some())
                .setc(SIZE_GUARDIAN, sz.is_some())
                .setc(RELT_GUARDIAN, reltype.is_some()),
        }
    }
    pub fn is_deref(&self) -> bool {
        self.guardians.get(IS_DEREF).unwrap()
    }
    pub fn set_size(&mut self, sz: Size) {
        self.guardians.set(SIZE_GUARDIAN, true);
        self.size = sz;
    }
    pub fn size(&self) -> Option<Size> {
        if self.guardians.get(SIZE_GUARDIAN).unwrap() {
            Some(self.size)
        } else {
            None
        }
    }
    pub fn reltype(&self) -> Option<RelType> {
        if self.guardians.get(RELT_GUARDIAN).unwrap() {
            Some(self.reltype)
        } else {
            None
        }
    }
    pub fn set_reltype(&mut self, reltype: RelType) {
        self.guardians.set(RELT_GUARDIAN, true);
        self.reltype = reltype;
    }
    pub fn set_addend(&mut self, addend: i32) {
        self.guardians.set(ADED_GUARDIAN, true);
        self.addend = addend;
    }
    pub fn addend(&self) -> Option<i32> {
        if self.guardians.get(ADED_GUARDIAN).unwrap() {
            Some(self.addend)
        } else {
            None
        }
    }
}

impl ToString for SymbolRef<'_> {
    fn to_string(&self) -> String {
        let mut string = self.symbol.to_string();
        string.push(':');
        string.push_str(if self.reltype().unwrap_or(RelType::REL32).is_rel() {
            "rel"
        } else {
            "abs"
        });
        if self.addend != 0 {
            string.push(':');
            string.push_str(&self.addend.to_string());
        }
        string
    }
}
