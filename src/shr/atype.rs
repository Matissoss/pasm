// rasmx86_64 - src/shr/atype.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{reg::Purpose as RegisterPurpose, size::Size};

pub const CR: AType = AType::Register(RegisterPurpose::Ctrl, Size::Dword);
pub const DR: AType = AType::Register(RegisterPurpose::Dbg, Size::Dword);
pub const SR: AType = AType::Register(RegisterPurpose::Sgmnt, Size::Word);

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

pub const MA: AType = AType::Memory(Size::Any);
pub const M64: AType = AType::Memory(Size::Qword);
pub const M32: AType = AType::Memory(Size::Dword);
pub const M16: AType = AType::Memory(Size::Word);
pub const M8: AType = AType::Memory(Size::Byte);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AType {
    Immediate(Size),                 // immediate
    Register(RegisterPurpose, Size), // register
    Memory(Size),                    // memory
    SMemory(Size),                   // segment memory
    Symbol,
}

pub trait ToAType {
    fn atype(&self) -> AType;
}
