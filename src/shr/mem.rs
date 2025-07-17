// pasm - src/shr/mem.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use crate::conf::{CLOSURE_END, CLOSURE_START};

use crate::shr::{
    booltable::BoolTable8,
    error::Error,
    num::Number,
    reg::{Purpose as RPurpose, Register},
    size::Size,
    smallvec::SmallVec,
};

use std::str::FromStr;

pub const RIP_ADDRESSING: u8 = 0x0;
pub const OBY_OFFSET: u8 = 0x1;
pub const IS_BCST: u8 = 0x2;
pub const HAS_BASE: u8 = 0x3;
pub const HAS_INDEX: u8 = 0x4;

// is index a vector register
pub const IS_VSIB_IDX: u8 = 0x5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Mem {
    // layout:
    // - 1st-4th: offset
    // - 5th: regs:
    //   XXXX_YYYY:
    //      XXXX: value of base register
    //      YYYY: value of index register
    // - 6th: metadata_1:
    //   XXXX_YYYA,
    //      XXXX - size (byte, word, dword, qword, xword, yword, zword, any)
    //      YYY - scale: (1, 2, 4, 8)
    //      A - sentinel for offset
    // - 7th: metadata_2:
    //      BBBF_FFCD
    //      BBB - size of used registers (byte, word, dword or qword registers)
    //          if IS_VSIB_IDX flag is set, then:
    //              0b001 - xword
    //              0b010 - yword
    //              0b011 - zword
    //          assert that address size is 64-bit for base
    //      FFF - segment:
    //          0b000 - no segment
    //          0b001 - CS
    //          0b010 - SS
    //          0b011 - DS
    //          0b100 - ES
    //          0b101 - FS
    //          0b110 - GS
    //          0b111 - reserved
    //      C - base ext_bits[0]
    //      D - index ext_bits[0]
    //  - 8th: flags
    offset: i32,
    regs: u8,
    metadata_1: u8,
    metadata_2: u8,
    flags: BoolTable8,
}

impl Mem {
    fn blank() -> Self {
        Self {
            offset: 0,
            regs: 0,
            metadata_1: 0,
            metadata_2: 0,
            flags: BoolTable8::new(),
        }
    }
    pub fn new(str: &str, sz: Size) -> Result<Self, Error> {
        match mem_par(mem_tok(str)) {
            Ok(mut o) => {
                mem_chk(&mut o);
                o.set_size(sz);
                Ok(o)
            }
            Err(e) => Err(e),
        }
    }
    // type
    pub fn is_bcst(&self) -> bool {
        self.get_flag(IS_BCST).unwrap_or(false)
    }
    pub fn is_riprel(&self) -> bool {
        self.get_flag(RIP_ADDRESSING).unwrap_or(false)
    }
    pub fn is_vsib(&self) -> bool {
        self.get_flag(IS_VSIB_IDX).unwrap_or(false)
    }
    pub fn is_sib(&self) -> bool {
        self.index().is_some() && self.base().is_some() && !self.is_riprel()
    }
    pub const fn get_segment(&self) -> Option<Register> {
        match (self.metadata_2 & 0b0001_1100) >> 2 {
            0b000 => None,
            0b001 => Some(Register::CS),
            0b010 => Some(Register::SS),
            0b011 => Some(Register::DS),
            0b100 => Some(Register::ES),
            0b101 => Some(Register::FS),
            0b110 => Some(Register::GS),
            _ => None,
        }
    }

    pub const fn set_bcst(&mut self, b: bool) {
        self.flags.set(IS_BCST, b);
    }

    pub fn set_segment(&mut self, rg: Register) -> bool {
        if rg.purpose() == RPurpose::Sgmnt {
            self.metadata_2 &= 0b1110_0011;
            self.metadata_2 |= (match rg {
                Register::CS => 0b001,
                Register::SS => 0b010,
                Register::DS => 0b011,
                Register::ES => 0b100,
                Register::FS => 0b101,
                Register::GS => 0b110,
                _ => 0b000,
            }) << 2;
            true
        } else {
            false
        }
    }

