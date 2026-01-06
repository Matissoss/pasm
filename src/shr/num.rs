// pasm - src/shr/num.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::size::Size;
use std::str::FromStr;

#[derive(Clone, Copy)]
pub union Number {
    _u64: u64,
    _u32: u32,
    _u16: u16,
    _u8: u8,
    _i64: i64,
    _i32: i32,
    _i16: i16,
    _i8: i8,
    _f32: f32,
    _f64: f64,
    _usize: usize,
}

impl PartialEq for Number {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe { self._u64 == rhs._u64 }
    }
}
impl std::fmt::Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", unsafe { self._u64 })
    }
}
impl FromStr for Number {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(n) = num_from_str(s) {
            Ok(n)
        } else {
            Err(())
        }
    }
}
impl Number {
    pub const fn new(content: u64) -> Self {
        Self { _u64: content }
    }
    pub fn get_as_usize(&self) -> usize {
        self.get_raw() as usize
    }
    pub const fn get_as_u64(&self) -> u64 {
        self.get_raw()
    }
    pub fn get_as_u32(&self) -> u32 {
        self.get_raw() as u32
    }
    pub fn get_as_i32(&self) -> i32 {
        self.get_raw() as i32
    }
    pub fn unsigned_size(&self) -> Size {
        let content = unsafe { self._u64 };
        if content & 0xFF == content {
            Size::Byte
        } else if content & 0xFFFF == content {
            Size::Word
        } else if content & 0xFFFF_FFFF == content {
            Size::Dword
        } else {
            Size::Qword
        }
    }
    pub fn signed_size(&self) -> Size {
        match self.get_real_size() {
            1 => Size::Byte,
            2 => Size::Word,
            4 => Size::Dword,
            8 => Size::Qword,
            _ => Size::Unknown,
        }
    }
    pub fn get_real_size(&self) -> usize {
        let sz: u8 = self.unsigned_size().into();
        sz as usize
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
    pub const fn get_raw(&self) -> u64 {
        unsafe { self._u64 }
    }
    pub const fn is_signed(&self) -> bool {
        unsafe {
            self._i64.is_negative()
                || self._i32.is_negative()
                || self._i16.is_negative()
                || self._i8.is_negative()
        }
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

fn num_from_str(str: &str) -> Option<Number> {
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
    if str_chars.first() == Some(&b'0') {
        return match str_chars.get(1) {
            Some(&b'x') => num_from_hex(&str_chars[2..], sign),
            Some(&b'o') => num_from_oct(&str_chars[2..], sign),
            Some(&b'b') => num_from_bin(&str_chars[2..], sign),
            None => Some(Number::uint64(0)),
            _ => None,
        };
    } else if is_num(str) {
        if let Ok(u64) = u64::from_str(str) {
            return Some(Number::uint64(u64));
        } else if let Ok(i64) = i64::from_str(str) {
            return Some(Number::int64(i64));
        } else if let Ok(f64) = f64::from_str(str) {
            return Some(Number::double(f64));
        }
    }
    if str.starts_with("'") {
        let chr = str_chars.get(1);
        chr.map(|c| Number::uint64(*c as u64))
    } else {
        None
    }
}

#[inline(always)]
fn is_num(str: &str) -> bool {
    let mut minus = false;
    let mut dot = false;
    for b in str.as_bytes() {
        match *b {
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => {}
            b'-' => {
                if minus {
                    return false;
                } else {
                    minus = true;
                }
            }
            b'.' => {
                if dot {
                    return false;
                } else {
                    dot = true
                }
            }
            _ => {
                return false;
            }
        }
    }
    true
}

fn num(val: u64, sign: bool) -> Option<Number> {
    if sign {
        // we extract number without sign
        let body = val & 0x7FFF_FFFF_FFFF_FFFF;
        // if sign is set
        if body != val {
            None
        } else {
            // sign extended u64
            Some(Number::int64(-(val as i64)))
        }
    } else {
        Some(Number::uint64(val))
    }
}

fn num_from_bin(v: &[u8], sign: bool) -> Option<Number> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &b'_' {
            continue;
        } else if let Some(u) = u8_from_bin(*c) {
            n += u as u64 * (1 << idx);
        } else {
            return None;
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
fn num_from_hex(v: &[u8], sign: bool) -> Option<Number> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &b'_' {
            continue;
        } else if let Some(u) = u8_from_hex(*c) {
            n += u as u64 * (16u64.pow(idx));
        } else {
            return None;
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
fn num_from_oct(v: &[u8], sign: bool) -> Option<Number> {
    let mut n: u64 = 0;
    let mut idx = 0;
    for c in v.iter().rev() {
        if c == &b'_' {
            continue;
        } else if let Some(u) = u8_from_oct(*c) {
            n += u as u64 * (8u64.pow(idx));
        } else {
            return None;
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
        unsafe { self._u64 }.to_string()
    }
}

#[cfg(test)]
mod tests_1 {
    use super::*;
    #[test]
    fn number_t_n() {
        let str = "1.050"; // should parse into float
        assert_eq!(Number::from_str(str), Some(Number::float(1.050)));
        let str = "0b100101";
        assert_eq!(Number::from_str(str), Some(Number::uint64(0b100101)));
        let str = "-0b101";
        assert_eq!(Number::from_str(str), Some(Number::int64(-0b101)));
        let str = "0x0FF";
        assert_eq!(Number::from_str(str), Some(Number::uint64(0xFF)));
        let str = "-0x0FF";
        assert_eq!(Number::from_str(str), Some(Number::int64(-0xFF)));
    }
}
