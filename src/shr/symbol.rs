// rasmx86_64 - src/shr/symbol.rs
// ------------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;

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
pub struct Symbol<'a>{
    pub name: Cow<'a, String>,
    pub offset: u64,
    pub size: Option<u32>,
    pub sindex: u16,
    pub stype: SymbolType,
    pub visibility: Visibility,
    pub content: Option<VarContent<'a>>,
    pub addend : i64,
    pub addt: u8
}

impl<'b> Symbol<'b>{
    pub fn new(name: Cow<'b, String>, offset: u64, addend: i64, size: Option<u32>, sindex: u16, 
               stype: SymbolType, visibility: Visibility, content: Option<VarContent<'b>>) -> Self
    {
        Self{
            name,
            offset,
            size,
            sindex,
            stype,
            visibility,
            content,
            addend,
            addt: 0
        }
    }
}
