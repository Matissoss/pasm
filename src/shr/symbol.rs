// pasm - src/shr/symbol.rs
// ------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{booltable::BoolTable8, reloc::RelType, size::Size, visibility::Visibility};

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
pub struct Symbol {
    pub name: crate::RString,
    pub offset: u32,
    pub size: u32,
    pub sindex: u16,
    pub visibility: Visibility,
    pub stype: SymbolType,
}

impl Symbol {
    pub fn is_global(&self) -> bool {
        self.visibility == Visibility::Public || self.visibility == Visibility::Weak
    }
}

const SIZE_GUARDIAN: u8 = 0x0;
const RELT_GUARDIAN: u8 = 0x1;
const ADED_GUARDIAN: u8 = 0x2;
const IS_DEREF: u8 = 0x3;

#[derive(PartialEq, Clone, Debug)]
pub struct SymbolRef {
    pub symbol: crate::RString,
    addend: i32,
    size: Size,
    reltype: RelType,
    guardians: BoolTable8,
}

impl SymbolRef {
    pub fn deref(&mut self, bool: bool) {
        self.guardians.set(IS_DEREF, bool)
    }
    pub fn new(
        symb: crate::RString,
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

impl ToString for SymbolRef {
    fn to_string(&self) -> String {
        let mut string = self.symbol.clone().to_string();
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
