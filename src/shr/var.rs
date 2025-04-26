// rasmx86_64 - src/shr/var.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;

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
pub struct Variable<'a>{
    pub name: Cow<'a, String>,
    pub vtype: VType,
    pub size: u32,
    pub content: VarContent<'a>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarContent<'a>{
    Number(Number),
    String(Cow<'a, Vec<u8>>),
    Uninit,
}

impl<'a> Variable<'a>{
    pub fn new(name: String, vtype: VType, size: u32, content: VarContent<'a>, visibility: Visibility) -> Self{
        Self{name:Cow::Owned(name),vtype,size,content,visibility}
    }
    pub fn bytes(&self) -> Vec<u8>{
        return self.content.bytes();
    }
}

impl<'a> VarContent<'a>{
    pub fn bytes(&self) -> Vec<u8>{
        match self{
            Self::Number(n) => n.split_into_bytes(),
            Self::String(s) => {
                s.to_vec()
                /*
                let mut tmp_buf = Vec::new();
                for b in s.iter().rev(){
                    tmp_buf.push(b.to_le());
                }
                tmp_buf
                    */
            },
            Self::Uninit    => Vec::new(),
        }
    }
}
