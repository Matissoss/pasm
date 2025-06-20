// rasmx86_64 - src/shr/atype.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    reg::{Purpose as RegisterPurpose, Register},
    size::Size,
};

pub const CR: AType = AType::Register(RegisterPurpose::Ctrl, Size::Any);
pub const DR: AType = AType::Register(RegisterPurpose::Dbg, Size::Any);
pub const SR: AType = AType::Register(RegisterPurpose::Sgmnt, Size::Any);

pub const ASTR: AType = AType::Immediate(Size::Any);
pub const IA: AType = AType::Immediate(Size::Any);
pub const I64: AType = AType::Immediate(Size::Qword);
pub const I32: AType = AType::Immediate(Size::Dword);
pub const I16: AType = AType::Immediate(Size::Word);
pub const I8: AType = AType::Immediate(Size::Byte);

pub const RA: AType = AType::Register(RegisterPurpose::General, Size::Any);
pub const R64: AType = AType::Register(RegisterPurpose::General, Size::Qword);
pub const R32: AType = AType::Register(RegisterPurpose::General, Size::Dword);
pub const R16: AType = AType::Register(RegisterPurpose::General, Size::Word);
pub const R8: AType = AType::Register(RegisterPurpose::General, Size::Byte);

pub const MMX: AType = AType::Register(RegisterPurpose::Mmx, Size::Any);
pub const XMM: AType = AType::Register(RegisterPurpose::F128, Size::Any);
pub const YMM: AType = AType::Register(RegisterPurpose::F256, Size::Any);

pub const MA: AType = AType::Memory(Size::Any);
pub const M256: AType = AType::Memory(Size::Yword);
pub const M128: AType = AType::Memory(Size::Xword);
pub const M64: AType = AType::Memory(Size::Qword);
pub const M32: AType = AType::Memory(Size::Dword);
pub const M16: AType = AType::Memory(Size::Word);
pub const M8: AType = AType::Memory(Size::Byte);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AType {
    Immediate(Size),                 // immediate
    Register(RegisterPurpose, Size), // register
    Memory(Size),                    // memory
    ExtendedRegister(Register),
}

pub trait ToAType {
    fn atype(&self) -> AType;
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for AType {
    fn to_string(&self) -> String {
        match self {
            Self::Register(rp, sz) => {
                format!("{} {} reg", rp.to_string(), sz)
            }
            Self::ExtendedRegister(rg) => {
                format!("{} reg", rg.to_string())
            }
            Self::Memory(sz) => {
                format!("{} mem", sz)
            }
            Self::Immediate(sz) => {
                format!("{} imm", sz)
            }
        }
    }
}

pub fn atype_arr_string(arr: &[AType]) -> String {
    let mut string = String::new();
    string.push('[');
    for (i, e) in arr.iter().enumerate() {
        string.push_str(&e.to_string());
        if i + 1 < arr.len() {
            string.push_str(", ");
        }
    }
    string.push(']');
    string
}
