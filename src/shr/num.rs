// rasmx86_64 - src/shr/num.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    atype::{AType, ToAType},
    error::RASMError,
    size::Size,
};
use std::str::FromStr;

impl ToAType for Number {
    fn atype(&self) -> AType {
        AType::Immediate(self.size())
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum NType {
    Float,
    Double,
    Unsigned,
    Signed,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Number {
    ntype: NType,
    content: u64,
}

use NType::*;
impl Number {
    pub fn new(ntype: NType, content: u64) -> Self {
        Self { ntype, content }
    }
    pub fn get_as_u64(&self) -> u64 {
        self.get_raw()
    }
    pub fn get_as_u32(&self) -> u32 {
        self.get_raw() as u32
    }
    pub fn get_as_i32(&self) -> i32 {
        self.get_raw() as i32
    }
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: &str) -> Result<Self, RASMError> {
        num_from_str(str)
    }
    pub fn size(&self) -> Size {
        match self.get_real_size() {
            1 => Size::Byte,
            2 => Size::Word,
            4 => Size::Dword,
            8 => Size::Qword,
            _ => Size::Unknown,
        }
    }
    pub fn get_real_size(&self) -> usize {
        if self.is_unsigned() {
            if u8::try_from(self.content).is_ok() {
                size_of::<u8>()
            } else if u16::try_from(self.content).is_ok() {
                size_of::<u16>()
            } else if u32::try_from(self.content).is_ok() {
                size_of::<u32>()
            } else {
                size_of::<u64>()
            }
        } else if self.is_signed() {
            let body = self.content;
            // try to guess where to set sign
            if body & 0xFF == body && body & 0x80 != 0x80 {
                size_of::<i8>()
            } else if body & 0xFFFF == body && body & 0x8000 != 0x8000 {
                size_of::<i16>()
            } else if body & 0xFFFF_FFFF == body && body & 0x8000_0000 == 0x8000_0000 {
                size_of::<i32>()
            } else {
                size_of::<i64>()
            }
        } else {
            if self.is_float() {
                size_of::<f32>()
            } else {
                size_of::<f64>()
            }
        }
    }
    pub fn split_into_bytes(self) -> Vec<u8> {
        self.get_raw_le()[..self.get_real_size()].to_vec()
    }
    pub fn get_raw_le(&self) -> [u8; 8] {
        self.get_raw().to_le_bytes()
    }
    pub fn get_raw_be(&self) -> [u8; 8] {
        self.get_raw().to_be_bytes()
    }
    pub fn get_raw(&self) -> u64 {
        if self.is_signed() {
            let body = self.content;
            // guess where to set sign
            if body & 0xFF == body && body & 0x80 != 0x80 {
                body | 0x80
            } else if body & 0xFFFF == body && body & 0x8000 != 0x8000 {
                body | 0x8000
            } else if body & 0xFFFF_FFFF == body && body & 0x8000_0000 == 0x8000_0000 {
                body | 0x8000_0000
            } else {
                body | 0x8000_0000_0000_0000
            }
        } else {
            self.content
        }
    }
    pub fn is_unsigned(&self) -> bool {
        self.ntype() == Unsigned
    }
    pub fn is_signed(&self) -> bool {
        self.ntype() == Signed
    }
    pub fn is_double(&self) -> bool {
        self.ntype() == Double
    }
    pub fn is_float(&self) -> bool {
        self.ntype() == Float
    }
    pub fn ntype(&self) -> NType {
        self.ntype
    }
    pub fn float(f: f32) -> Self {
        Self::new(Float, f as u64)
    }
    pub fn double(f: f64) -> Self {
        Self::new(Double, f as u64)
    }
    pub fn uint64(u: u64) -> Self {
        Self::new(Unsigned, u)
    }
    pub fn int64(i: i64) -> Self {
        Self::new(Signed, i as u64)
    }
}

fn num_from_str(str: &str) -> Result<Number, RASMError> {
    let sign = str.starts_with("-");
    let plus_sign = str.starts_with("+");
    let sign_str = if sign { "-" } else if plus_sign { "+" } else { "" };
    let hex_init = "0x";
    let bin_init = "0b";
    let oct_init = "0o";
    let ef_start = 2 + sign as usize;
    let str_chars = str.chars().collect::<Vec<char>>();
    if str.starts_with(&format!("{sign_str}{hex_init}")) {
        num_from_hex(&str_chars[ef_start..], sign)
    } else if str.starts_with(&format!("{sign_str}{bin_init}")) {
        num_from_bin(&str_chars[ef_start..], sign)
    } else if str.starts_with(&format!("{sign_str}{oct_init}")) {
        num_from_oct(&str_chars[ef_start..], sign)
    } else {
        if let Ok(u32) = u32::from_str(str) {
            Ok(Number::uint64(u32 as u64))
        } else if let Ok(u64) = u64::from_str(str) {
            Ok(Number::uint64(u64))
        } else if let Ok(i32) = i32::from_str(str) {
            Ok(Number::int64(i32 as i64))
        } else if let Ok(i64) = i64::from_str(str) {
            Ok(Number::int64(i64))
        } else if let Ok(float) = f32::from_str(str) {
            Ok(Number::float(float))
        } else if let Ok(double) = f64::from_str(str) {
            Ok(Number::double(double))
        } else {
            if str.starts_with("'") {
                let chr = str_chars.get(1);
                match chr {
                    Some('\\') => {
                        if let Some(c) = str_chars.get(2) {
                            if let Some(e) = escape_char(*c) {
                                Ok(Number::uint64(e as u64))
                            } else if c == &'\'' {
                                Ok(Number::uint64('\\' as u64))
                            } else {
                                Err(RASMError::no_tip(
                                    None,
                                    Some(format!("Invalid escape character: {c}")),
                                ))
                            }
                        } else {
                            Err(RASMError::no_tip(None, Some("Unclosed '' delimeter")))
                        }
                    }
                    Some(c) => Ok(Number::uint64(*c as u64)),
                    None => Err(RASMError::no_tip(None, Some("Unclosed '' delimeter"))),
                }
            } else {
                Err(RASMError::no_tip(
                    None,
                    Some(format!("Couldn't parse \"{str}\" into number")),
                ))
            }
        }
    }
}

fn num(val: u64, sign: bool, ntype: NType) -> Result<Number, RASMError> {
    if ntype == NType::Float {
        return Ok(Number::float(val as f32));
    } else if ntype == NType::Double {
        return Ok(Number::double(val as f64));
    }
    if sign {
        // we extract number without sign
        let body = val & 0x7FFF_FFFF_FFFF_FFFF;
        // if sign is set
        if body != val {
            Err(RASMError::no_tip(
                None,
                Some(format!(
                    "Tried to use too large number to be signed (consider removing sign): {}",
                    body
                )),
            ))
        } else {
            // sign extended u64
            Ok(Number::int64(-(val as i64)))
        }
    } else {
        Ok(Number::uint64(val))
    }
}

fn num_from_bin(v: &[char], sign: bool) -> Result<Number, RASMError> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &'_' {
            continue;
        } else {
            if let Some(u) = u8_from_bin(*c) {
                n += u as u64 * (1 << idx);
            } else {
                return Err(RASMError::no_tip(
                    None,
                    Some("Invalid character was found inside binary number declaration!"),
                ));
            }
        }
        idx += 1;
    }
    num(n, sign, NType::Unsigned)
}
fn u8_from_bin(c: char) -> Option<u8> {
    match c {
        '0' => Some(0),
        '1' => Some(1),
        _ => None,
    }
}
fn num_from_hex(v: &[char], sign: bool) -> Result<Number, RASMError> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &'_' {
            continue;
        } else {
            if let Some(u) = u8_from_hex(*c) {
                n += u as u64 * (16u64.pow(idx));
            } else {
                return Err(RASMError::no_tip(
                    None,
                    Some("Invalid character was found inside hex number declaration!"),
                ));
            }
        }
        idx += 1;
    }
    num(n, sign, NType::Unsigned)
}
fn u8_from_hex(c: char) -> Option<u8> {
    match c {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => Some(c as u8 - b'0'),
        'a' | 'A' => Some(10),
        'b' | 'B' => Some(11),
        'c' | 'C' => Some(12),
        'd' | 'D' => Some(13),
        'e' | 'E' => Some(14),
        'f' | 'F' => Some(15),
        _ => None,
    }
}
fn num_from_oct(v: &[char], sign: bool) -> Result<Number, RASMError> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &'_' {
            continue;
        } else {
            if let Some(u) = u8_from_oct(*c) {
                n += u as u64 * (8u64.pow(idx));
            } else {
                return Err(RASMError::no_tip(
                    None,
                    Some("Invalid character was found inside octal number declaration!"),
                ));
            }
        }
        idx += 1;
    }
    num(n, sign, NType::Unsigned)
}
fn u8_from_oct(c: char) -> Option<u8> {
    match c {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' => Some(c as u8 - b'0'),
        _ => None,
    }
}