    // returns size of base or index
    pub fn addrsize(&self) -> Size {
        Size::de((self.metadata_2 & 0b1110_0000) >> 5)
    }
    pub fn needs_rex(&self) -> (bool, bool) {
        (self.base_rex(), self.index_rex())
    }
    // returns size of memory address
    pub fn size(&self) -> Size {
        Size::de((self.metadata_1 & 0b1111_0000) >> 4)
    }
    pub fn base_evex(&self) -> bool {
        self.metadata_2 & 0b0000_0010 == 0b0000_0010
    }
    pub fn index_evex(&self) -> bool {
        self.metadata_2 & 0b0000_0001 == 0b0000_0001
    }
    pub fn base_rex(&self) -> bool {
        self.regs & 0b1000_0000 == 0b1000_0000
    }
    pub fn index_rex(&self) -> bool {
        self.regs & 0b0000_1000 == 0b0000_1000
    }
    pub fn base(&self) -> Option<Register> {
        if self.get_flag(RIP_ADDRESSING).unwrap_or(false) {
            return Some(Register::RIP);
        }
        if self.flags.get(HAS_BASE).unwrap() {
            if self.flags.get(IS_VSIB_IDX).unwrap() {
                Some(Register::new(
                    RPurpose::General,
                    Size::Qword,
                    [self.base_evex(), self.base_rex()],
                    (self.regs & 0b0111_0000) >> 4,
                ))
            } else {
                Some(Register::new(
                    RPurpose::General,
                    self.addrsize(),
                    [self.base_evex(), self.base_rex()],
                    (self.regs & 0b0111_0000) >> 4,
                ))
            }
        } else {
            None
        }
    }
    pub fn index(&self) -> Option<Register> {
        if self.get_flag(RIP_ADDRESSING).unwrap_or(false) {
            return None;
        }
        if self.flags.get(HAS_INDEX).unwrap() {
            if self.flags.get(IS_VSIB_IDX).unwrap() {
                let (asz, rpr) = match self.addrsize() {
                    Size::Byte => (Size::Xword, RPurpose::F128),
                    Size::Word => (Size::Yword, RPurpose::F256),
                    Size::Dword => (Size::Zword, RPurpose::F512),
                    _ => (Size::Unknown, RPurpose::F128),
                };
                Some(Register::new(
                    rpr,
                    asz,
                    [self.index_evex(), self.index_rex()],
                    self.regs & 0b0000_0111,
                ))
            } else {
                Some(Register::new(
                    RPurpose::General,
                    self.addrsize(),
                    [self.index_evex(), self.index_rex()],
                    self.regs & 0b0000_0111,
                ))
            }
        } else {
            None
        }
    }
    pub fn offset_x86(&self) -> Option<([u8; 4], usize)> {
        if let Some(off) = self.offset() {
            let size = if self.flags.get(OBY_OFFSET).unwrap_or(false) {
                1
            } else {
                4
            };
            let off_le = off.to_le_bytes();
            Some((off_le, size))
        } else {
            None
        }
    }
    pub fn offset(&self) -> Option<i32> {
        if self.metadata_1 & 0b0000_0001 == 0b0000_0001 {
            Some(self.offset)
        } else {
            None
        }
    }
    pub fn scale(&self) -> Size {
        Size::de((self.metadata_1 & 0b0000_1110) >> 1)
    }
    pub fn get_flag(&self, idx: u8) -> Option<bool> {
        self.flags.get(idx)
    }

