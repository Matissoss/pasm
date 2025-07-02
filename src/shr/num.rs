// pasm - src/shr/num.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{error::RError as Error, size::Size};
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
    let sign = str.starts_with("-");
    let plus_sign = str.starts_with("+");
    let sign_str = if sign {
        "-"
    } else if plus_sign {
        "+"
    } else {
        ""
    };
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
    } else if let Ok(u32) = u32::from_str(str) {
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

fn num_from_bin(v: &[char], sign: bool) -> Result<Number, Error> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &'_' {
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
fn u8_from_bin(c: char) -> Option<u8> {
    match c {
        '0' => Some(0),
        '1' => Some(1),
        _ => None,
    }
}
fn num_from_hex(v: &[char], sign: bool) -> Result<Number, Error> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &'_' {
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
fn num_from_oct(v: &[char], sign: bool) -> Result<Number, Error> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &'_' {
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
fn u8_from_oct(c: char) -> Option<u8> {
    match c {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' => Some(c as u8 - b'0'),
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
