// rasmx86_64 - src/shr/symbol.rs
// ------------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::var::VarContent;

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum Visibility{
    #[default]
    Local = 0,
    Global = 1,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SymbolType{
    NoType = 0,
    Object = 1,
    Func   = 2,
    Section= 3,
    File   = 4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol{
    pub name: String,
    pub offset: u64,
    pub size: Option<u32>,
    pub sindex: u16,
    pub stype: SymbolType,
    pub visibility: Visibility,
    pub content: Option<VarContent>,
    pub addt: u8
}

impl Symbol{
    pub fn new(name: String, offset: u64, size: Option<u32>, sindex: u16, 
               stype: SymbolType, visibility: Visibility, content: Option<VarContent>) -> Self
    {
        Self{
            name,
            offset,
            size,
            sindex,
            stype,
            visibility,
            content,
            addt: 0
        }
    }
}
