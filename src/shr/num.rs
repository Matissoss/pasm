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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Number {
    Float(f32),
    Double(f64),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Char(char),
}

impl FromStr for Number {
    type Err = RASMError;
    fn from_str(str: &str) -> Result<Self, <Self as FromStr>::Err> {
        let bytes = str.as_bytes();

        match bytes.len() {
            1 => {
                if (bytes[0] as char).is_numeric() {
                    Ok(Self::UInt8(bytes[0] - b'0'))
                } else {
                    Ok(Self::Char(bytes[0] as char))
                }
            }
            _ => {
                if str.starts_with("'") {
                    return match parse_char(str.as_bytes()) {
                        Ok(c) => Ok(Self::Char(c)),
                        Err(e) => Err(e),
                    };
                }
                if let Ok(n) = str.parse::<u64>() {
                    return Ok(Self::squeeze_u64(n));
                }
                if let Ok(n) = str.parse::<i64>() {
                    return Ok(Self::squeeze_i64(n));
                }

                if str.starts_with("0x") {
                    let mut number: u64 = 0;

                    for (index, i) in (2..bytes.len()).rev().enumerate() {
                        let nm = hexchar(bytes[i] as char) as u64;
                        if nm != 16 {
                            number += nm * (16u64.pow(index as u32))
                        } else {
                            return Err(RASMError::with_tip(
                                None,
                                Some(format!(
                                    "Invalid hexadecimal character was found inside number: {}",
                                    bytes[i] as char
                                )),
                                Some(format!(
                                    "Consider changing {} to either 0 to 9 and A to F",
                                    bytes[i] as char
                                )),
                            ));
                        }
                    }
                    return Ok(Self::squeeze_u64(number));
                }
                if str.starts_with("-0x") {
                    let mut number: i64 = 0;

                    for (index, i) in (3..bytes.len()).rev().enumerate() {
                        let nm = hexchar(bytes[i] as char) as i64;
                        if nm != 16 {
                            number += nm * (16i64.pow(index as u32))
                        } else {
                            return Err(RASMError::with_tip(
                                None,
                                Some(format!(
                                    "Invalid hexadecimal character was found inside number: {}",
                                    bytes[i] as char
                                )),
                                Some(format!(
                                    "Consider changing {} to either 0 to 9 and A to F",
                                    bytes[i] as char
                                )),
                            ));
                        }
                    }
                    return Ok(Self::squeeze_i64(-number));
                }
                if str.starts_with("0b") {
                    let mut number: u64 = 0;

                    for (index, i) in (2..bytes.len()).enumerate() {
                        let nm = (bytes[i] - b'0') as u64;
                        if nm < 2 {
                            number += nm * (1 << index)
                        } else {
                            return Err(RASMError::with_tip(
                                None,
                                Some(format!(
                                    "Invalid binary character was found inside number: {}",
                                    bytes[i] as char
                                )),
                                Some(format!(
                                    "Consider changing {} to either 1 or 0",
                                    bytes[i] as char
                                )),
                            ));
                        }
                    }
                    return Ok(Self::squeeze_u64(number));
                }
                if str.starts_with("-0b") {
                    let mut number: i64 = 0;

                    for (index, i) in (3..bytes.len()).enumerate() {
                        let nm = (bytes[i] - b'0') as i64;
                        if nm < 2 {
                            number += nm * (1 << index)
                        } else {
                            return Err(RASMError::with_tip(
                                None,
                                Some(format!(
                                    "Invalid binary character was found inside number: {}",
                                    bytes[i] as char
                                )),
                                Some(format!(
                                    "Consider changing {} to either 1 or 0",
                                    bytes[i] as char
                                )),
                            ));
                        }
                    }
                    return Ok(Self::squeeze_i64(-number));
                }

                if let Ok(db) = str.parse::<f64>() {
                    return Ok(Self::squeeze_f64(db));
                }

                Err(RASMError::no_tip(
                    None,
                    Some(format!("Couldn't parse string into number: {}", str)),
                ))
            }
        }
    }
}

const MAX_U8: u64 = u8::MAX as u64;
const MAX_U16: u64 = u16::MAX as u64;
const MAX_U32: u64 = u32::MAX as u64;

const MIN_I8: i64 = i8::MIN as i64;
const MIN_I16: i64 = i16::MIN as i64;
const MIN_I32: i64 = i32::MIN as i64;

const MAX_I8: i64 = i8::MAX as i64;
const MAX_I16: i64 = i16::MAX as i64;
const MAX_I32: i64 = i32::MAX as i64;

const MAX_F32: f64 = f32::MAX as f64;
const MIN_F32: f64 = f32::MIN as f64;