fn escape_char(c: char) -> Option<char> {
    match c {
        'n' => Some('\n'),
        't' => Some('\t'),
        '0' => Some('\0'),
        '\'' => Some('\''),
        '\\' => Some('\\'),
        'r' => Some('\r'),
        _ => None,
    }
}

impl ToString for Number {
    fn to_string(&self) -> String {
        if self.is_float() {
            (self.content as f32).to_string()
        } else if self.is_double() {
            (self.content as f64).to_string()
        } else if self.is_signed() {
            (self.content as i64).to_string()
        } else {
            self.content.to_string()
        }
    }
}

#[cfg(test)]
mod tests_1 {
    use super::*;
    #[test]
    fn number_t_n() {
        let str = "1.050"; // should parse into float
        assert_eq!(Number::from_str(str), Ok(Number::float(1.050)));
        let str = "0b100101";
        assert_eq!(Number::from_str(str), Ok(Number::uint64(0b100101)));
        let str = "-0b101";
        assert_eq!(Number::from_str(str), Ok(Number::int64(-0b101)));
        let str = "0x0FF";
        assert_eq!(Number::from_str(str), Ok(Number::uint64(0xFF)));
        let str = "-0x0FF";
        assert_eq!(Number::from_str(str), Ok(Number::int64(-0xFF)));
    }
}
