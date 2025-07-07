// pasm - src/shr/ast.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use std::{
    collections::HashMap,
    fmt::{Debug, Error as FmtError, Formatter},
    iter::Iterator,
    mem::{ManuallyDrop, MaybeUninit},
    path::PathBuf,
};

use crate::shr::{
    error::Error,
    ins::Mnemonic,
    mem::Mem,
    num::Number,
    reg::{Purpose as RPurpose, Register},
    section::Section,
    size::Size,
    symbol::SymbolRef,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Operand<'a> {
    Register(Register),
    Imm(Number),
    Mem(Mem),
    SymbolRef(ManuallyDrop<Box<SymbolRef<'a>>>),
    String(ManuallyDrop<Box<&'a str>>),
}

#[allow(clippy::redundant_allocation)]
pub union OperandData<'a> {
    mem: Mem,
    sym: ManuallyDrop<Box<SymbolRef<'a>>>,
    str: ManuallyDrop<Box<&'a str>>,
    num: Number,
    reg: Register,
    oth: u64,
}

pub struct Instruction<'a> {
    // 32B
    pub operands: [OperandData<'a>; 4],
    // layout:
    // 0bXXXX_YYYY_ZZZZ_AAAA:
    // XXXX - operand type for 4th operand
    // YYYY - operand type for ssrc.
    // ZZZZ - operand type for src.
    // AAAA - operand type for dst.
    pub metadata: u16,
    pub mnemonic: Mnemonic,
    pub additional: MaybeUninit<Mnemonic>,
    // 0bMLLL_00CB_BBAZ_YXXX:
    // XXX - explicit prefix:
    //  0b000 - None
    //  0b001 - VEX
    //  0b010 - EVEX
    //  0b... - reserved
    // M  : has additional mnemonic
    // LLL: length
    // Y  : if XXX is EVEX: SAE
    // Z  : if XXX is EVEX: Z
    // E  : if XXX is EVEX: er
    // BBB: if XXX is EVEX: mask as byte
    // C  : has mask
    pub metadata_2: u16,
    // debug
    pub line: usize,
}

impl Clone for Instruction<'_> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for o in self.iter() {
            new.push(o);
        }
        new.line = self.line;
        new.metadata = self.metadata;
        new.metadata_2 = self.metadata_2;
        new.mnemonic = self.mnemonic;
        new.additional = self.additional;
        new
    }
}

impl Debug for Instruction<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(fmt, "Instruction {{")?;
        write!(fmt, "Additional: {:?}, ", self.addt())?;
        write!(fmt, "Mnemonic: {:?}, ", self.mnemonic)?;
        write!(fmt, "Operands: [")?;
        let mut i = 0;
        for o in self.iter() {
            write!(fmt, "{:?}", o)?;
            if i + 1 != self.len() {
                write!(fmt, ", ")?;
            }
            i += 1;
        }
        write!(fmt, "]")?;
        write!(fmt, "}}")?;
        Ok(())
    }
}

impl PartialEq for Instruction<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        self.line == rhs.line
    }
}

pub const REG: u16 = 0b001;
pub const MEM: u16 = 0b010;
pub const SYM: u16 = 0b011;
pub const STR: u16 = 0b100;
pub const IMM: u16 = 0b101;

