// rasmx86_64 - src/shr/symbol.rs
// ------------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    num::Number,
    error::RASMError,
    reloc::RelType,
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
pub struct Symbol<'a> {
    pub name: &'a String,
    pub offset: u32,
    pub size  : u32,
    pub sindex: u16,
    pub visibility: Visibility,
    pub stype: SymbolType,
    // TODO: add flags and stuff
    // [...]
}

#[derive(PartialEq, Clone, Debug)]
pub struct SymbolRef {
    pub symbol: String,
    pub addend: i32,
    pub reltype: RelType,
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
        if str == "abs" {
            Token::RelType(RelType::ABS32)
        } else if str == "rel" {
            Token::RelType(RelType::REL32)
        } else {
            Token::String(str)
        }
    }
}

impl SymbolRef {
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
                b':' => {
                    if !tmp_buf.is_empty() {
                        let string = String::from_utf8(tmp_buf).unwrap_or_default();
                        match new_tok(string) {
                            Token::Number(n) => addend = n.get_as_i32(),
                            Token::String(s) => name = s,
                            Token::RelType(r)=> rtype = r,
                        }
                        tmp_buf = Vec::new();
                    }
                    if *b == b'+' || *b == b'-' {
                        tmp_buf.push(*b);
                    }
                    continue
                }
                b' '|b'\t' => continue,
                _ => tmp_buf.push(*b),
            }
        }
        if !tmp_buf.is_empty() {
            let string = String::from_utf8(tmp_buf).unwrap_or_default();
            match new_tok(string) {
                Token::Number(n) => addend = n.get_as_i32(),
                Token::String(s) => name = s,
                Token::RelType(r)=> rtype = r,
            }
        }

        if name.is_empty() {
            return Err(Error::msg("Tried to use reference a symbol, but you didn't provided name"));
        }


        Ok(Self {
            symbol: name,
            reltype: rtype,
            addend
        })
    }
}

impl ToString for SymbolRef {
    fn to_string(&self) -> String {
        let mut string = self.symbol.clone();
        string.push(':');
        string.push_str(if self.reltype == RelType::REL32 {"rel"} else {"abs"});
        if self.addend != 0 {
            string.push(':');
            string.push_str(&self.addend.to_string());
        }
        string
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rel_fromstr(){
        let str0 = "symbol:abs:+10";
        let str1 = "symbol:-10";
        let str2 = "symbol:abs";
        let str3 = "symbol";
        assert_eq!(SymbolRef::try_new(str0), Ok(SymbolRef {
            symbol: String::from("symbol"),
            reltype: RelType::ABS32,
            addend: 10
        }));
        assert_eq!(SymbolRef::try_new(str1), Ok(SymbolRef {
            symbol: String::from("symbol"),
            addend: -10,
            reltype: RelType::REL32
        }));
        assert_eq!(SymbolRef::try_new(str2), Ok(SymbolRef {
            symbol: String::from("symbol"),
            reltype: RelType::ABS32,
            addend: 0
        }));
        assert_eq!(SymbolRef::try_new(str3), Ok(SymbolRef {
            symbol: String::from("symbol"),
            addend: 0,
            reltype: RelType::REL32
        }));
    }
}
