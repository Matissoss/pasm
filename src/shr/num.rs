// pasm - src/shr/num.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{error::Error, size::Size};
use std::str::FromStr;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Number {
    content: u64,
}

impl Number {
    pub const fn new(content: u64) -> Self {
        Self { content }
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
    pub fn from_str(str: &str) -> Result<Self, Error> {
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
        if u8::try_from(self.content).is_ok() {
            size_of::<u8>()
        } else if u16::try_from(self.content).is_ok() {
            size_of::<u16>()
        } else if u32::try_from(self.content).is_ok() {
            size_of::<u32>()
        } else {
            size_of::<u64>()
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
    pub fn is_signed(&self) -> bool {
        self.content & (1 << 63) == 1 << 63
    }
    pub const fn float(f: f32) -> Self {
        Self::new(f as u64)
    }
    pub const fn double(f: f64) -> Self {
        Self::new(f as u64)
    }
    pub const fn uint64(u: u64) -> Self {
        Self::new(u)
    }
    pub const fn int64(i: i64) -> Self {
        Self::new(i as u64)
    }
}

fn num_from_str(str: &str) -> Result<Number, Error> {
    let sab = str.as_bytes();
    let sign = sab.first() == Some(&b'-');
    let str_chars: &[u8] = {
        let mut start = 0;
        for b in sab {
            if *b == b'-' || *b == b'+' {
                start += 1;
            } else {
                break;
            }
        }
        &sab[start..]
    };
    if str_chars.starts_with(b"0x") {
        return num_from_hex(&str_chars[2..], sign);
    }
    if str_chars.starts_with(b"0b") {
        return num_from_bin(&str_chars[2..], sign);
    }
    if str_chars.starts_with(b"0o") {
        num_from_oct(&str_chars[2..], sign)
    } else if let Ok(u64) = u64::from_str(str) {
        Ok(Number::uint64(u64))
    } else if let Ok(i64) = i64::from_str(str) {
        Ok(Number::int64(i64))
    } else if let Ok(double) = f64::from_str(str) {
        Ok(Number::double(double))
    } else if str.starts_with("'") {
        let chr = str_chars.get(1);
        match chr {
            Some(c) => Ok(Number::uint64(*c as u64)),
            None => Err(Error::new(
                "found unclosed '' delimeter inside character declaration",
                0,
            )),
        }
    } else {
        Err(Error::new(
            format!("you provided string \"{str}\", which could not be parsed into number"),
            105,
        ))
    }
}

fn num(val: u64, sign: bool) -> Result<Number, Error> {
    if sign {
        // we extract number without sign
        let body = val & 0x7FFF_FFFF_FFFF_FFFF;
        // if sign is set
        if body != val {
            Err(Error::new("you tried to use number which already occupies sign bit (consider removing sign and making it unsigned)", 1))
        } else {
            // sign extended u64
            Ok(Number::int64(-(val as i64)))
        }
    } else {
        Ok(Number::uint64(val))
    }
}

fn num_from_bin(v: &[u8], sign: bool) -> Result<Number, Error> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &b'_' {
            continue;
        } else if let Some(u) = u8_from_bin(*c) {
            n += u as u64 * (1 << idx);
        } else {
            return Err(Error::new(
                "you tried to make binary number, but you used character that isn't 0 or 1 or _",
                2,
            ));
        }
        idx += 1;
    }
    num(n, sign)
}
fn u8_from_bin(c: u8) -> Option<u8> {
    match c {
        b'0' => Some(0),
        b'1' => Some(1),
        _ => None,
    }
}
fn num_from_hex(v: &[u8], sign: bool) -> Result<Number, Error> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &b'_' {
            continue;
        } else if let Some(u) = u8_from_hex(*c) {
            n += u as u64 * (16u64.pow(idx));
        } else {
            return Err(Error::new(
                "you tried to make hex number, but you used character that isn't hexadecimal or _",
                2,
            ));
        }
        idx += 1;
    }
    num(n, sign)
}
fn u8_from_hex(c: u8) -> Option<u8> {
    match c {
        b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => Some(c - b'0'),
        b'a' | b'A' => Some(10),
        b'b' | b'B' => Some(11),
        b'c' | b'C' => Some(12),
        b'd' | b'D' => Some(13),
        b'e' | b'E' => Some(14),
        b'f' | b'F' => Some(15),
        _ => None,
    }
}
fn num_from_oct(v: &[u8], sign: bool) -> Result<Number, Error> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &b'_' {
            continue;
        } else if let Some(u) = u8_from_oct(*c) {
            n += u as u64 * (8u64.pow(idx));
        } else {
            return Err(Error::new("you tried to make octal number, but you used character that isn't octal (0 to 7) or _", 2));
        }
        idx += 1;
    }
    num(n, sign)
}
fn u8_from_oct(c: u8) -> Option<u8> {
    match c {
        b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' => Some(c - b'0'),
        _ => None,
    }
}

impl ToString for Number {
    fn to_string(&self) -> String {
        self.content.to_string()
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
