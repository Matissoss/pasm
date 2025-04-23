// rasmx86_64 - var.rs
// -------------------
// made by matissoss
// licensed under MPL

use crate::shr::{
    num::Number,
    symbol::Visibility
};

#[derive(Debug, Clone, PartialEq)]
pub enum VType{
    Readonly, // .rodata
    Const   , // .data
    Uninit  , // .bss
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable{
    pub name: String,
    pub vtype: VType,
    pub size: u32,
    pub content: VarContent,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarContent{
    Number(Number),
    String(Vec<u8>),
    Uninit,
}

impl Variable{
    pub fn new(name: String, vtype: VType, size: u32, content: VarContent, visibility: Visibility) -> Self{
        Self{name,vtype,size,content,visibility}
    }
    pub fn bytes(&self) -> Vec<u8>{
        return self.content.bytes();
    }
}

impl VarContent{
    pub fn bytes(&self) -> Vec<u8>{
        match self{
            Self::Number(n) => n.split_into_bytes(),
            Self::String(s) => {
                let mut tmp_buf = Vec::new();
                for b in s.iter().rev(){
                    tmp_buf.push(b.to_le());
                }
                tmp_buf
            },
            Self::Uninit    => Vec::new(),
        }
    }
}