    // setters
    pub fn set_addrsize(&mut self, addrsize: Size) {
        let sz = Size::se(&addrsize) & 0b111;
        // clear addr size
        let mask = !0b1110_0000;
        // set addr_size
        self.metadata_2 = (self.metadata_2 & mask) | sz << 5;
    }
    // false if it fails
    // fails, if addrsize != base.size()
    pub fn set_base(&mut self, base: Register) -> bool {
        let bb = base.to_byte();
        let [ex, rx] = base.get_ext_bits();

        // check
        if self.addrsize() != base.size() && !self.is_vsib() {
            return false;
        }
        // set guardian
        self.flags.set(HAS_BASE, true);

        // set base
        let mask = !0b1111_0000;
        self.regs = (self.regs & mask) | (rx as u8) << 7 | bb << 4;
        self.metadata_2 = (self.metadata_2 & !0b10) | (ex as u8) << 1;
        true
    }
    // false if it fails
    // fails, if addrsize != base.size()
    pub fn set_index(&mut self, index: Register) -> bool {
        let bb = index.to_byte();
        let [ex, rx] = index.get_ext_bits();

        // check
        if index.purpose().is_avx() {
            self.flags.set(IS_VSIB_IDX, true);
            self.set_addrsize(match index.size() {
                Size::Xword => Size::Byte,
                Size::Yword => Size::Word,
                Size::Zword => Size::Dword,
                _ => Size::Unknown,
            });
        }
        if self.addrsize() != index.size() && !self.is_vsib() {
            return false;
        }
        // set guardian
        self.flags.set(HAS_INDEX, true);

        // set index
        let mask = !0b0000_1111;
        self.regs = (self.regs & mask) | (rx as u8) << 3 | bb;
        self.metadata_2 = (self.metadata_2 & !0b01) | ex as u8;
        true
    }
    pub fn set_offset(&mut self, offset: i32) {
        self.metadata_1 = (self.metadata_1 & !0b0000_0001) | 0b0000_0001;
        if <i32 as TryInto<i8>>::try_into(offset).is_ok() {
            self.flags.set(OBY_OFFSET, true);
        }
        self.offset = offset;
    }
    pub fn clear_offset(&mut self) {
        self.metadata_1 &= !0b0000_0001;
    }
    pub fn set_size(&mut self, sz: Size) {
        let sz = Size::se(&sz) & 0b111;
        let mask = !0b1111_0000;
        self.metadata_1 = (self.metadata_1 & mask) | sz << 4
    }
    pub fn set_scale(&mut self, sz: Size) {
        let sz = Size::se(&sz) & 0b111;
        let mask = !0b0000_1110;
        self.metadata_1 = (self.metadata_1 & mask) | (sz << 1);
    }
    pub fn set_flag(&mut self, idx: u8) {
        self.flags.set(idx, true)
    }
    pub fn clear_flag(&mut self, idx: u8) {
        self.flags.set(idx, false)
    }
}

#[derive(PartialEq, Debug)]
enum Token {
    Register(Register),
    Number(i32),
    // +
    Add,
    // -
    Sub,
    // *
    Mul,
}

fn mem_chk(mem: &mut Mem) {
    let base = mem.base();
    let index = mem.index();
    let offset = mem.offset();

    if let (None, Some(_)) = (base, index) {
        match index.unwrap().size() {
            Size::Word => mem.set_base(Register::BP),
            Size::Dword => mem.set_base(Register::EBP),
            Size::Qword => mem.set_base(Register::RBP),
            _ => false,
        };
    }

    if let (None, None, Some(_)) = (base, index, offset) {
        mem.set_flag(RIP_ADDRESSING);
    }
}

