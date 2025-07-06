// pasm - src/shr/label.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{ast::Instruction, symbol::SymbolType, visibility::Visibility};

#[derive(Default, Debug, Clone, PartialEq)]
#[repr(C)]
pub struct Label<'a> {
    pub content: Vec<Instruction<'a>>,
    pub name: &'a str,
    pub offset: usize,
    pub size: usize,
    pub align: u16,
    pub attributes: LabelAttributes,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct LabelAttributes {
    pub symbol_type: SymbolType,
    pub visibility: Visibility,
    pub bits: u8,
    pub _reserved: u8,
}

// we make manual getters and setters, because i might
// optimize this struct through bit packing
impl LabelAttributes {
    pub const fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }
    pub const fn get_visibility(&self) -> Visibility {
        self.visibility
    }
    pub const fn set_symbol_type(&mut self, symbol: SymbolType) {
        self.symbol_type = symbol;
    }
    pub const fn get_symbol_type(&self) -> SymbolType {
        self.symbol_type
    }
    pub const fn set_bits(&mut self, bits: u8) {
        self.bits = bits;
    }
    pub const fn get_bits(&self) -> u8 {
        self.bits
    }
}