impl Number {
    pub fn squeeze_u64(numb: u64) -> Self {
        #[allow(overlapping_range_endpoints, clippy::match_overlapping_arm)]
        match numb {
            0..=MAX_U8 => Self::UInt8(numb as u8),
            MAX_U8..=MAX_U16 => Self::UInt16(numb as u16),
            MAX_U16..=MAX_U32 => Self::UInt32(numb as u32),
            _ => Self::UInt64(numb),
        }
    }
    pub fn squeeze_i64(numb: i64) -> Self {
        match numb {
            MIN_I8..=MAX_I8 => Self::Int8(numb as i8),
            MIN_I16..=MAX_I16 => Self::Int16(numb as i16),
            MIN_I32..=MAX_I32 => Self::Int32(numb as i32),
            _ => Self::Int64(numb),
        }
    }
    pub fn squeeze_f64(numb: f64) -> Self {
        match numb {
            MIN_F32..MAX_F32 => Self::Float(numb as f32),
            _ => Self::Double(numb),
        }
    }
    pub fn get_int(&self) -> Option<i64> {
        match self {
            Self::Int64(u) => Some(*u),
            Self::Int32(u) => Some(*u as i64),
            Self::Int16(u) => Some(*u as i64),
            Self::Int8(u) => Some(*u as i64),
            _ => None,
        }
    }
    pub fn get_uint(&self) -> Option<u64> {
        match self {
            Self::Char(u) => Some(*u as u64),
            Self::UInt64(u) => Some(*u),
            Self::UInt32(u) => Some(*u as u64),
            Self::UInt16(u) => Some(*u as u64),
            Self::UInt8(u) => Some(*u as u64),
            _ => None,
        }
    }
    pub fn size(&self) -> Size {
        match self {
            Self::Char(_) | Self::UInt8(_) | Self::Int8(_) => Size::Byte,
            Self::UInt16(_) | Self::Int16(_) => Size::Word,
            Self::Float(_) | Self::UInt32(_) | Self::Int32(_) => Size::Dword,
            Self::Double(_) | Self::UInt64(_) | Self::Int64(_) => Size::Qword,
        }
    }
    pub fn split_into_bytes(self) -> Vec<u8> {
        match self {
            Self::Int8(n) => n.to_le_bytes().to_vec(),
            Self::Int16(n) => n.to_le_bytes().to_vec(),
            Self::Int32(n) => n.to_le_bytes().to_vec(),
            Self::Int64(n) => n.to_le_bytes().to_vec(),
            Self::UInt64(n) => n.to_le_bytes().to_vec(),
            Self::UInt8(n) => n.to_le_bytes().to_vec(),
            Self::UInt16(n) => n.to_le_bytes().to_vec(),
            Self::UInt32(n) => n.to_le_bytes().to_vec(),
            Self::Char(c) => vec![(c as u8).to_le()],
            _ => vec![],
        }
    }
}

fn hexchar(c: char) -> u8 {
    match c {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => c as u8 - b'0',
        'a' | 'A' => 10,
        'b' | 'B' => 11,
        'c' | 'C' => 12,
        'd' | 'D' => 13,
        'e' | 'E' => 14,
        'f' | 'F' => 15,
        _ => 16,
    }
}

const ESCAPE: u8 = b'\\';
fn parse_char(txt: &[u8]) -> Result<char, RASMError> {
    if !(txt.starts_with(&[(b'\'')]) && txt.ends_with(&[(b'\'')])) {
        Err(RASMError::no_tip(
            None,
            Some(format!(
                "Invalidly formatted char: {}",
                String::from_utf8_lossy(txt)
            )),
        ))
    } else if txt.len() == 4 {
        if txt[1] == ESCAPE {
            match txt[2] as char {
                't' => Ok('\t'),
                'n' => Ok('\n'),
                'r' => Ok('\r'),
                '0' => Ok('\0'),
                '\\' => Ok('\\'),
                '\'' => Ok('\''),
                _ => Err(RASMError::no_tip(
                    None,
                    Some(format!("Invalid escape character \\{}", txt[2] as char)),
                )),
            }
        } else {
            Err(RASMError::no_tip(
                None,
                Some("Character declaration has more than 1 character inside closure `'`."),
            ))
        }
    } else {
        Ok(txt[1] as char)
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Number {
    fn to_string(&self) -> String {
        match self {
            Self::Float(f) => f.to_string(),
            Self::Double(d) => d.to_string(),
            Self::Int8(i) => i.to_string(),
            Self::Int16(i) => i.to_string(),
            Self::Int32(i) => i.to_string(),
            Self::Int64(i) => i.to_string(),
            Self::UInt8(i) => i.to_string(),
            Self::UInt16(i) => i.to_string(),
            Self::UInt32(i) => i.to_string(),
            Self::UInt64(i) => i.to_string(),
            Self::Char(c) => c.to_string(),
        }
    }
}

impl ToAType for Number {
    fn atype(&self) -> AType {
        AType::Immediate(self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn number_t() {
        let str = "1.050"; // should parse into float
        assert!(Number::from_str(str) == Ok(Number::Float(1.050)));
        let str = "1.05000000001";
        assert!(Number::from_str(str) == Ok(Number::Float(1.05000000001)));
        let str = "0b101";
        assert!(Number::from_str(str) == Ok(Number::UInt8(5)));
        let str = "-0b101";
        assert!(Number::from_str(str) == Ok(Number::Int8(-5)));
        let str = "0x0FF";
        assert!(Number::from_str(str) == Ok(Number::UInt8(255)));
        let str = "-0x0FF";
        assert!(Number::from_str(str) == Ok(Number::Int16(-255)));
    }
}