impl<'a> Instruction<'a> {
    pub fn needs_evex(&self) -> bool {
        if self.metadata_2 & 0b111 == 0b010 {
            return true;
        }
        if self.get_mask().is_some() {
            return true;
        }
        if self.get_er() {
            return true;
        }
        if self.get_sae() {
            return true;
        }
        if self.get_z() {
            return true;
        }
        if self.get_bcst() {
            return true;
        }
        if self.size() == Size::Zword {
            return true;
        }
        for i in 0..self.len() {
            if REG == self.get_type(i) && unsafe { self.get_as_reg(i) }.get_ext_bits()[0] {
                return true;
            }
        }
        false
    }
    pub fn needs_rex(&self) -> bool {
        crate::core::rex::needs_rex(self, &self.dst(), &self.src())
    }
    pub const fn get_mask(&self) -> Option<u8> {
        if self.metadata_2 & 1 << 9 == 1 << 9 {
            Some(((self.metadata_2 & 0b111 << 6) >> 6) as u8)
        } else {
            None
        }
    }
    pub const fn set_mask(&mut self, val: u16) {
        self.metadata_2 |= 1 << 9;
        self.metadata_2 |= (val & 0b111) << 6;
    }
    pub const fn get_z(&self) -> bool {
        self.metadata_2 & 0b10000 == 0b10000
    }
    pub const fn set_z(&mut self) {
        self.metadata_2 |= 0b10000;
    }
    pub const fn get_sae(&self) -> bool {
        self.metadata_2 & 0b100000 == 0b100000
    }
    pub const fn set_sae(&mut self) {
        self.metadata_2 |= 0b100000;
    }
    pub const fn get_er(&self) -> bool {
        self.metadata_2 & 0b1000 == 0b1000
    }
    pub const fn set_er(&mut self) {
        self.metadata_2 |= 0b1000;
    }
    pub const fn set_evex(&mut self) {
        if self.metadata_2 & 0b111 == 0b000 {
            self.metadata_2 |= 0b010;
        }
    }
    pub const fn set_vex(&mut self) {
        if self.metadata_2 & 0b111 == 0b000 {
            self.metadata_2 |= 0b001;
        }
    }
    pub fn get_bcst(&self) -> bool {
        for i in 0..self.len() {
            if MEM == self.get_type(i) {
                return unsafe { self.get_as_mem(i).is_bcst() };
            }
        }
        false
    }
    pub const unsafe fn get_as_reg(&self, idx: usize) -> &Register {
        &self.operands[idx].reg
    }
    pub const unsafe fn get_as_mem(&self, idx: usize) -> &Mem {
        &self.operands[idx].mem
    }
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            operands: unsafe { MaybeUninit::uninit().assume_init() },
            metadata: 0,
            metadata_2: 0,
            mnemonic: Mnemonic::__LAST,
            additional: MaybeUninit::uninit(),
            line: 0,
        }
    }
    pub const fn set_addt(&mut self, addt: Option<Mnemonic>) {
        if let Some(mnem) = addt {
            self.metadata_2 |= 1 << 15;
            self.additional = MaybeUninit::new(mnem);
        }
    }
    pub const fn addt(&self) -> Option<Mnemonic> {
        if self.metadata_2 & 1 << 15 == 1 << 15 {
            Some(unsafe { self.additional.assume_init() })
        } else {
            None
        }
    }
    pub const fn set(&mut self, idx: usize, opr: Operand) {
        use std::mem::transmute;
        if idx < 4 {
            let idt = (idx as u16) << 2;
            let (con, opt) = unsafe {
                match opr {
                    Operand::Register(r) => (r as u64, REG),
                    Operand::Mem(m) => (transmute(m), MEM),
                    Operand::Imm(i) => (i.get_as_u64(), IMM),
                    Operand::String(s) => (transmute(s), STR),
                    Operand::SymbolRef(s) => (transmute(s), SYM),
                }
            };
            self.metadata &= !(0b1111 << idt);
            self.metadata |= opt << idt;
            self.operands[idx] = OperandData { oth: con };
        }
    }
    pub fn push(&mut self, opr: Operand) {
        self.set(self.len(), opr);
        let len = ((self.metadata_2 & (0b111 << 12)) >> 12) + 1;
        self.metadata_2 &= !(0b111 << 12);
        self.metadata_2 |= len << 12;
    }
    #[inline(always)]
    pub fn get_type(&'a self, idx: usize) -> u16 {
        if idx >= self.len() {
            return 0;
        }
        let idt = (idx as u16) << 2;
        (self.metadata & (0b1111 << idt)) >> idt
    }
    #[inline(always)]
    pub fn get(&'a self, idx: usize) -> Option<Operand<'a>> {
        if idx >= self.len() {
            return None;
        }
        let op = self.operands.get(idx).unwrap();
        let idt = (idx as u16) << 2;
        let opt = (self.metadata & (0b1111 << idt)) >> idt;
        unsafe {
            match opt {
                REG => Some(Operand::Register(op.reg)),
                IMM => Some(Operand::Imm(op.num)),
                MEM => Some(Operand::Mem(op.mem)),
                STR => Some(Operand::String(op.str.clone())),
                SYM => Some(Operand::SymbolRef(op.sym.clone())),
                _ => None,
            }
        }
    }
    #[inline(always)]
    pub fn src2(&'a self) -> Option<Operand<'a>> {
        self.get(2)
    }
    #[inline(always)]
    pub fn src(&'a self) -> Option<Operand<'a>> {
        self.get(1)
    }
    #[inline(always)]
    pub fn dst(&'a self) -> Option<Operand<'a>> {
        self.get(0)
    }
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub const fn len(&self) -> usize {
        ((self.metadata_2 & (0b111 << 12)) >> 12) as usize
    }
    pub fn iter(&'a self) -> impl Iterator<Item = Operand<'a>> {
        (0..self.len()).map(|i| self.get(i).unwrap())
    }
    pub fn which_variant(&self) -> IVariant {
        match self.dst() {
            Some(Operand::Register(r)) => match r.size() {
                Size::Yword => IVariant::YMM,
                Size::Xword => IVariant::XMM,
                Size::Qword | Size::Dword => {
                    if r.purpose() == RPurpose::Mmx || r.size() == Size::Xword {
                        IVariant::MMX
                    } else {
                        match self.src() {
                            Some(Operand::Register(r)) => {
                                if r.purpose() == RPurpose::Mmx {
                                    IVariant::MMX
                                } else if r.size() == Size::Yword {
                                    IVariant::YMM
                                } else if r.size() == Size::Xword {
                                    IVariant::XMM
                                } else {
                                    IVariant::STD
                                }
                            }
                            _ => IVariant::STD,
                        }
                    }
                }
                _ => IVariant::STD,
            },
            Some(Operand::Mem(m)) => match m.size() {
                Size::Yword => IVariant::YMM,
                Size::Xword => IVariant::XMM,
                Size::Qword | Size::Dword => match self.src() {
                    Some(Operand::Register(r)) => {
                        if r.purpose() == RPurpose::Mmx {
                            IVariant::MMX
                        } else if r.size() == Size::Xword {
                            IVariant::XMM
                        } else if r.size() == Size::Yword {
                            IVariant::YMM
                        } else {
                            IVariant::STD
                        }
                    }
                    _ => IVariant::STD,
                },
                _ => IVariant::STD,
            },
            _ => IVariant::STD,
        }
    }
    #[inline(always)]
    pub fn fast_size(dst: &Operand, src: &Operand) -> Size {
        let dst = dst.size();
        let src = src.size();
        if dst < src {
            src
        } else {
            dst
        }
    }
    pub fn size(&self) -> Size {
        let dst = match &self.dst() {
            Some(o) => o.size(),
            None => Size::Unknown,
        };
        let src = match &self.src() {
            Some(o) => o.size(),
            None => Size::Unknown,
        };

        if dst == Size::Unknown && src != Size::Unknown {
            src
        } else if dst != Size::Unknown && src == Size::Unknown {
            dst
        } else if dst < src {
            src
        } else {
            dst
        }
    }
    #[inline]
    pub fn uses_cr(&self) -> bool {
        for o in self.iter() {
            if let Operand::Register(r) = o {
                if r.is_ctrl_reg() {
                    return true;
                }
            }
        }
        false
    }
    #[inline]
    pub fn uses_dr(&self) -> bool {
        for o in self.iter() {
            if let Operand::Register(r) = o {
                if r.is_dbg_reg() {
                    return true;
                }
            }
        }
        false
    }
    #[inline]
    pub fn uses_rip(&self) -> bool {
        if let Some(m) = self.get_mem() {
            return m.is_riprel();
        }
        if self.get_symbs().iter().flatten().count() >= 1 {
            return true;
        }
        false
    }
    #[inline]
    pub fn reg_byte(&self, idx: usize) -> Option<u8> {
        if let Some(Operand::Register(r)) = self.get(idx) {
            Some(r.to_byte())
        } else {
            None
        }
    }
    #[inline]
    pub fn get_symbs(&self) -> [Option<(&ManuallyDrop<Box<SymbolRef>>, usize)>; 2] {
        let mut ops = [None, None];

        let mut idx = 0;
        while idx < self.len() {
            if SYM == self.get_type(idx) {
                let sym = unsafe { &self.operands[idx].sym };
                if sym.is_deref() {
                    ops[0] = Some((sym, idx));
                } else {
                    ops[1] = Some((sym, idx));
                }
            }
            idx += 1;
        }
        ops
    }
    #[inline]
    pub fn get_mem(&self) -> Option<Mem> {
        let idx = self.get_mem_idx()?;
        Some(unsafe { *self.get_as_mem(idx) })
    }
    #[inline]
    pub fn get_sib_idx(&self) -> Option<usize> {
        let idx = self.get_mem_idx()?;
        if self.get(idx)?.get_mem()?.is_sib() {
            Some(idx)
        } else {
            None
        }
    }
    #[inline]
    pub fn uses_sib(&self) -> bool {
        self.get_sib_idx().is_some()
    }
    #[inline]
    pub fn get_mem_idx(&self) -> Option<usize> {
        let mut idx = 0;
        while idx < 4 {
            if MEM == self.get_type(idx) {
                return Some(idx);
            }
            idx += 1;
        }
        None
    }
}

impl Drop for Instruction<'_> {
    fn drop(&mut self) {
        let mut idx = 0;
        while idx < 4 {
            let t = self.get_type(idx);
            if t == STR {
                unsafe { ManuallyDrop::drop(&mut self.operands[idx].str) }
            } else if t == SYM {
                unsafe { ManuallyDrop::drop(&mut self.operands[idx].sym) }
            }
            idx += 1;
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct AST<'a> {
    pub sections: Vec<Section<'a>>,
    pub defines: HashMap<&'a str, Number>,
    pub includes: Vec<PathBuf>,
    pub externs: Vec<&'a str>,

    pub format: Option<&'a str>,
    pub default_bits: Option<u8>,
    pub default_output: Option<PathBuf>,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum IVariant {
    #[default]
    STD,
    MMX,
    XMM, // SSE/AVX
    YMM, // AVX
}

impl Operand<'_> {
    pub fn get_reg(&self) -> Option<&Register> {
        match self {
            Operand::Register(r) => Some(r),
            _ => None,
        }
    }
    pub fn is_imm(&self) -> bool {
        matches!(self, Operand::Imm(_) | Operand::String(_))
    }
    pub fn is_mem(&self) -> bool {
        matches!(self, Operand::Mem(_))
    }
    pub fn get_mem(&self) -> Option<&Mem> {
        match self {
            Operand::Mem(m) => Some(m),
            _ => None,
        }
    }
    pub fn size(&self) -> Size {
        match self {
            Self::Imm(n) => n.size(),
            Self::Register(r) => r.size(),
            Self::Mem(m) => m.size(),
            Self::SymbolRef(s) => {
                if let Some(sz) = s.size() {
                    sz
                } else {
                    Size::Dword
                }
            }
            Self::String(_) => Size::Unknown,
        }
    }
}

impl AST<'_> {
    pub fn validate(&self) -> Result<(), Error> {
        use std::collections::HashSet;
        let iter = self.sections.iter().flat_map(|l| &l.content);
        let mut set: HashSet<&str> = HashSet::with_capacity(iter.count());
        for l in self.sections.iter().flat_map(|l| &l.content) {
            if !set.insert(l.name) {
                return Err(Error::new(
                    format!(
                        "file(s) contains multiple declarations of label of name \"{}\"",
                        l.name
                    ),
                    21,
                ));
            }
        }
        set.clear();
        for s in self.sections.iter() {
            if !set.insert(s.name) {
                return Err(Error::new(
                    format!(
                        "file(s) contains multiple declarations of sections of name \"{}\"",
                        s.name
                    ),
                    21,
                ));
            }
        }
        Ok(())
    }
    pub fn extend(&mut self, rhs: Self) -> Result<(), Error> {
        for l in rhs.sections {
            let attr = l.attributes;
            let align = l.align;
            let bits = l.bits;
            let name = l.name;
            self.sections.push(l);
            for s in 0..self.sections.len() - 1 {
                if self.sections[s].name == name {
                    if !(self.sections[s].bits == bits
                        && self.sections[s].align == align
                        && self.sections[s].attributes == attr)
                    {
                        return Err(
                            Error::new(
                                format!("if you changed one of \"{}\" to match the other one, then we could merge content of these sections", 
                                    self.sections[s].name), 12)
                        );
                    }
                    // section we pushed
                    let l = self.sections.pop().unwrap();
                    // concat two sections
                    for label in l.content {
                        for self_l in &self.sections[s].content {
                            if self_l.name == label.name {
                                return Err(Error::new(format!("failed to concat two sections as they contain same label of name \"{}\"", label.name), 12));
                            }
                        }
                        self.sections[s].content.push(label);
                    }
                    break;
                }
            }
        }
        for l in rhs.includes {
            if self.includes.contains(&l) {
                continue;
            }
            self.includes.push(l);
        }
        self.defines.extend(rhs.defines);
        Ok(())
    }
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn test() {
        assert_eq!(0b111 << (4 << (0 * 3)) >> (0 * 3) >> 4, 0b111);
        let mut ins = Instruction::new();
        ins.push(Operand::Imm(Number::uint64(10)));
        println!("{:016b}", ins.metadata);
        assert_eq!(ins.get(0), Some(Operand::Imm(Number::uint64(10))));
        assert_eq!(ins.len(), 1);
        ins.push(Operand::Register(Register::EAX));
        assert_eq!(ins.get(1), Some(Operand::Register(Register::EAX)));
        assert_eq!(ins.len(), 2);
        ins.set(0, Operand::Register(Register::EAX));
        assert_eq!(ins.get(0), Some(Operand::Register(Register::EAX)));
        assert_eq!(ins.len(), 2);
    }
}