fn mem_par(toks: SmallVec<Token, 8>) -> Result<Mem, Error> {
    let mut mem = Mem::blank();

    let mut unspec_reg: Option<Register> = None;
    let mut base: Option<Register> = None;
    let mut index: Option<Register> = None;
    let mut offset: Option<i32> = None;
    let mut scale: Option<Size> = None;

    // if number was prefixed with *
    let mut mul_modf = false;
    let mut num_ismin = false;
    for tok in toks.into_iter() {
        match tok {
            Token::Register(r) => {
                if unspec_reg.is_none() {
                    unspec_reg = Some(r);
                } else if base.is_none() {
                    base = Some(r);
                } else if index.is_none() {
                    index = Some(r);
                } else {
                    return Err(Error::new(
                        "memory declaration has more than 2 registers",
                        11,
                    ));
                }
            }
            Token::Mul => mul_modf = true,
            Token::Add | Token::Sub => {
                if unspec_reg.is_some() {
                    base = unspec_reg;
                    unspec_reg = None;
                }
                if tok == Token::Sub {
                    num_ismin = true;
                }
            }
            Token::Number(n) => {
                if mul_modf {
                    if unspec_reg.is_some() {
                        index = unspec_reg;
                        unspec_reg = None;
                    }
                    if let Ok(sz) = Size::try_from(n as u16) {
                        match sz {
                            Size::Byte | Size::Word | Size::Dword | Size::Qword => {}
                            _ => {
                                return Err(Error::new(
                                    "you tried to use scale that is larger than 8 in memory",
                                    11,
                                ))
                            }
                        }
                        scale = Some(sz);
                    } else {
                        return Err(Error::new(
                            "you tried to provide scale, but it was not 1, 2, 4 or 8",
                            11,
                        ));
                    }
                    mul_modf = false;
                } else {
                    let n = if num_ismin { -n } else { n };
                    offset = Some(n);
                }
            }
        }
    }
    if let (Some(base), Some(index)) = (base, index) {
        let ipr = index.purpose();
        if base.size() != index.size() && !ipr.is_avx() {
            return Err(Error::new(
                "base and index registers in SIB memory declaration have different sizes",
                11,
            ));
        } else {
            mem.set_addrsize(base.size());
            let _ = mem.set_base(base);
            let _ = mem.set_index(index);
        }
    } else if let Some(index) = index {
        mem.set_addrsize(index.size());
        base = match index.size() {
            Size::Qword => Some(Register::RBP),
            Size::Dword => Some(Register::EBP),
            Size::Word => Some(Register::BP),
            _ => {
                return Err(Error::new(
                    "index register has size that is not 16, 32 or 64 bits",
                    11,
                ));
            }
        };
        let _ = mem.set_base(base.unwrap());
        let _ = mem.set_index(index);
    } else if let Some(base) = base {
        mem.set_addrsize(base.size());
        let _ = mem.set_base(base);
    }

    if let (Some(r), None) = (unspec_reg, base) {
        mem.set_addrsize(r.size());
        unspec_reg = None;
        mem.set_base(r);
    }
    if let (Some(r), None) = (unspec_reg, index) {
        mem.set_addrsize(r.size());
        mem.set_index(r);
    }

    if let Some(scale) = scale {
        mem.set_scale(scale);
    }
    if let Some(offset) = offset {
        mem.set_offset(offset);
    }

    Ok(mem)
}

const MS: u8 = CLOSURE_START as u8;
const ME: u8 = CLOSURE_END as u8;
fn mem_tok(str: &str) -> SmallVec<Token, 8> {
    let mut tokens = SmallVec::new();
    let bytes: &[u8] = str.as_bytes();
    let mut sstart = 0;
    let mut send = 0;
    for b in bytes {
        match *b {
            b'*' => {
                let b = &bytes[sstart..send];
                send += 1;
                sstart = send;
                if let Some(tok) = mem_tok_from_buf(b) {
                    tokens.push(tok);
                }
                tokens.push(Token::Mul);
            }
            b'-' => {
                let b = &bytes[sstart..send];
                send += 1;
                sstart = send;
                if let Some(tok) = mem_tok_from_buf(b) {
                    tokens.push(tok);
                }
                tokens.push(Token::Sub);
            }
            b'+' => {
                let b = &bytes[sstart..send];
                send += 1;
                sstart = send;
                if let Some(tok) = mem_tok_from_buf(b) {
                    tokens.push(tok);
                }
                tokens.push(Token::Add);
            }
            MS | ME | b' ' | b'\t' => {
                let b = &bytes[sstart..send];
                send += 1;
                sstart = send;
                if let Some(tok) = mem_tok_from_buf(b) {
                    tokens.push(tok);
                }
            }
            _ => send += 1,
        }
    }
    if sstart != send {
        if let Some(tok) = mem_tok_from_buf(&bytes[sstart..send]) {
            tokens.push(tok);
        }
    }

    tokens
}

