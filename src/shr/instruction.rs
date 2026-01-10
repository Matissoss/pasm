// pasm - src/shr/instruction.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use std::{
    fmt::{Debug, Error as FmtError, Formatter},
    iter::Iterator,
    mem::{ManuallyDrop, MaybeUninit},
};

use crate::core::apx::APXVariant;
use crate::shr::{
    mnemonic::Mnemonic,
    mem::Mem,
    num::Number,
    reg::{Purpose as RPurpose, Register},
    size::Size,
    stackvec::StackVec,
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

//const FPFX_NONE: u16 = 0b0000;
const FPFX_VEX: u16 = 0b0001;
const FPFX_EVEX: u16 = 0b0010;
const FPFX_APX: u16 = 0b0011;

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
    //      if FPFX_EVEX:
    //          0bSZB0_MMM0_AEEE:
    //              - S: {sae}
    //              - Z: {z}
    //              - B: {bcst}
    //              - A: requires APX extension
    //              - EEE: er:
    //                  0b000 - none
    //                  0b001 - rn
    //                  0b010 - rd
    //                  0b011 - ru
    //                  0b100 - rz
    //              - MMM: {k0/1/2/3/4/5/6/7}
    //      if FPFX_APX:
    //          0b0ZZZ_0RRR_RRRR:
    //              ZZZ - APXVariant
    //              RRR_RRRR:
    //                  if EEVEX and EEVEX_COND:
    //                      COS_Z000:
    //                          C - CF
    //                          O - OF
    //                          S - SF
    //                          Z - ZF
    //                  if EEVEX and EEVEX_LEGACY:
    //                      NF0_0000
    //                          N - ND
    //                          F - NF
    //                  if EEVEX and EEVEX_VEX:
    //                      F00_0000:
    //                          F - NF
    //                  if EEVEX and EEVEX_EVEX:
    //                      000_0000
    //                  else:
    //                      reserved
    //      else:
    //          0000_0000_0000
    pub metadata: u16,

    // I'm forced to re-add it again
    // I'll try to remove it later, but it stays for now
    pub line: usize,
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
    pub fn with_operands(operands: StackVec<OperandOwned, 4>) -> Self {
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
    pub fn size_full_gt(&self) -> Size {
        let mut sz = Size::Byte;
        for o in self.iter() {
            let osz = o.size();
            if sz.se() < osz.se() {
                sz = osz;
            }
        }
        sz
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
    #[inline(always)]
    pub fn set_evex_bcst(&mut self) {
        self.set_fpfx(FPFX_EVEX);

        self.metadata &= !0b0000_0010_0000_0000;
        self.metadata |= 0b0000_0010_0000_0000;
    }
    #[inline(always)]
    pub fn evex_bcst(&self) -> Option<bool> {
        if self.is_evex() {
            Some(self.metadata & 0b0000_0010_0000_0000 == 0b0000_0010_0000_0000)
        } else {
            None
        }
    }

    // apx
    pub fn apx_set_eevex(&mut self) {
        self.set_apx();
        self.metadata &= !0b0100_0000_0000;
    }
    pub fn apx_set_rex2(&mut self) {
        self.set_apx();
        self.metadata &= !0b0100_0000_0000;
        self.metadata |= 0b0100_0000_0000;
    }
    pub fn set_apx(&mut self) {
        self.set_fpfx(FPFX_APX);
    }
    pub fn apx_get_eevex_mode(&self) -> Option<APXVariant> {
        if !self.is_apx() {
            return None;
        }
        // check for REX2
        if self.metadata & 0b1000_0000_0000 == 0b1000_0000_0000 {
            return None;
        }

        if self.metadata & 0b111_1111 == 0 {
            return None;
        }

        Some(unsafe { std::mem::transmute((self.metadata & 0b0011_0000_0000 >> 8) as u8) })
    }
    pub fn is_apx_eevex_cond(&self) -> bool {
        self.is_apx() && self.apx_get_eevex_mode() == Some(APXVariant::CondTestCmpExtension)
    }
    pub fn is_apx_eevex_vex(&self) -> bool {
        self.is_apx() && self.apx_get_eevex_mode() == Some(APXVariant::VexExtension)
    }
    pub fn is_apx_eevex_evex(&self) -> bool {
        self.is_apx() && self.apx_get_eevex_mode() == Some(APXVariant::EvexExtension)
    }
    pub fn is_apx_eevex_legc(&self) -> bool {
        self.is_apx() && self.apx_get_eevex_mode() == Some(APXVariant::LegacyExtension)
    }
    pub fn set_apx_legc(&mut self) {
        self.set_apx();
        self.metadata &= !0b0011_0000_0000;
        self.metadata |= (APXVariant::LegacyExtension as u16) << 8;
    }
    pub fn set_apx_vex(&mut self) {
        self.set_apx();
        self.metadata &= !0b0011_0000_0000;
        self.metadata |= (APXVariant::VexExtension as u16) << 8;
    }
    pub fn set_apx_evex(&mut self) {
        self.set_apx();
        self.metadata &= !0b0011_0000_0000;
        self.metadata |= (APXVariant::EvexExtension as u16) << 8;
    }
    pub fn set_apx_cond(&mut self) {
        self.set_apx();
        self.metadata &= !0b0011_0000_0000;
        self.metadata |= APXVariant::CondTestCmpExtension as u16;
    }

    pub fn apx_get_leg_nd(&self) -> Option<bool> {
        if !self.is_apx() {
            return None;
        }
        Some(self.metadata & 0b100_0000 == 0b100_0000)
    }
    pub fn apx_get_leg_nf(&self) -> Option<bool> {
        if !self.is_apx() {
            return None;
        }
        Some(self.metadata & 0b010_0000 == 0b010_0000)
    }
    pub fn apx_set_leg_nd(&mut self) {
        self.set_apx_legc();
        self.metadata &= !0b100_0000;
        self.metadata |= 0b100_0000;
    }
    pub fn apx_set_leg_nf(&mut self) {
        self.set_apx_legc();
        self.metadata &= !0b010_0000;
        self.metadata |= 0b010_0000;
    }
    pub fn apx_set_vex_nf(&mut self) {
        self.set_apx_vex();
        self.metadata &= !0b100_0000;
        self.metadata |= 0b100_0000;
    }
    pub fn apx_set_default(&mut self) {
        self.set_apx();
        self.metadata &= !0b0111_0000_0000;
        self.metadata |= (APXVariant::Auto as u16) << 8;
    }

    // be glad that these methods aren't called:
    // intel_apx_extended_evex_conditional_cmptest_set_cf
    // I'm just trying to make my code "readable", but Intel
    // does not want me to :D
    pub fn apx_evex_set_apx_extension(&mut self, b: bool) {
        self.set_fpfx(FPFX_EVEX);
        self.metadata &= !(1 << 3);
        self.metadata |= (b as u16) << 3;
    }
    pub fn apx_evex_requires_apx_extension(&self) -> Option<bool> {
        if !self.is_evex() {
            return None;
        }
        Some(self.metadata & 1 << 3 == 1 << 3)
    }
    pub fn apx_eevex_cond_set_cf(&mut self) {
        self.set_apx_cond();
        self.metadata |= 0b100_0000;
    }
    pub fn apx_eevex_cond_set_of(&mut self) {
        self.set_apx_cond();
        self.metadata |= 0b010_0000;
    }
    pub fn apx_eevex_cond_set_sf(&mut self) {
        self.set_apx_cond();
        self.metadata |= 0b001_0000;
    }
    pub fn apx_eevex_cond_set_zf(&mut self) {
        self.set_apx_cond();
        self.metadata |= 0b000_1000;
    }
    pub fn apx_eevex_vex_get_nf(&self) -> Option<bool> {
        if self.is_apx() {
            return None;
        }
        Some(self.metadata & 0b100_0000 == 0b100_0000)
    }
    pub fn apx_eevex_cond_get_cf(&self) -> Option<bool> {
        if self.is_apx() {
            return None;
        }
        Some(self.metadata & 0b100_0000 == 0b100_0000)
    }
    pub fn apx_eevex_cond_get_of(&self) -> Option<bool> {
        if self.is_apx() {
            return None;
        }
        Some(self.metadata & 0b010_0000 == 0b010_0000)
    }
    pub fn apx_eevex_cond_get_sf(&self) -> Option<bool> {
        if self.is_apx() {
            return None;
        }
        Some(self.metadata & 0b001_0000 == 0b001_0000)
    }
    pub fn apx_eevex_cond_get_zf(&self) -> Option<bool> {
        if self.is_apx() {
            return None;
        }
        Some(self.metadata & 0b000_1000 == 0b000_1000)
    }

    // operands
    #[inline(always)]
    pub fn iter(&'a self) -> impl Iterator<Item = Operand<'a>> {
        (0..self.len()).map(|o| self.get(o).unwrap())
    }

    pub fn needs_apx_extension(&self) -> bool {
        if self.is_evex() {
            return self.apx_evex_requires_apx_extension().unwrap_or(false);
        }
        if self.get_fpfx() == FPFX_APX {
            return true;
        }

        for i in 0..self.len() {
            if REG == self.gett(i) {
                let srg = unsafe { self.get_as_reg(i) };
                if srg.ebits()[0] && srg.purpose().is_gpr() {
                    return true;
                }
            } else if MEM == self.gett(i) {
                let mem = unsafe { self.get_as_mem(i) };
                if let Some(base) = mem.base() {
                    if base.ebits()[0] {
                        return true;
                    }
                } else if let Some(idx) = mem.index() {
                    if idx.ebits()[0] {
                        return true;
                    }
                }
            }
        }
        false
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
    pub fn get_symbs(&self) -> StackVec<(&SymbolRef<'_>, usize), 2> {
        let mut syms = StackVec::new();

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

impl Default for Instruction<'_> {
    fn default() -> Self {
        unsafe {
            Self {
                line: 0,
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
mod tests {
    use super::*;
    #[test]
    fn tinstruction_0() {
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
