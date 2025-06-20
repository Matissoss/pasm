// rasmx86_64 - src/shr/symbol.rs
// ------------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::conf::PREFIX_VAL;
use crate::shr::{
    booltable::BoolTable8, error::RASMError, num::Number, reloc::RelType, size::Size,
};

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum Visibility {
    #[default]
    Local = 0,
    Global = 1,
}

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
    pub is_extern: bool,
    // TODO: add flags and stuff
    // [...]
}

impl Symbol {
    pub fn is_global(&self) -> bool {
        self.visibility == Visibility::Global
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

type Error = RASMError;

enum Token {
    String(String),
    RelType(RelType),
    Number(Number),
}

fn new_tok(str: String) -> Token {
    if let Ok(num) = Number::from_str(&str) {
        Token::Number(num)
    } else {
        match str.as_ref() {
            "abs" => Token::RelType(RelType::ABS32),
            "rel" | "rel32" => Token::RelType(RelType::REL32),
            "rel16" => Token::RelType(RelType::REL16),
            "rel8" => Token::RelType(RelType::REL8),
            _ => Token::String(str),
        }
    }
}

const PREFIX_VAL_U8: u8 = PREFIX_VAL as u8;
impl SymbolRef {
    pub fn deref(&mut self, bool: bool) {
        self.guardians.set(IS_DEREF, bool)
    }
    pub fn new(
        symb: String,
        addend: Option<i32>,
        is_deref: bool,
        sz: Option<Size>,
        reltype: Option<RelType>,
    ) -> Self {
        Self {
            symbol: symb.into(),
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
    pub fn addend(&self) -> Option<i32> {
        if self.guardians.get(ADED_GUARDIAN).unwrap() {
            Some(self.addend)
        } else {
            None
        }
    }
    // syntax:
    //
    // @(name:reltype:+/-addend)
    //
    pub fn try_new(str: &str) -> Result<Self, Error> {
        let bytes = str.as_bytes();
        let (mut name, mut rtype, mut addend) = (String::new(), RelType::REL32, 0);
        let mut tmp_buf = Vec::new();
        for b in bytes {
            match *b {
                PREFIX_VAL_U8 | b'(' | b')' => continue,
                b':' => {
                    if !tmp_buf.is_empty() {
                        let string = String::from_utf8(tmp_buf).unwrap_or_default();
                        match new_tok(string) {
                            Token::Number(n) => addend = n.get_as_i32(),
                            Token::String(s) => name = s,
                            Token::RelType(r) => rtype = r,
                        }
                        tmp_buf = Vec::new();
                    }
                    if *b == b'+' || *b == b'-' {
                        tmp_buf.push(*b);
                    }
                    continue;
                }
                b' ' | b'\t' => continue,
                _ => tmp_buf.push(*b),
            }
        }
        if !tmp_buf.is_empty() {
            let string = String::from_utf8(tmp_buf).unwrap_or_default();
            match new_tok(string) {
                Token::Number(n) => addend = n.get_as_i32(),
                Token::String(s) => name = s,
                Token::RelType(r) => rtype = r,
            }
        }

        if name.is_empty() {
            return Err(Error::msg(
                "Tried to use reference a symbol, but you didn't provided name",
            ));
        }

        Ok(Self::new(name, Some(addend), false, None, Some(rtype)))
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
