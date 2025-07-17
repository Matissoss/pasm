// pasm - src/shr/atype.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::Operand,
    booltable::BoolTable8,
    reg::{Purpose, Register},
    size::Size,
};

pub const BCST_FLAG: u8 = 0x0;
pub const VSIB_FLAG: u8 = 0x1;

pub const ANY: AType = AType::Any;
pub const CS: AType = AType::Register(Register::CS, true);
pub const DS: AType = AType::Register(Register::DS, true);
pub const ES: AType = AType::Register(Register::ES, true);
pub const SS: AType = AType::Register(Register::SS, true);
pub const FS: AType = AType::Register(Register::FS, true);
pub const GS: AType = AType::Register(Register::GS, true);

pub const SR: AType = AType::Register(Register::CS, false);
pub const CR: AType = AType::Register(Register::CR0, false);
pub const DR: AType = AType::Register(Register::DR0, false);

pub const CL: AType = AType::Register(Register::CL, true);
pub const AL: AType = AType::Register(Register::AL, true);
pub const AX: AType = AType::Register(Register::AX, true);
pub const EAX: AType = AType::Register(Register::EAX, true);
pub const EDX: AType = AType::Register(Register::EDX, true);
pub const DX: AType = AType::Register(Register::DX, true);

pub const RA: AType = AType::Register(Register::__ANY, false);
pub const R8: AType = AType::Register(Register::AL, false);
pub const R16: AType = AType::Register(Register::AX, false);
pub const R32: AType = AType::Register(Register::EAX, false);
pub const R64: AType = AType::Register(Register::RAX, false);
pub const MMX: AType = AType::Register(Register::MM0, false);
pub const XMM: AType = AType::Register(Register::XMM0, false);
pub const YMM: AType = AType::Register(Register::YMM0, false);
pub const ZMM: AType = AType::Register(Register::ZMM0, false);
pub const K: AType = AType::Register(Register::K0, false);

pub const MA: AType = AType::Memory(Size::Any, Size::Any, BoolTable8::new());
pub const M8: AType = AType::Memory(Size::Byte, Size::Any, BoolTable8::new());
pub const M16: AType = AType::Memory(Size::Word, Size::Any, BoolTable8::new());
pub const MBCST16: AType = AType::Memory(
    Size::Word,
    Size::Any,
    BoolTable8::new().setc(BCST_FLAG, true),
);
pub const M32: AType = AType::Memory(Size::Dword, Size::Any, BoolTable8::new());
pub const MBCST32: AType = AType::Memory(
    Size::Dword,
    Size::Any,
    BoolTable8::new().setc(BCST_FLAG, true),
);
pub const M64: AType = AType::Memory(Size::Qword, Size::Any, BoolTable8::new());
pub const MBCST64: AType = AType::Memory(
    Size::Qword,
    Size::Any,
    BoolTable8::new().setc(BCST_FLAG, true),
);
pub const M128: AType = AType::Memory(Size::Xword, Size::Any, BoolTable8::new());
pub const M256: AType = AType::Memory(Size::Yword, Size::Any, BoolTable8::new());
pub const M512: AType = AType::Memory(Size::Zword, Size::Any, BoolTable8::new());

pub const VM64Z: AType = AType::Memory(
    Size::Qword,
    Size::Zword,
    BoolTable8::new().setc(VSIB_FLAG, true),
);
pub const VM64Y: AType = AType::Memory(
    Size::Qword,
    Size::Yword,
    BoolTable8::new().setc(VSIB_FLAG, true),
);
pub const VM64X: AType = AType::Memory(
    Size::Qword,
    Size::Xword,
    BoolTable8::new().setc(VSIB_FLAG, true),
);

pub const VM32Z: AType = AType::Memory(
    Size::Dword,
    Size::Zword,
    BoolTable8::new().setc(VSIB_FLAG, true),
);
pub const VM32Y: AType = AType::Memory(
    Size::Dword,
    Size::Yword,
    BoolTable8::new().setc(VSIB_FLAG, true),
);
pub const VM32X: AType = AType::Memory(
    Size::Dword,
    Size::Xword,
    BoolTable8::new().setc(VSIB_FLAG, true),
);

pub const IA: AType = AType::Immediate(Size::Any, false);
pub const I8: AType = AType::Immediate(Size::Byte, false);
pub const I16: AType = AType::Immediate(Size::Word, false);
pub const I32: AType = AType::Immediate(Size::Dword, false);
pub const I64: AType = AType::Immediate(Size::Qword, false);
pub const STRING: AType = AType::Immediate(Size::Any, true);

#[derive(Debug, Clone, Copy)]
pub enum AType {
    NoType,
    Any,

    //                fixed register
    Register(Register, bool),
    //     size|address size  (registers used) | flags (vsib and bcst)
    Memory(Size, Size, BoolTable8),
    //              is_string
    Immediate(Size, bool),
}

