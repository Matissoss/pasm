// pasm - src/shr/ast.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

use std::path::PathBuf;

use std::collections::HashMap;

use crate::shr::{
    atype::{AType, ToAType},
    error::RError as Error,
    ins::Mnemonic,
    mem::Mem,
    num::Number,
    reg::{Purpose as RPurpose, Register},
    section::Section,
    size::Size,
    smallvec::SmallVec,
    symbol::{SymbolRef, SymbolType, Visibility},
};
use crate::RString;

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Reg(Register),
    SegReg(Register),
    CtrReg(Register),
    DbgReg(Register),
    Imm(Number),
    Mem(Mem),
    SymbolRef(SymbolRef),
    String(RString),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub oprs: SmallVec<Operand, 5>,
    pub line: usize,
    pub addt: Option<Mnemonic>,
    pub mnem: Mnemonic,

    // layout: 0b0SAE_XYYY:
    // S - uses {sae}
    // A - uses {z}
    // E - uses {er}
    // X - has mask
    // YYY - mask code
    pub meta: u16,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    Ins(Instruction),
    Attributes(RString),
    Bits(u8),
    Label(RString),
    Extern(RString),
    Include(PathBuf),
    Define(RString, u64),

    Section(RString),
    Align(u16),
    Exec,
    Write,
    Alloc,
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Label {
    pub name: RString,
    pub line: usize,
    pub inst: Vec<Instruction>,
    pub shidx: u16,
    pub align: u16,
    pub stype: SymbolType,
    // layout:
    // 0bXYYY_YZZZ
    // X : is global
    // YYYY: is reserved
    // ZZZ: bits: 0b000 - 16; 0b001 - 32; 0b010 - 64
    pub meta: u8,
}

