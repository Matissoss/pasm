// rasmx86_64 - src/shr/mem.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    atype::AType,
    booltable::BoolTable8,
    error::RASMError,
    num::Number,
    reg::{Purpose as RPurpose, Register},
    size::Size,
};

use std::str::FromStr;

pub const RIP_ADDRESSING: u8 = 0x0;
pub const OBY_OFFSET: u8 = 0x1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Mem {
    // layout:
    // - 1st-4th: offset
    // - 5th: regs:
    //   XX_YYY_ZZZ:
    //      XX: sentinels for base and index (first for base, second for index)
    //      YYY: value of base register
    //      ZZZ: value of index register
    // - 6th: metadata_1:
    //   XXXX_YYYA,
    //      XXXX - size (byte, word, dword, qword, xword, yword, zword, any)
    //      YYY - scale: (1, 2, 4, 8)
    //      A - sentinel for offset
    // - 7th: metadata_2:
    //      BBBF_FFCD
    //      BBB - size of used registers (byte, word, dword or qword registers)
    //      FFF - segment:
    //          0b000 - no segment
    //          0b001 - CS
    //          0b010 - SS
    //          0b011 - DS
    //          0b100 - ES
    //          0b101 - FS
    //          0b110 - GS
    //          0b111 - reserved
    //      C - base uses REX
    //      D - index uses REX
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
    pub fn new(str: &str, sz: Size) -> Result<Self, RASMError> {
        match mem_par_new(mem_tok_new(str)) {
            Ok(mut o) => {
                mem_chk(&mut o);
                o.set_size(sz);
                Ok(o)
            }
            Err(e) => Err(e),
        }
    }
    // type
    pub fn is_riprel(&self) -> bool {
        self.get_flag(RIP_ADDRESSING).unwrap_or(false)
    }
    pub fn is_sib(&self) -> bool {
        self.index().is_some()
            && self.scale().is_some()
            && self.base().is_some()
            && !self.is_riprel()
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

    // getters
    pub fn atype(&self) -> AType {
        let sz = self.size().unwrap_or(Size::Unknown);
        AType::Memory(sz)
    }

    // returns size of base or index
    pub fn addrsize(&self) -> Option<Size> {
        Self::compressed_size((self.metadata_2 & 0b1110_0000) >> 5)
    }
    pub fn needs_rex(&self) -> (bool, bool) {
        (self.base_rex(), self.index_rex())
    }
    // returns size of memory address
    pub fn size(&self) -> Option<Size> {
        Self::compressed_size((self.metadata_1 & 0b1111_0000) >> 4)
    }
    pub fn base_rex(&self) -> bool {
        self.metadata_2 & 0b0000_0010 == 0b0000_0010
    }
    pub fn index_rex(&self) -> bool {
        self.metadata_2 & 0b0000_0001 == 0b0000_0001
    }
    pub fn base(&self) -> Option<Register> {
        if self.get_flag(RIP_ADDRESSING).unwrap_or(false) {
            return Some(Register::RIP);
        }
        if self.regs & 0b1000_0000 == 0b1000_0000 {
            let val = (self.regs & 0b0011_1000) >> 3;
            Self::compressed_reg(
                self.addrsize().unwrap_or(Size::Unknown),
                self.base_rex(),
                val,
            )
        } else {
            None
        }
    }
    pub fn index(&self) -> Option<Register> {
        if self.get_flag(RIP_ADDRESSING).unwrap_or(false) {
            return None;
        }
        if self.regs & 0b0100_0000 == 0b0100_0000 {
            let val = self.regs & 0b0000_0111;
            Self::compressed_reg(
                self.addrsize().unwrap_or(Size::Unknown),
                self.index_rex(),
                val,
            )
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
    pub fn scale(&self) -> Option<Size> {
        Self::compressed_size((self.metadata_1 & 0b0000_1110) >> 1)
    }
    pub fn get_flag(&self, idx: u8) -> Option<bool> {
        self.flags.get(idx)
    }

    // setters
    pub fn set_addrsize(&mut self, addrsize: Size) {
        let sz = Self::compressed_size_rev(addrsize).unwrap_or(0b000) & 0b111;
        // clear addr size
        let mask = !0b1110_0000;
        // set addr_size
        self.metadata_2 = (self.metadata_2 & mask) | sz << 5;
    }
    // false if it fails
    // fails, if addrsize != base.size()
    pub fn set_base(&mut self, base: Register) -> bool {
        let bb = base.to_byte();
        let rx = base.needs_rex() as u8;

        // check
        if self.addrsize().unwrap_or(Size::Unknown) != base.size() {
            return false;
        }
        // set guardian
        self.regs = (self.regs & !0b1000_0000) | 0b1000_0000;

        // set rex
        let mask = !0b0000_0010;
        self.metadata_2 = (self.metadata_2 & mask) | rx << 1;
        // set base
        let mask = !0b0011_1000;
        self.regs = (self.regs & mask) | bb << 3;
        true
    }
    // false if it fails
    // fails, if addrsize != base.size()
    pub fn set_index(&mut self, index: Register) -> bool {
        let bb = index.to_byte();
        let rx = index.needs_rex() as u8;

        // check
        if self.addrsize().unwrap_or(Size::Unknown) != index.size() {
            return false;
        }
        // set guardian
        self.regs = (self.regs & !0b0100_0000) | 0b0100_0000;

        // set rex
        let mask = !0b0000_0001;
        self.metadata_2 = (self.metadata_2 & mask) | rx;
        // set index
        let mask = !0b0000_0111;
        self.regs = (self.regs & mask) | bb;
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
        let sz = Self::compressed_size_rev(sz).unwrap_or(0b1111);
        let mask = !0b1111_0000;
        self.metadata_1 = (self.metadata_1 & mask) | sz << 4
    }
    pub fn set_scale(&mut self, sz: Size) {
        let sz = Self::compressed_size_rev(sz).unwrap_or(0b111);
        let mask = !0b0000_1110;
        self.metadata_1 = (self.metadata_1 & mask) | (sz << 1);
    }
    pub fn set_flag(&mut self, idx: u8) {
        self.flags.set(idx, true)
    }
    pub fn clear_flag(&mut self, idx: u8) {
        self.flags.set(idx, false)
    }

    // misc
    fn compressed_reg(sz: Size, rex: bool, reg: u8) -> Option<Register> {
        use Register::*;
        match sz {
            Size::Byte => {
                if rex {
                    match reg {
                        0b000 => Some(R8B),
                        0b001 => Some(R9B),
                        0b010 => Some(R10B),
                        0b011 => Some(R11B),
                        0b100 => Some(R12B),
                        0b101 => Some(R13B),
                        0b110 => Some(R14B),
                        0b111 => Some(R15B),
                        _ => None,
                    }
                } else {
                    match reg {
                        0b000 => Some(AL),
                        0b001 => Some(CL),
                        0b010 => Some(DL),
                        0b011 => Some(BL),
                        0b100 => Some(AH),
                        0b101 => Some(CH),
                        0b110 => Some(DH),
                        0b111 => Some(BH),
                        _ => None,
                    }
                }
            }
            Size::Word => {
                if rex {
                    match reg {
                        0b000 => Some(R8W),
                        0b001 => Some(R9W),
                        0b010 => Some(R10W),
                        0b011 => Some(R11W),
                        0b100 => Some(R12W),
                        0b101 => Some(R13W),
                        0b110 => Some(R14W),
                        0b111 => Some(R15W),
                        _ => None,
                    }
                } else {
                    match reg {
                        0b000 => Some(AX),
                        0b001 => Some(CX),
                        0b010 => Some(DX),
                        0b011 => Some(BX),
                        0b100 => Some(SP),
                        0b101 => Some(BP),
                        0b110 => Some(SI),
                        0b111 => Some(DI),
                        _ => None,
                    }
                }
            }
            Size::Dword => {
                if rex {
                    match reg {
                        0b000 => Some(R8D),
                        0b001 => Some(R9D),
                        0b010 => Some(R10D),
                        0b011 => Some(R11D),
                        0b100 => Some(R12D),
                        0b101 => Some(R13D),
                        0b110 => Some(R14D),
                        0b111 => Some(R15D),
                        _ => None,
                    }
                } else {
                    match reg {
                        0b000 => Some(EAX),
                        0b001 => Some(ECX),
                        0b010 => Some(EDX),
                        0b011 => Some(EBX),
                        0b100 => Some(ESP),
                        0b101 => Some(EBP),
                        0b110 => Some(ESI),
                        0b111 => Some(EDI),
                        _ => None,
                    }
                }
            }
            Size::Qword => {
                if rex {
                    match reg {
                        0b000 => Some(R8),
                        0b001 => Some(R9),
                        0b010 => Some(R10),
                        0b011 => Some(R11),
                        0b100 => Some(R12),
                        0b101 => Some(R13),
                        0b110 => Some(R14),
                        0b111 => Some(R15),
                        _ => None,
                    }
                } else {
                    match reg {
                        0b000 => Some(RAX),
                        0b001 => Some(RCX),
                        0b010 => Some(RDX),
                        0b011 => Some(RBX),
                        0b100 => Some(RSP),
                        0b101 => Some(RBP),
                        0b110 => Some(RSI),
                        0b111 => Some(RDI),
                        _ => None,
                    }
                }
            }
            _ => None,
        }
    }
    fn compressed_size_rev(sz: Size) -> Option<u8> {
        match sz {
            Size::Byte => Some(0b0000),
            Size::Word => Some(0b0001),
            Size::Dword => Some(0b0010),
            Size::Qword => Some(0b0011),
            Size::Xword => Some(0b0100),
            Size::Yword => Some(0b0101),
            Size::Any => Some(0b1111),
            _ => None,
        }
    }
    fn compressed_size(sz: u8) -> Option<Size> {
        match sz {
            0b0000 => Some(Size::Byte),
            0b0001 => Some(Size::Word),
            0b0010 => Some(Size::Dword),
            0b0011 => Some(Size::Qword),
            0b0100 => Some(Size::Xword),
            0b0101 => Some(Size::Yword),
            0b1111 => Some(Size::Any),
            _ => None,
        }
    }
}

type Error = RASMError;

#[derive(PartialEq)]
enum Token {
    Register(Register),
    Number(i32),
    Error(Error),
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
    let scale = mem.scale();
    let offset = mem.offset();

    if let (None, Some(_)) = (base, index) {
        match index.unwrap().size() {
            Size::Word => mem.set_base(Register::BP),
            Size::Dword => mem.set_base(Register::EBP),
            Size::Qword => mem.set_base(Register::RBP),
            _ => false,
        };
    }

    if let (Some(_), Some(_), None) = (base, index, scale) {
        mem.set_scale(Size::Byte);
    }
    if let (None, None, Some(_)) = (base, index, offset) {
        mem.set_flag(RIP_ADDRESSING);
    }
}

fn mem_par_new(toks: Vec<Token>) -> Result<Mem, Error> {
    let mut mem = Mem::blank();

    let mut unspec_reg: Option<Register> = None;
    let mut base: Option<Register> = None;
    let mut index: Option<Register> = None;
    let mut offset: Option<i32> = None;
    let mut scale: Option<Size> = None;

    // if number was prefixed with *
    let mut mul_modf = false;
    let mut num_ismin = false;
    for tok in toks {
        match tok {
            Token::Error(e) => return Err(e),
            Token::Register(r) => {
                if unspec_reg.is_none() {
                    unspec_reg = Some(r);
                } else if base.is_none() {
                    base = Some(r);
                } else if index.is_none() {
                    index = Some(r);
                } else {
                    return Err(Error::no_tip(
                        None,
                        Some("Memory declaration has too many (3+) registers!"),
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
                            _ => return Err(Error::no_tip(
                                None,
                                Some(
                                    "Memory declaration's scale is larger than 8 (maximum scale)!",
                                ),
                            )),
                        }
                        scale = Some(sz);
                    } else {
                        return Err(Error::no_tip(
                            None,
                            Some("Memory declaration has scale, but it isn't either 1, 2, 4 or 8!"),
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
        if base.size() != index.size() {
            return Err(Error::no_tip(
                None,
                Some("Base and index registers in memory declaration have different sizes."),
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
                return Err(Error::no_tip(
                    None,
                    Some("Index has size that doesn't match 16/32/64 bits!"),
                ))
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

fn mem_tok_new(str: &str) -> Vec<Token> {
    let mut tokens = Vec::with_capacity(8);
    let bytes: &[u8] = str.as_bytes();
    let mut tmp_buf: Vec<u8> = Vec::with_capacity(16);
    for b in bytes {
        match b {
            b'*' => {
                if let Some(tok) = mem_tok_from_buf(&tmp_buf) {
                    tokens.push(tok);
                    tmp_buf.clear();
                }
                tokens.push(Token::Mul);
            }
            b'-' => {
                if let Some(tok) = mem_tok_from_buf(&tmp_buf) {
                    tokens.push(tok);
                    tmp_buf.clear();
                }
                tokens.push(Token::Sub);
            }
            b'+' => {
                if let Some(tok) = mem_tok_from_buf(&tmp_buf) {
                    tokens.push(tok);
                    tmp_buf.clear();
                }
                tokens.push(Token::Add);
            }
            b' ' | b'\t' => continue,
            _ => tmp_buf.push(*b),
        }
    }
    if !tmp_buf.is_empty() {
        if let Some(tok) = mem_tok_from_buf(&tmp_buf) {
            tokens.push(tok);
        }
    }

    tokens
}

fn mem_tok_from_buf(buf: &[u8]) -> Option<Token> {
    if buf.is_empty() {
        return None;
    }
    if let Some(prefix) = buf.first() {
        let utf8_buf = String::from_utf8_lossy(buf);
        if prefix == &b'%' {
            if let Ok(reg) = Register::from_str(&utf8_buf[1..]) {
                if reg.purpose() != RPurpose::General {
                    Some(Token::Error(Error::no_tip(
                        None,
                        Some("Tried to use register which purpose isn't general (like *ax, *bx, etc.)")
                    )))
                } else {
                    Some(Token::Register(reg))
                }
            } else {
                None
            }
        } else if prefix == &b'$' {
            match Number::from_str(&utf8_buf[1..]) {
                Ok(num) => Some(Token::Number(num.get_as_i32())),
                Err(error) => Some(Token::Error(error)),
            }
        } else {
            match Number::from_str(&utf8_buf) {
                Ok(num) => Some(Token::Number(num.get_as_i32())),
                Err(_) => None,
            }
        }
    } else {
        None
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Mem {
    fn to_string(&self) -> String {
        let mut str = String::new();
        str.push('(');
        if let Some(reg) = self.base() {
            str.push('%');
            str.push_str(&reg.to_string());
            if self.index().is_some() {
                str.push_str(" + ");
            }
        }
        if let Some(reg) = self.index() {
            str.push('%');
            str.push_str(&reg.to_string());
        }
        if let (Some(scale), Some(_)) = (self.scale(), self.index()) {
            str.push_str(" * ");
            str.push('$');
            str.push_str(&(<Size as Into<u8>>::into(scale).to_string()));
        }
        if let Some(offset) = self.offset() {
            let is_neg = offset.is_negative();
            let to_add = if is_neg { " - " } else { " + " };
            str.push_str(to_add);
            str.push('$');
            if is_neg {
                str.push_str(&offset.to_string()[1..]);
            } else {
                str.push_str(&offset.to_string());
            }
        }
        str.push(')');
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
        assert_eq!(mem.addrsize(), Some(Size::Qword));
        mem.set_addrsize(Size::Dword);
        assert_eq!(mem.addrsize(), Some(Size::Dword));
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
        assert_eq!(mem.scale(), Some(Size::Byte));
        let mut mem = Mem::blank();
        mem.set_addrsize(Size::Qword);
        mem.set_index(Register::RCX);
        assert_eq!(mem.index(), Some(Register::RCX));
    }
    #[test]
    fn mem_par_check() {
        let str = "%rax";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.index(), None);
        assert!(!mem.is_sib());
        assert_eq!(mem.base(), Some(Register::RAX));
        let str = "%rax + %rcx";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RAX));
        assert_eq!(mem.index(), Some(Register::RCX));
        assert_eq!(mem.scale(), Some(Size::Byte));
        let str = "%rax + $20";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RAX));
        assert_eq!(mem.offset(), Some(20));
        let str = "%rax + %rcx * $4 + $20";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RAX));
        assert_eq!(mem.index(), Some(Register::RCX));
        assert_eq!(mem.scale(), Some(Size::Dword));
        assert_eq!(mem.offset(), Some(20));
        let str = "%rcx*$4";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RBP));
        assert_eq!(mem.index(), Some(Register::RCX));
        assert_eq!(mem.scale(), Some(Size::Dword));
        let str = "%rcx * $4 + $20";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RBP));
        assert_eq!(mem.index(), Some(Register::RCX));
        assert_eq!(mem.scale(), Some(Size::Dword));
        assert_eq!(mem.offset(), Some(20));
        let str = "-0xFF";
        let mem = Mem::new(str, Size::Qword);
        assert!(mem.is_ok());
        let mem = mem.unwrap();
        assert_eq!(mem.base(), Some(Register::RIP));
        assert_eq!(mem.offset(), Some(-0xFF));
        let mem = Mem::new("%eax + %ebx", Size::Qword).unwrap();
        assert_eq!(mem.base(), Some(Register::EAX));
        assert_eq!(mem.index(), Some(Register::EBX));
        assert_eq!(mem.scale(), Some(Size::Byte));
    }
}
