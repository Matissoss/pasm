// rasmx86_64 - src/shr/ast.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;

use crate::pre::tok::Token;
use crate::shr::{
    atype::{AType, ToAType},
    ins::Mnemonic as Mnm,
    mem::Mem,
    num::Number,
    reg::{Purpose as RPurpose, Register},
    segment::Segment,
    size::Size,
    symbol::Visibility,
    var::{VType, Variable},
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
    Segment(Segment),
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub mnem: Mnm,
    pub addt: Option<Vec<Mnm>>,
    pub oprs: Vec<Operand>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum ASTNode<'a> {
    Ins(Instruction),
    Bits(u8),
    Entry(String),
    Label(String),
    Extern(String),
    Global(String),
    Variable(Variable<'a>),
}

#[derive(Debug, Clone, Default)]
pub struct Label<'a> {
    pub name: Cow<'a, String>,
    pub inst: Vec<Instruction>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Default)]
pub struct AST<'a> {
    pub labels: Vec<Label<'a>>,
    pub vars: Vec<Variable<'a>>,
    pub globals: Vec<String>,
    pub externs: Vec<String>,
    pub bits: Option<u8>,
    pub entry: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolDeclaration {
    name: String,
    size: Option<u32>,
    cont: Option<SymbolValue>,
    stype: Section,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SymbolValue {
    Number(Number),
    String(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Section {
    Data,
    Bss,
    Readonly,
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

impl TryFrom<&Token> for Operand {
    type Error = ();
    fn try_from(tok: &Token) -> Result<Self, <Self as TryFrom<&Token>>::Error> {
        match tok {
            // experimental
            // idk what if this works (i hope so; will have to check)
            Token::MemAddr(m) => match Mem::try_make(m, Some(super::kwd::Keyword::Any)) {
                Ok(m) => Ok(Operand::Mem(m)),
                Err(e) => {
                    // we have to do error handling here :(
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            },

            Token::Register(reg) => {
                if reg.is_ctrl_reg() {
                    Ok(Self::CtrReg(*reg))
                } else if reg.is_dbg_reg() {
                    Ok(Self::DbgReg(*reg))
                } else if reg.purpose() == RPurpose::Sgmnt {
                    Ok(Self::SegReg(*reg))
                } else {
                    Ok(Self::Reg(*reg))
                }
            }
            Token::Immediate(nm) => Ok(Self::Imm(*nm)),
            Token::SymbolRef(val) => Ok(Self::SymbolRef(val.to_string())),
            _ => Err(()),
        }
    }
}

impl Operand {
    pub fn size(&self) -> Size {
        match self {
            Self::Imm(n) => n.size(),
            Self::Reg(r) => r.size(),
            Self::CtrReg(r) => r.size(),
            Self::DbgReg(r) => r.size(),
            Self::Mem(m) => m.size(),
            Self::SymbolRef(_) => Size::Any,
            Self::Segment(s) => s.address.size(),
            Self::SegReg(_) => Size::Word,
        }
    }
}

impl ToAType for Operand {
    fn atype(&self) -> AType {
        match self {
            Self::Mem(m) => m.atype(),
            Self::CtrReg(r) | Self::SegReg(r) | Self::DbgReg(r) | Self::Reg(r) => r.atype(),
            Self::Imm(n) => n.atype(),
            Self::SymbolRef(_) => AType::Symbol,
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
            Some(Operand::Mem(m)) => match m.size() {
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

        match (dst, src) {
            (Size::Unknown, _) => src,
            (_, Size::Unknown) => dst,
            (_, _) => {
                if let Some(Operand::Imm(_)) = &self.src() {
                    if dst >= src {
                        dst
                    } else {
                        Size::Unknown
                    }
                } else {
                    if dst < src {
                        src
                    } else {
                        dst
                    }
                }
            }
        }
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
    pub fn get_sib_idx(&self) -> Option<usize> {
        if let Some(Operand::Mem(
            Mem::SIB(_, _, _, _)
            | Mem::SIBOffset(_, _, _, _, _)
            | Mem::Index(_, _, _)
            | Mem::IndexOffset(_, _, _, _),
        )) = self.dst()
        {
            return Some(0);
        }
        if let Some(Operand::Mem(
            Mem::SIB(_, _, _, _)
            | Mem::SIBOffset(_, _, _, _, _)
            | Mem::Index(_, _, _)
            | Mem::IndexOffset(_, _, _, _),
        )) = self.src()
        {
            return Some(1);
        }
        if let Some(Operand::Mem(
            Mem::SIB(_, _, _, _)
            | Mem::SIBOffset(_, _, _, _, _)
            | Mem::Index(_, _, _)
            | Mem::IndexOffset(_, _, _, _),
        )) = self.src2()
        {
            return Some(2);
        }
        None
    }
    #[inline]
    pub fn uses_sib(&self) -> bool {
        matches!(
            (self.dst(), self.src()),
            (
                Some(Operand::Mem(
                    Mem::SIB(_, _, _, _)
                        | Mem::SIBOffset(_, _, _, _, _)
                        | Mem::Index(_, _, _)
                        | Mem::IndexOffset(_, _, _, _),
                )),
                _,
            ) | (
                _,
                Some(Operand::Mem(
                    Mem::SIB(_, _, _, _)
                        | Mem::SIBOffset(_, _, _, _, _)
                        | Mem::Index(_, _, _)
                        | Mem::IndexOffset(_, _, _, _),
                )),
            ),
        )
    }
}

impl<'a> AST<'a> {
    pub fn fix_entry(&mut self) {
        if let Some(entry) = &self.entry {
            for index in 0..self.labels.len() {
                if self.labels[index].name == Cow::Borrowed(entry) {
                    if index == 0 {
                        return;
                    }
                    let (flabel, llabel) = self.labels.split_at_mut(index);
                    llabel[0].visibility = Visibility::Global;
                    std::mem::swap(&mut flabel[0], &mut llabel[0]);
                    return;
                }
            }
        }
    }
    pub fn make_globals(&mut self) {
        for g in &self.globals {
            let mut finished = false;
            for l in &mut self.labels {
                if l.name == Cow::Borrowed(g) {
                    finished = true;
                    l.visibility = Visibility::Global;
                    break;
                }
            }
            if !finished {
                for s in &mut self.vars {
                    if s.name == Cow::Borrowed(g) {
                        s.visibility = Visibility::Global;
                        break;
                    }
                }
            } else {
                continue;
            }
        }
    }
    pub fn filter_vars(vars: &'a Vec<Variable<'a>>) -> Vec<(u32, Vec<&'a Variable<'a>>)> {
        let mut ronly = Vec::new();
        let mut consts = Vec::new();
        let mut uninits = Vec::new();
        for v in vars {
            match v.vtype {
                VType::Readonly => ronly.push(v),
                VType::Uninit => uninits.push(v),
                VType::Const => consts.push(v),
            }
        }
        let mut toret = Vec::new();
        if !consts.is_empty() {
            toret.push((0x1, consts));
        }
        if !ronly.is_empty() {
            toret.push((0x2, ronly));
        }
        if !uninits.is_empty() {
            toret.push((0x3, uninits));
        }
        toret
    }
}