fn mem_tok_from_buf(buf: &[u8]) -> Option<Token> {
    if buf.is_empty() {
        return None;
    }
    let utf8_buf = unsafe { std::str::from_utf8_unchecked(buf) };
    if let Ok(reg) = Register::from_str(utf8_buf) {
        Some(Token::Register(reg))
    } else {
        Number::from_str(utf8_buf).map(|n| Token::Number(n.get_as_i32()))
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Mem {
    fn to_string(&self) -> String {
        let mut str = String::new();
        str.push(CLOSURE_START);
        if let Some(reg) = self.base() {
            str.push_str(&reg.to_string());
            if self.index().is_some() {
                str.push_str(" + ");
            }
        }
        if let Some(reg) = self.index() {
            str.push_str(&reg.to_string());
            str.push_str(" * ");
            str.push_str(&(<Size as Into<u8>>::into(self.scale()).to_string()));
        }
        if let Some(offset) = self.offset() {
            let is_neg = offset.is_negative();
            let to_add = if is_neg { " - " } else { " + " };
            str.push_str(to_add);
            if is_neg {
                str.push_str(&offset.to_string()[1..]);
            } else {
                str.push_str(&offset.to_string());
            }
        }
        str.push(CLOSURE_END);
        str
    }
}

#[cfg(test)]
mod new_test {
    use super::*;
    #[test]
    fn mem_api_check() {
        assert!(size_of::<Mem>() == 8);
        let mut mem = Mem::blank();
        mem.set_addrsize(Size::Qword);
        assert_eq!(mem.addrsize(), Size::Qword);
        mem.set_addrsize(Size::Dword);
        assert_eq!(mem.addrsize(), Size::Dword);
        mem.set_offset(0x01);
        assert_eq!(mem.offset(), Some(0x01));
        mem.clear_offset();
        assert_eq!(mem.offset(), None);
        let _ = mem.set_base(Register::EAX);
        assert_eq!(mem.base(), Some(Register::EAX));
        let _ = mem.set_index(Register::EAX);
        assert_eq!(mem.index(), Some(Register::EAX));
        assert_eq!(mem.base(), Some(Register::EAX));
        mem.set_scale(Size::Byte);
        assert_eq!(mem.scale(), Size::Byte);
        let mut mem = Mem::blank();
        mem.set_addrsize(Size::Qword);
        mem.set_index(Register::RCX);
        assert_eq!(mem.index(), Some(Register::RCX));
    }
    #[test]
    fn mem_tok_t() {
        let str = "rax";
        assert_eq!(
            mem_tok(str).into_vec(),
            vec![Token::Register(Register::RAX)]
        );
        let str = "rax + rcx";
        assert_eq!(
            mem_tok(str).into_vec(),
            vec![
                Token::Register(Register::RAX),
                Token::Add,
                Token::Register(Register::RCX)
            ]
        );
    }
    #[test]
    fn mem_par_check() {
        let str = "rax";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.index(), None);
        assert!(!mem.is_sib());
        assert_eq!(mem.base(), Some(Register::RAX));
        let str = "rax + rcx";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RAX));
        assert_eq!(mem.index(), Some(Register::RCX));
        assert_eq!(mem.scale(), Size::Byte);
        let str = "rax + 20";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RAX));
        assert_eq!(mem.offset(), Some(20));
        let str = "rax + rcx * 4 + 20";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RAX));
        assert_eq!(mem.index(), Some(Register::RCX));
        assert_eq!(mem.scale(), Size::Dword);
        assert_eq!(mem.offset(), Some(20));
        let str = "rcx*4";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RBP));
        assert_eq!(mem.index(), Some(Register::RCX));
        assert_eq!(mem.scale(), Size::Dword);
        let str = "rcx * 4 + 20";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RBP));
        assert_eq!(mem.index(), Some(Register::RCX));
        assert_eq!(mem.scale(), Size::Dword);
        assert_eq!(mem.offset(), Some(20));
        let str = "-0xFF";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RIP));
        assert_eq!(mem.offset(), Some(-0xFF));
        let mem = Mem::new("eax + ebx", Size::Qword).unwrap();
        assert_eq!(mem.base(), Some(Register::EAX));
        assert_eq!(mem.index(), Some(Register::EBX));
        assert_eq!(mem.scale(), Size::Byte);
        let mem = Mem::new("rax + zmm23", Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RAX));
        assert_eq!(mem.index(), Some(Register::ZMM23));
    }
}