impl Label {
    pub fn bits(&self) -> u8 {
        (self.meta & 0b111) << 4
    }
    pub fn visibility(&self) -> Visibility {
        if self.meta & 0b1000_0000 == 0b1000_0000 {
            Visibility::Global
        } else {
            Visibility::Local
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AST {
    pub sections: Vec<Section>,
    pub externs: Vec<RString>,
    pub bits: Option<u8>,
    pub entry: Option<RString>,
    pub includes: Vec<PathBuf>,
    pub defined: HashMap<RString, u64>,
    pub file: PathBuf,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum IVariant {
    #[default]
    STD,
    MMX,
    XMM, // SSE/AVX
    YMM, // AVX
}

impl Operand {
    pub fn get_reg(&self) -> Option<&Register> {
        match self {
            Operand::Reg(r) | Operand::SegReg(r) | Operand::CtrReg(r) | Operand::DbgReg(r) => {
                Some(r)
            }
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
            Self::Reg(r) => r.size(),
            Self::CtrReg(r) => r.size(),
            Self::DbgReg(r) => r.size(),
            Self::Mem(m) => m.size().unwrap_or(Size::Unknown),
            Self::SymbolRef(s) => {
                if let Some(sz) = s.size() {
                    sz
                } else {
                    Size::Dword
                }
            }
            Self::SegReg(_) => Size::Word,
            Self::String(_) => Size::Unknown,
        }
    }
    pub fn ext_atype(&self) -> AType {
        match self {
            Self::Mem(m) => m.atype(),
            Self::CtrReg(r) | Self::SegReg(r) | Self::DbgReg(r) | Self::Reg(r) => {
                AType::ExtendedRegister(*r)
            }
            Self::Imm(n) => n.atype(),
            Self::SymbolRef(s) => {
                if s.is_deref() {
                    AType::Memory(s.size().unwrap_or(Size::Unknown))
                } else {
                    AType::Immediate(Size::Dword)
                }
            }
            Self::String(_) => AType::Immediate(Size::Unknown),
        }
    }
}

impl ToAType for Operand {
    fn atype(&self) -> AType {
        match self {
            Self::Mem(m) => m.atype(),
            Self::CtrReg(r) | Self::SegReg(r) | Self::DbgReg(r) | Self::Reg(r) => r.atype(),
            Self::Imm(n) => n.atype(),
            Self::SymbolRef(s) => {
                if s.is_deref() {
                    AType::Memory(s.size().unwrap_or(Size::Unknown))
                } else {
                    AType::Immediate(Size::Dword)
                }
            }
            Self::String(s) => match s.len() {
                1 => AType::Immediate(Size::Byte),
                2 => AType::Immediate(Size::Word),
                3..=4 => AType::Immediate(Size::Dword),
                5..=8 => AType::Immediate(Size::Qword),
                _ => AType::Immediate(Size::Unknown),
            },
        }
    }
}
impl Instruction {
    pub fn get_bcst(&self) -> bool {
        if let Some(m) = self.get_mem() {
            return m.is_bcst();
        }
        false
    }
    pub const fn get_er(&self) -> bool {
        self.meta & 0b0001_0000 == 0b0001_0000
    }
    pub const fn set_er(&mut self) {
        self.meta |= 0b0001_0000;
    }
    pub const fn set_z(&mut self) {
        self.meta |= 0b0010_0000;
    }
    pub const fn get_z(&self) -> bool {
        self.meta & 0b0010_0000 == 0b0010_0000
    }
    pub const fn set_sae(&mut self) {
        self.meta |= 0b0100_0000;
    }
    pub const fn get_sae(&self) -> bool {
        self.meta & 0b0100_0000 == 0b0100_0000
    }
    pub const fn get_mask(&self) -> Option<Register> {
        let has_mask = self.meta & 0b1000;
        if has_mask == 0b1000 {
            Some(match self.meta & 0b111 {
                0b000 => Register::K0,
                0b001 => Register::K1,
                0b010 => Register::K2,
                0b011 => Register::K3,
                0b100 => Register::K4,
                0b101 => Register::K5,
                0b110 => Register::K6,
                0b111 => Register::K7,
                _ => Register::__ANY,
            })
        } else {
            None
        }
    }
    pub const fn set_mask(&mut self, m: u16) {
        self.meta |= m & 0b111;
        self.meta |= 0b1000;
    }

    pub fn needs_evex(&self) -> bool {
        if self.size() == Size::Zword {
            return true;
        }
        for o in self.oprs.iter() {
            if let Operand::Reg(r) = o {
                if r.needs_evex() {
                    return true;
                }
            }
        }
        false
    }
    pub fn which_variant(&self) -> IVariant {
        match self.dst() {
            Some(Operand::Reg(r)) => match r.size() {
                Size::Yword => IVariant::YMM,
                Size::Xword => IVariant::XMM,
                Size::Qword | Size::Dword => {
                    if r.purpose() == RPurpose::Mmx || r.size() == Size::Xword {
                        IVariant::MMX
                    } else {
                        match self.src() {
                            Some(Operand::Reg(r)) => {
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
            Some(Operand::Mem(m)) => match m.size().unwrap_or(Size::Unknown) {
                Size::Yword => IVariant::YMM,
                Size::Xword => IVariant::XMM,
                Size::Qword | Size::Dword => match self.src() {
                    Some(Operand::Reg(r)) => {
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
    pub fn uses_rip(&self) -> bool {
        if let Some(m) = self.get_mem() {
            return m.is_riprel();
        }
        if self.get_symbs().iter().flatten().count() >= 1 {
            return true;
        }
        false
    }
    pub fn uses_cr(&self) -> bool {
        for o in self.oprs.iter() {
            if let Operand::CtrReg(_) = o {
                return true;
            }
        }
        false
    }
    pub fn uses_dr(&self) -> bool {
        for o in self.oprs.iter() {
            if let Operand::DbgReg(_) = o {
                return true;
            }
        }
        false
    }
    #[inline]
    pub fn dst(&self) -> Option<&Operand> {
        self.oprs.first()
    }
    #[inline]
    pub fn reg_byte(&self, idx: usize) -> Option<u8> {
        if let Some(Operand::Reg(r)) = self.oprs.get(idx) {
            Some(r.to_byte())
        } else {
            None
        }
    }
    #[inline]
    pub fn src(&self) -> Option<&Operand> {
        self.oprs.get(1)
    }
    #[inline]
    pub fn src2(&self) -> Option<&Operand> {
        self.oprs.get(2)
    }
    #[inline]
    pub fn get_opr(&self, idx: usize) -> Option<&Operand> {
        self.oprs.get(idx)
    }
    #[inline]
    pub fn get_mem_idx(&self) -> Option<usize> {
        for (i, o) in self.oprs.iter().enumerate() {
            if o.is_mem() {
                return Some(i);
            }
        }
        None
    }
    //                                  operand,  index
    pub fn get_symbs(&self) -> [Option<(&SymbolRef, usize)>; 2] {
        let mut ops = [None, None];
        for (idx, s) in self.oprs.iter().enumerate() {
            if let Operand::SymbolRef(s) = s {
                if s.is_deref() {
                    ops[0] = Some((s, idx));
                } else {
                    ops[1] = Some((s, idx));
                }
            }
        }
        ops
    }
    #[inline]
    pub fn get_mem(&self) -> Option<&Mem> {
        let idx = self.get_mem_idx()?;
        if let Operand::Mem(m) = self.get_opr(idx)? {
            Some(m)
        } else {
            None
        }
    }
    #[inline]
    pub fn get_sib_idx(&self) -> Option<usize> {
        let idx = self.get_mem_idx()?;
        if self.get_opr(idx)?.get_mem()?.is_sib() {
            Some(idx)
        } else {
            None
        }
    }
    #[inline]
    pub fn uses_sib(&self) -> bool {
        self.get_sib_idx().is_some()
    }
}

impl AST {
    pub fn validate(&self) -> Result<(), Error> {
        for (i0, s0) in self.sections.iter().enumerate() {
            for (i1, s1) in self.sections.iter().enumerate() {
                if i0 == i1 {
                    continue;
                }

                if s0.name == s1.name {
                    return Err(Error::new(
                        format!("this file contains multiple sections named \"{}\"", s0.name),
                        12,
                    ));
                }
            }
            for (i0, l0) in s0.content.iter().enumerate() {
                for (i1, l1) in s0.content.iter().enumerate() {
                    if i0 == i1 {
                        continue;
                    }
                    if l0.name == l1.name {
                        return Err(Error::new_wline_actx(format!("section {} contains two labels with name \"{}\". Declaration at line {}, redeclaration at line {}",  s0.name, l1.name,l0.line,l1.line), 12, l0.line, l1.line));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn extend(&mut self, rhs: Self) -> Result<(), Error> {
        for l in rhs.sections {
            let attr = l.attributes;
            let align = l.align;
            let bits = l.bits;
            let name = l.name.clone();
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
        self.defined.extend(rhs.defined);
        Ok(())
    }
}