impl std::fmt::Display for AType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Register(r, true) => write!(f, "{}", r.to_string())?,
            Self::Register(r, false) => match (r.size(), r.purpose()) {
                (Size::Byte, _) => write!(f, "r8")?,
                (Size::Word, Purpose::General) => write!(f, "r16")?,
                (Size::Word, Purpose::Sgmnt) => write!(f, "sreg")?,
                (Size::Dword, Purpose::General) => write!(f, "r32")?,
                (Size::Dword, Purpose::Dbg) => write!(f, "dr0")?,
                (Size::Dword, Purpose::Ctrl) => write!(f, "cr0")?,
                (Size::Qword, Purpose::General) => write!(f, "r64")?,
                (Size::Qword, Purpose::Mmx) => write!(f, "mm0")?,
                (Size::Xword, _) => write!(f, "xmm0")?,
                (Size::Yword, _) => write!(f, "ymm0")?,
                (Size::Qword, Purpose::Mask) => write!(f, "k0")?,
                (Size::Zword, _) => write!(f, "zmm0")?,
                _ => write!(f, "")?,
            },
            Self::Immediate(s, false) => match s {
                Size::Byte => write!(f, "imm8")?,
                Size::Word => write!(f, "imm16")?,
                Size::Dword => write!(f, "imm32")?,
                Size::Qword => write!(f, "imm64")?,
                _ => write!(f, "immANY")?,
            },
            Self::Immediate(_, true) => write!(f, "string")?,
            Self::Memory(sz, addrsz, flags) => {
                let vsib = flags.get(VSIB_FLAG).unwrap();
                let bcst = flags.get(BCST_FLAG).unwrap();
                if vsib {
                    match addrsz {
                        Size::Xword => {
                            write!(f, "vm{}x", (<Size as Into<u8>>::into(*sz) as u16) << 3)?
                        }
                        Size::Yword => {
                            write!(f, "vm{}y", (<Size as Into<u8>>::into(*sz) as u16) << 3)?
                        }
                        Size::Zword => {
                            write!(f, "vm{}z", (<Size as Into<u8>>::into(*sz) as u16) << 3)?
                        }
                        _ => write!(f, "")?,
                    }
                } else {
                    match addrsz {
                        Size::Any => write!(
                            f,
                            "m{}{}",
                            if bcst { "bcst" } else { "" },
                            (<Size as Into<u8>>::into(*sz) as u16) << 3
                        )?,
                        Size::Word => write!(
                            f,
                            "m{}{}&16",
                            if bcst { "bcst" } else { "" },
                            (<Size as Into<u8>>::into(*sz) as u16) << 3
                        )?,
                        Size::Dword => write!(
                            f,
                            "m{}{}&32",
                            if bcst { "bcst" } else { "" },
                            (<Size as Into<u8>>::into(*sz) as u16) << 3
                        )?,
                        Size::Qword => write!(
                            f,
                            "m{}{}&64",
                            if bcst { "bcst" } else { "" },
                            (<Size as Into<u8>>::into(*sz) as u16) << 3
                        )?,
                        _ => write!(f, "")?,
                    }
                }
            }
            Self::Any | Self::NoType => write!(f, "")?,
        };
        Ok(())
    }
}

pub trait ToType {
    fn atype(&self) -> AType;
}

impl ToType for Operand<'_> {
    fn atype(&self) -> AType {
        match self {
            Self::Register(r) => AType::Register(*r, false),
            Self::Mem(m) => AType::Memory(
                m.size(),
                m.addrsize(),
                BoolTable8::new()
                    .setc(BCST_FLAG, m.is_bcst())
                    .setc(VSIB_FLAG, m.is_vsib()),
            ),
            Self::Symbol(s) => {
                if s.is_deref() {
                    AType::Memory(
                        s.size().unwrap_or(Size::Unknown),
                        Size::Any,
                        BoolTable8::new(),
                    )
                } else {
                    AType::Immediate(Size::Dword, false)
                }
            }
            Self::Imm(i) => AType::Immediate(i.size(), false),
            Self::String(s) => match s.len() {
                1 => AType::Immediate(Size::Byte, false),
                2 => AType::Immediate(Size::Word, false),
                3..=4 => AType::Immediate(Size::Dword, false),
                5..=8 => AType::Immediate(Size::Qword, false),
                _ => AType::Immediate(Size::Unknown, true),
            },
        }
    }
}

impl PartialEq for AType {
    fn eq(&self, rhs: &Self) -> bool {
        match (*self, *rhs) {
            (AType::Register(lr, lf), AType::Register(rr, rf)) => {
                if lf || rf {
                    lr == rr
                } else if lr.purpose() == rr.purpose() && lr.size() == rr.size() {
                    true
                } else if lr.is_any() || rr.is_any() {
                    lr.size() == rr.size()
                } else {
                    false
                }
            }
            (AType::Memory(lsz, laddr, lbcst), AType::Memory(rsz, raddr, rbcst)) => {
                (lbcst == rbcst || raddr.is_any() || laddr.is_any() || laddr == raddr) && lsz == rsz
            }
            (AType::Immediate(lsz, ls), AType::Immediate(rsz, rs)) => {
                if ls && rs {
                    true
                } else if ls || rs {
                    false
                } else {
                    rsz <= lsz
                }
            }
            (AType::Any, _) | (_, AType::Any) => true,
            _ => false,
        }
    }
}
