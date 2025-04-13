// rasmx86_64 - atype.rs
// ---------------------
// made by matissoss
// licensed under MPL

use crate::shr::size::Size;

pub const I64 : AType = AType::Imm(Size::Qword);
pub const I32 : AType = AType::Imm(Size::Dword);
pub const I16 : AType = AType::Imm(Size::Word);
pub const I8  : AType = AType::Imm(Size::Byte);

pub const R64 : AType = AType::Reg(Size::Qword);
pub const R32 : AType = AType::Reg(Size::Dword);
pub const R16 : AType = AType::Reg(Size::Word);
pub const R8  : AType = AType::Reg(Size::Byte);

pub const M64 : AType = AType::Mem(Size::Qword);
pub const M32 : AType = AType::Mem(Size::Dword);
pub const M16 : AType = AType::Mem(Size::Word);
pub const M8  : AType = AType::Mem(Size::Byte);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AType{
    Imm(Size),
    Reg(Size),
    Mem(Size),
    Sym,
}

pub trait ToAType{
    fn atype(&self) -> AType;
}
