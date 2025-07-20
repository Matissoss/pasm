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
    smallvec::SmallVec,
    symbol::SymbolRef,
};

pub const REG: u16 = 0b001;
pub const MEM: u16 = 0b010;
pub const SYM: u16 = 0b011;
pub const STR: u16 = 0b100;
pub const IMM: u16 = 0b101;

const ADDT_MASK: u16 = !0b0001_0000_0000_0000;
const LEN_MASK: u16 = !0b1110_0000_0000_0000;
const DST_MASK: u16 = !0b0000_0000_0000_0111;
const SRC_MASK: u16 = !0b0000_0000_0011_1000;
const SSRC_MASK: u16 = !0b0000_0001_1100_0000;
const TSRC_MASK: u16 = !0b0000_1110_0000_0000;
const FPFX_MASK: u16 = !0b1111_0000_0000_0000;

#[allow(unused)]
const FPFX_NONE: u16 = 0b0000;
const FPFX_VEX: u16 = 0b0001;
const FPFX_EVEX: u16 = 0b0010;
const FPFX_APX: u16 = 0b0011;

#[derive(Default, Debug)]
pub struct AST<'a> {
    pub sections: Vec<Section<'a>>,
    pub defines: HashMap<&'a str, Number>,
    pub includes: Vec<PathBuf>,
    pub externs: Vec<&'a str>,

    pub format: Option<&'a str>,
    pub default_bits: Option<u8>,
    pub default_output: Option<PathBuf>,
    pub blank_lines: Vec<usize>,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum IVariant {
    #[default]
    STD,
    MMX,
    XMM, // SSE/AVX
    YMM, // AVX
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperandOwned<'a> {
    Register(Register),
    Imm(Number),
    Mem(Mem),
    Symbol(ManuallyDrop<Box<SymbolRef<'a>>>),
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

#[derive(PartialEq, Debug)]
pub enum Operand<'a> {
    String(&'a &'a str),
    Symbol(&'a SymbolRef<'a>),
    Register(Register),
    Mem(Mem),
    Imm(Number),
}

pub struct Instruction<'a> {
    pub operands: [OperandData<'a>; 4],
    pub mnemonic: Mnemonic,
    pub additional: MaybeUninit<Mnemonic>,

    // layout:
    //
    //  0bLLLM_XXX_YYY_ZZZ_AAA:
    //   - XXX: operand type for 4th operand
    //   - YYY: operand type for ssrc.
    //   - ZZZ: operand type for src.
    //   - AAA: operand type for dst.
    //   - LLL: length
    //   - M  : has additional mnemonic
    pub operand_data: u16,
    //  0bXXXX_RRRR_RRRR_RRRR
    //   - XXXX: forced prefix (fpfx)
    //      0b0000 - None
    //      0b0001 - VEX
    //      0b0010 - EVEX
    //      0b0011 - APX (variants)
    //      0b.... - reserved
    //   - RRRR_RRRR_RRRR - forced prefix specific:
    //      if EVEX:
    //          0bSZ00_MMM0_0EEE:
    //              - S: {sae}
    //              - Z: {z}
    //              - EEE: er:
    //                  0b000 - none
    //                  0b001 - rn
    //                  0b010 - rd
    //                  0b011 - ru
    //                  0b100 - rz
    //              - MMM: {k0/1/2/3/4/5/6/7}
    //      if APX:
    //          0bMRRR_RRRR_RRRR:
    //              - M: if 0 then EEVEX, if 1 then REX2
    //              - RRR_RRRR_RRRR:
    //                  if EEVEX:
    //                      [...]
    //                  if REX2:
    //                      [...]
    //      else:
    //          reserved
    pub metadata: u16,
}

impl<'a> Instruction<'a> {
    #[inline(always)]
    const fn get_op_mask(&self, idx: usize) -> u16 {
        match idx {
            0 => DST_MASK,
            1 => SRC_MASK,
            2 => SSRC_MASK,
            3 => TSRC_MASK,
            _ => 0,
        }
    }
    #[inline(always)]
    pub fn push(&mut self, opr: OperandOwned) {
        self.set(self.len(), opr);
        self.set_len(self.len() + 1);
    }
    #[inline(always)]
    pub fn with_operands(operands: SmallVec<OperandOwned, 4>) -> Self {
        let mut ins = Self::default();
        ins.set_len(operands.len());
        let mut idx = 0;
        for o in operands.into_iter() {
            ins.set(idx, o);
            idx += 1;
        }
        ins
    }
    #[inline(always)]
    pub fn set_len(&mut self, len: usize) {
        self.operand_data &= LEN_MASK;
        self.operand_data |= (len as u16 & 0b111) << 13;
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        ((self.operand_data & !LEN_MASK) >> 13) as usize
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    // get (operand) type
    #[inline(always)]
    pub fn gett(&self, idx: usize) -> u16 {
        let mask = self.get_op_mask(idx);
        (self.operand_data & !mask) >> (idx as u16 * 3)
    }
    #[inline(always)]
    pub fn set(&mut self, idx: usize, opr: OperandOwned) {
        use std::mem::transmute;
        self.operand_data &= self.get_op_mask(idx);
        let (val, opt): (u64, u16) = unsafe {
            match opr {
                OperandOwned::Mem(m) => (transmute(m), MEM),
                OperandOwned::Register(r) => (r.0 as u64, REG),
                OperandOwned::String(s) => (transmute(s), STR),
                OperandOwned::Symbol(s) => (transmute(s), SYM),
                OperandOwned::Imm(i) => (transmute(i), IMM),
            }
        };
        self.operands[idx].oth = val;
        self.operand_data |= opt << (idx * 3) as u16;
    }
    #[inline(always)]
    pub fn get(&'a self, idx: usize) -> Option<Operand<'a>> {
        let ot = self.gett(idx);
        if ot != 0 {
            let od = &self.operands[idx];
            unsafe {
                match ot {
                    REG => Some(Operand::Register(od.reg)),
                    MEM => Some(Operand::Mem(od.mem)),
                    IMM => Some(Operand::Imm(od.num)),
                    STR => Some(Operand::String(&od.str)),
                    SYM => Some(Operand::Symbol(&od.sym)),
                    _ => None,
                }
            }
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn dst(&'a self) -> Option<Operand<'a>> {
        self.get(0)
    }
    #[inline(always)]
    pub fn src(&'a self) -> Option<Operand<'a>> {
        self.get(1)
    }
    #[inline(always)]
    pub fn ssrc(&'a self) -> Option<Operand<'a>> {
        self.get(2)
    }
    #[inline(always)]
    pub fn tsrc(&'a self) -> Option<Operand<'a>> {
        self.get(3)
    }
    #[inline(always)]
    pub fn size_lt(&self) -> Size {
        let dst = match &self.dst() {
            Some(o) => o.size(),
            None => return Size::Unknown,
        };
        let src = match &self.src() {
            Some(o) => o.size(),
            None => return dst,
        };

        if dst > src {
            src
        } else {
            dst
        }
    }
    #[inline(always)]
    pub fn size_gt(&self) -> Size {
        let dst = match &self.dst() {
            Some(o) => o.size(),
            None => return Size::Unknown,
        };
        let src = match &self.src() {
            Some(o) => o.size(),
            None => return dst,
        };

        if dst < src {
            src
        } else {
            dst
        }
    }
    #[inline(always)]
    pub fn size(&self) -> Size {
        self.size_gt()
    }

    #[inline(always)]
    pub const fn set_addt(&mut self, am: Mnemonic) {
        self.operand_data &= ADDT_MASK;
        self.operand_data |= 1 << 12;
        self.additional = MaybeUninit::new(am);
    }
    #[inline(always)]
    pub const fn get_addt(&self) -> Option<Mnemonic> {
        if (self.operand_data & !ADDT_MASK) >> 12 == 1 {
            unsafe { Some(self.additional.assume_init()) }
        } else {
            None
        }
    }
    // metadata
    #[inline(always)]
    pub const fn set_vex(&mut self) {
        self.set_fpfx(FPFX_VEX);
    }
    #[inline(always)]
    pub const fn set_evex(&mut self) {
        self.set_fpfx(FPFX_EVEX);
    }

    #[inline(always)]
    pub const fn set_fpfx(&mut self, v: u16) {
        self.metadata &= FPFX_MASK;
        self.metadata |= v << 12;
    }
    #[inline(always)]
    pub const fn get_fpfx(&self) -> u16 {
        (self.metadata & !FPFX_MASK) >> 12
    }

    #[inline(always)]
    pub const fn is_evex(&self) -> bool {
        self.get_fpfx() == FPFX_EVEX
    }
    #[inline(always)]
    pub const fn is_vex(&self) -> bool {
        self.get_fpfx() == FPFX_VEX
    }
    #[inline(always)]
    pub const fn is_apx(&self) -> bool {
        self.get_fpfx() == FPFX_APX
    }
    #[inline(always)]
    pub const fn is_eevex(&self) -> bool {
        self.get_fpfx() == FPFX_APX && self.apx_is_eevex().unwrap()
    }
    #[inline(always)]
    pub const fn is_rex2(&self) -> bool {
        self.get_fpfx() == FPFX_APX && self.apx_is_rex2().unwrap()
    }

    // evex
    #[inline(always)]
    pub fn set_evex_er(&mut self, vl: u8) {
        self.set_fpfx(FPFX_EVEX);
        self.metadata &= !0b0000_0000_0000_0111;
        self.metadata |= vl as u16 & 0b111;
    }
    #[inline(always)]
    pub const fn evex_er(&self) -> Option<u8> {
        if !self.is_evex() {
            return None;
        }
        if self.metadata & 0b111 == 0 {
            return None;
        }
        Some((self.metadata as u8 & 0b111) - 1)
    }
    #[inline(always)]
    pub fn set_evex_mask(&mut self, o: u8) {
        self.set_fpfx(FPFX_EVEX);

        self.metadata &= !0b0000_0000_0111_0000;
        self.metadata |= (o as u16 & 0b111) << 4;
    }
    #[inline(always)]
    pub fn evex_mask(&self) -> Option<u8> {
        if self.is_evex() {
            Some(((self.metadata & 0b0000_0000_0111_0000) >> 4) as u8)
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn set_evex_z(&mut self) {
        self.set_fpfx(FPFX_EVEX);

        self.metadata &= !0b0000_0100_0000_0000;
        self.metadata |= 0b0000_0100_0000_0000;
    }
    #[inline(always)]
    pub fn evex_z(&self) -> Option<bool> {
        if self.is_evex() {
            Some(self.metadata & 0b0000_0100_0000_0000 == 0b0000_0100_0000_0000)
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn set_evex_sae(&mut self) {
        self.set_fpfx(FPFX_EVEX);

        self.metadata &= !0b0000_1000_0000_0000;
        self.metadata |= 0b0000_1000_0000_0000;
    }
    #[inline(always)]
    pub fn evex_sae(&self) -> Option<bool> {
        if self.is_evex() {
            Some(self.metadata & 0b0000_1000_0000_0000 == 0b0000_1000_0000_0000)
        } else {
            None
        }
    }

    // apx
    #[inline(always)]
    pub fn set_apx_eevex(&mut self) {
        self.metadata &= !0b0000_1000_0000_0000;
    }
    #[inline(always)]
    pub fn set_apx_rex2(&mut self) {
        self.metadata &= !0b0000_1000_0000_0000;
        self.metadata |= 0b0000_1000_0000_0000;
    }
    #[inline(always)]
    pub const fn apx_is_eevex(&self) -> Option<bool> {
        if let Some(b) = self.apx_is_rex2() {
            Some(!b)
        } else {
            None
        }
    }
    #[inline(always)]
    pub const fn apx_is_rex2(&self) -> Option<bool> {
        if self.is_apx() {
            Some(self.metadata & 0b0000_1000_0000_0000 == 0b0000_1000_0000_0000)
        } else {
            None
        }
    }

    // operands
    #[inline(always)]
    pub fn iter(&'a self) -> impl Iterator<Item = Operand<'a>> {
        (0..self.len()).map(|o| self.get(o).unwrap())
    }

    // legacy
    #[inline(always)]
    pub fn needs_evex(&self) -> bool {
        if self.is_evex() {
            return true;
        }
        if self.size() == Size::Zword {
            return true;
        }
        for i in 0..self.len() {
            if REG == self.gett(i) && unsafe { self.get_as_reg(i) }.ebits()[0] {
                return true;
            }
        }
        false
    }
    #[inline(always)]
    pub fn needs_rex(&self) -> bool {
        crate::core::rex::needs_rex(self, &self.dst(), &self.src())
    }
    pub fn get_bcst(&self) -> bool {
        for i in 0..self.len() {
            if MEM == self.gett(i) {
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
        if !self.get_symbs().is_empty() {
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
    pub fn get_symbs(&self) -> SmallVec<(&SymbolRef, usize), 2> {
        let mut syms = SmallVec::new();

        let mut idx = 0;
        while idx < self.len() {
            if SYM == self.gett(idx) {
                let sym = unsafe { &**self.operands[idx].sym };
                syms.push((sym, idx));
            }
            idx += 1;
        }
        syms
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
            if MEM == self.gett(idx) {
                return Some(idx);
            }
            idx += 1;
        }
        None
    }
}

impl Drop for Instruction<'_> {
    fn drop(&mut self) {
        for i in 0..self.len() {
            let ot = self.gett(i);
            unsafe {
                match ot {
                    SYM => ManuallyDrop::drop(&mut self.operands[i].sym),
                    STR => ManuallyDrop::drop(&mut self.operands[i].str),
                    _ => {}
                }
            }
        }
    }
}

impl Debug for Instruction<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(fmt, "Instruction {{")?;
        write!(fmt, "Additional: {:?}, ", self.get_addt())?;
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
        if self.mnemonic != rhs.mnemonic || self.len() != rhs.len() {
            return false;
        }
        for i in 0..self.len() {
            if self.get(i) != rhs.get(i) {
                return false;
            }
        }
        true
    }
}

impl Operand<'_> {
    pub fn ebits(&self) -> [[bool; 2]; 2] {
        match self {
            Self::Register(r) => [r.ebits(), [false; 2]],
            Self::Mem(m) => [
                if let Some(r) = m.base() {
                    r.ebits()
                } else {
                    [false; 2]
                },
                if let Some(r) = m.index() {
                    r.ebits()
                } else {
                    [false; 2]
                },
            ],
            _ => [[false; 2]; 2],
        }
    }
    pub fn get_reg(&self) -> Option<&Register> {
        match self {
            Operand::Register(r) => Some(r),
            _ => None,
        }
    }
    pub fn is_reg(&self) -> bool {
        matches!(self, Operand::Register(_))
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
            Self::Imm(n) => n.signed_size(),
            Self::Register(r) => r.size(),
            Self::Mem(m) => m.size(),
            Self::Symbol(s) => {
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
    pub fn effective_line(&self, base_line: usize) -> usize {
        if self.blank_lines.is_empty() {
            return base_line;
        }
        let mut nline = base_line;
        for line in &self.blank_lines {
            if *line < base_line {
                nline += 1;
            } else {
                break;
            }
        }
        nline
    }
    pub fn validate(&self) -> Result<(), Error> {
        use std::collections::HashSet;
        let iter = self.sections.iter().flat_map(|l| &l.content);
        let mut set: HashSet<&str> = HashSet::with_capacity(iter.count());
        for l in self.sections.iter().flat_map(|l| &l.content) {
            if !l.name.is_empty() && !set.insert(l.name) {
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

impl Default for Instruction<'_> {
    fn default() -> Self {
        unsafe {
            Self {
                mnemonic: Mnemonic::__LAST,
                operand_data: 0,
                metadata: 0,
                additional: MaybeUninit::uninit(),
                operands: MaybeUninit::uninit().assume_init_read(),
            }
        }
    }
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn test() {
        assert_eq!(0b111 << (4 << (0 * 3)) >> (0 * 3) >> 4, 0b111);
        let mut ins = Instruction::default();
        ins.set(0, OperandOwned::Imm(Number::uint64(10)));
        ins.set_len(1);
        assert_eq!(ins.get(0), Some(Operand::Imm(Number::uint64(10))));
        ins.set(1, OperandOwned::Register(Register::EAX));
        ins.set_len(2);
        assert_eq!(ins.get(1), Some(Operand::Register(Register::EAX)));
        assert_eq!(ins.len(), 2);
        ins.set(0, OperandOwned::Register(Register::EAX));
        assert_eq!(ins.get(0), Some(Operand::Register(Register::EAX)));
        assert_eq!(ins.len(), 2);
        ins.set_fpfx(FPFX_EVEX);
        ins.set_fpfx(FPFX_EVEX);
        assert_eq!(ins.is_evex(), true);
        ins.set_evex_mask(1);
        assert_eq!(ins.evex_mask(), Some(1));
        ins.set_evex_z();
        assert_eq!(ins.evex_z(), Some(true));
    }
}
