// rasmx86_64 - src/shr/ast.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use std::path::PathBuf;

use crate::pre::tok::Token;
use crate::shr::{
    atype::{AType, ToAType},
    error::RASMError,
    ins::Mnemonic,
    math::MathematicalEvaluation as MathEval,
    mem::Mem,
    num::Number,
    reg::{Purpose as RPurpose, Register},
    section::Section,
    segment::Segment,
    size::Size,
    symbol::{SymbolRef, SymbolType, Visibility},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Reg(Register),
    SegReg(Register),
    CtrReg(Register),
    DbgReg(Register),
    Imm(Number),
    Mem(Mem),
    SymbolRef(String),
    SymbolRefExt(SymbolRef),
    String(String),
    Segment(Segment),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub oprs: [Option<Operand>; 5],
    pub line: usize,
    pub addt: Option<Mnemonic>,
    pub mnem: Mnemonic,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    Ins(Instruction),
    Attributes(String),
    Bits(u8),
    Entry(String),
    Label(String),
    Extern(String),
    Include(PathBuf),
    MathEval(String, String),

    Section(String),
    Align(u16),
    Exec,
    Write,
    Alloc,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Label {
    pub name: String,
    pub inst: Vec<Instruction>,
    pub shidx: usize,
    pub align: u16,
    pub stype: SymbolType,
    pub visibility: Visibility,
    pub bits: u8,
}

#[derive(Debug, Clone, Default)]
pub struct AST {
    pub sections: Vec<Section>,
    pub externs: Vec<String>,
    pub bits: Option<u8>,
    pub entry: Option<String>,
    pub includes: Vec<PathBuf>,
    pub math: Vec<(String, String)>,
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

// implementations

impl TryFrom<Token> for Operand {
    type Error = RASMError;
    fn try_from(tok: Token) -> Result<Self, <Self as TryFrom<Token>>::Error> {
        match tok {
            // experimental
            // idk what if this works (i hope so; will have to check)
            Token::Closure(' ', m) => match Mem::new(&m, Size::Any) {
                Ok(m) => Ok(Operand::Mem(m)),
                Err(e) => Err(e),
            },
            Token::Closure('$', m) => match MathEval::from_str(&m) {
                Ok(v) => {
                    let e = MathEval::eval(v);
                    if let Some(e) = e {
                        Ok(Self::Imm(Number::uint64(e)))
                    } else {
                        Err(Self::Error::no_tip(
                            None,
                            Some("Failed to evaluate mathematical expression"),
                        ))
                    }
                }
                Err(e) => Err(e),
            },
            Token::Register(reg) => {
                if reg.is_ctrl_reg() {
                    Ok(Self::CtrReg(reg))
                } else if reg.is_dbg_reg() {
                    Ok(Self::DbgReg(reg))
                } else if reg.purpose() == RPurpose::Sgmnt {
                    Ok(Self::SegReg(reg))
                } else {
                    Ok(Self::Reg(reg))
                }
            }
            Token::String(val) => Ok(Self::String(val)),
            Token::Immediate(nm) => Ok(Self::Imm(nm)),
            Token::SymbolRef(val) => Ok(Self::SymbolRef(val)),
            Token::SymbolRefExt(val) => Ok(Self::SymbolRefExt(val)),
            _ => Err(Self::Error::no_tip(None, Some("Failed to create operand!"))),
        }
    }
}

impl Operand {
    pub fn is_mem(&self) -> bool {
        matches!(self, Operand::Mem(_) | Operand::Segment(_))
    }
    pub fn get_mem(&self) -> Option<&Mem> {
        match self {
            Operand::Mem(m) => Some(m),
            Operand::Segment(Segment {
                segment: _,
                address: m,
            }) => Some(m),
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
            Self::SymbolRef(_) | Self::SymbolRefExt(_) => Size::Any,
            Self::Segment(s) => s.address.size().unwrap_or(Size::Unknown),
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
            Self::SymbolRef(_) | Self::SymbolRefExt(_) => AType::Symbol,
            Self::Segment(s) => s.address.atype(),
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
            Self::SymbolRef(_) | Self::SymbolRefExt(_) => AType::Symbol,
            Self::String(_) => AType::Immediate(Size::Unknown),
            Self::Segment(s) => s.address.atype(),
        }
    }
}

impl Instruction {
    pub fn which_variant(&self) -> IVariant {
        match self.dst() {
            Some(Operand::Reg(r)) => match r.size() {
                Size::Yword => IVariant::YMM,
                Size::Xword => IVariant::XMM,
                Size::Qword | Size::Dword => {
                    if r.purpose() == RPurpose::Mmx || r.purpose() == RPurpose::F128 {
                        IVariant::MMX
                    } else {
                        match self.src() {
                            Some(Operand::Reg(r)) => {
                                if r.purpose() == RPurpose::Mmx {
                                    IVariant::MMX
                                } else if r.purpose() == RPurpose::F256 {
                                    IVariant::YMM
                                } else if r.purpose() == RPurpose::F128 {
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
                        } else if r.purpose() == RPurpose::F128 {
                            IVariant::XMM
                        } else if r.purpose() == RPurpose::F128 {
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
        false
    }
    pub fn uses_cr(&self) -> bool {
        let dst = if let Some(dst) = self.dst() {
            dst
        } else {
            return false;
        };

        if let Operand::CtrReg(_) = dst {
            return true;
        }
        let src = if let Some(src) = self.src() {
            src
        } else {
            return false;
        };

        matches!(src, Operand::CtrReg(_))
    }
    pub fn uses_dr(&self) -> bool {
        let dst = if let Some(dst) = self.dst() {
            dst
        } else {
            return false;
        };

        if let Operand::DbgReg(_) = dst {
            return true;
        }
        let src = if let Some(src) = self.src() {
            src
        } else {
            return false;
        };

        matches!(src, Operand::DbgReg(_))
    }
    #[inline]
    pub fn dst(&self) -> Option<&Operand> {
        if let Some(Some(o)) = self.oprs.first() {
            Some(o)
        } else {
            None
        }
    }
    #[inline]
    pub fn reg_byte(&self, idx: usize) -> Option<u8> {
        if let Some(Some(Operand::Reg(r))) = self.oprs.get(idx) {
            Some(r.to_byte())
        } else {
            None
        }
    }
    #[inline]
    pub fn src(&self) -> Option<&Operand> {
        if let Some(Some(o)) = self.oprs.get(1) {
            Some(o)
        } else {
            None
        }
    }
    #[inline]
    pub fn src2(&self) -> Option<&Operand> {
        if let Some(Some(o)) = self.oprs.get(2) {
            Some(o)
        } else {
            None
        }
    }
    #[inline]
    // operand existence
    pub fn op_ex(&self) -> (bool, bool) {
        match (self.dst(), self.src()) {
            (Some(_), None) => (true, false),
            (Some(_), Some(_)) => (true, true),
            (None, Some(_)) => (false, true),
            (None, None) => (false, false),
        }
    }
    #[inline]
    pub fn get_opr(&self, idx: usize) -> Option<&Operand> {
        if let Some(Some(o)) = self.oprs.get(idx) {
            Some(o)
        } else {
            None
        }
    }
    #[inline]
    pub fn get_mem_idx(&self) -> Option<usize> {
        if let Some(Operand::Mem(_) | Operand::Segment(_)) = self.dst() {
            return Some(0);
        }
        if let Some(Operand::Mem(_) | Operand::Segment(_)) = self.src() {
            return Some(1);
        }
        if let Some(Operand::Mem(_) | Operand::Segment(_)) = self.src2() {
            return Some(2);
        }
        None
    }
    #[inline]
    pub fn get_mem(&self) -> Option<&Mem> {
        if let Some(idx) = self.get_mem_idx() {
            if let Some(
                Operand::Mem(m)
                | Operand::Segment(Segment {
                    segment: _,
                    address: m,
                }),
            ) = self.get_opr(idx)
            {
                Some(m)
            } else {
                None
            }
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
    pub fn fix_entry(&mut self) {
        if let Some(entry) = &self.entry {
            for index in 0..self.sections.len() {
                for label in &mut self.sections[index].content {
                    if &label.name == entry {
                        let (flabel, llabel) = self.sections[index].content.split_at_mut(index);
                        llabel[0].visibility = Visibility::Global;
                        std::mem::swap(&mut flabel[0], &mut llabel[0]);
                        return;
                    }
                }
            }
        }
    }
    pub fn extend(&mut self, rhs: Self) -> Result<(), RASMError> {
        for l in rhs.sections {
            if self.sections.contains(&l) {
                return Err(RASMError::no_tip(
                    None,
                    Some(format!("Multiple files contains label {}", l.name)),
                ));
            }
            self.sections.push(l);
        }
        for l in rhs.includes {
            if self.includes.contains(&l) {
                return Err(RASMError::no_tip(
                    None,
                    Some(format!(
                        "Multiple files contains include {}",
                        l.to_string_lossy()
                    )),
                ));
            }
            self.includes.push(l);
        }
        self.math.extend(rhs.math);
        Ok(())
    }
}
