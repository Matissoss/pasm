// rasmx86_64 - symbol.rs
// ----------------------
// made by matissoss
// licensed under MPL

use crate::shr::var::VarContent;

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Visibility{
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
    pub offset: u32,
    pub size: Option<u32>,
    pub sindex: u16,
    pub stype: SymbolType,
    pub visibility: Visibility,
    pub content: Option<VarContent>
}

impl Symbol{
    pub fn new(name: String, offset: u32, size: Option<u32>, sindex: u16, 
               stype: SymbolType, visibility: Visibility, content: Option<VarContent>) -> Self
    {
        Self{
            name,
            offset,
            size,
            sindex,
            stype,
            visibility,
            content
        }
    }
}
