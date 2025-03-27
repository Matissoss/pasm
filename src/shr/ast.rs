// rasmx86_64 - ast.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::reg::Register;

pub struct AST;

#[derive(Debug)]
pub enum MSScale{
    One   = 0b00,
    Two   = 0b01,
    Four  = 0b10,
    Eight = 0b11,
}

impl TryFrom<i8> for MSScale{
    type Error = ();
    fn try_from(num: i8) -> Result<Self, <Self as TryFrom<i8>>::Error> {
        return match num {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            4 => Ok(Self::Four),
            8 => Ok(Self::Eight),
            _ => Err(())
        };
    }
}

#[derive(Debug)]
pub struct MemSIB{
    pub base: Register,
    pub index: Register,
    pub scale: MSScale,
    pub displacement: Option<i32>
}

#[derive(Debug)]
pub struct Label{
    name: String,
    ins: Vec<Instruction>
}
#[derive(Debug)]
pub struct Section{
    name: String,
    args: Vec<Vec<String>>
}

#[derive(Debug)]
pub struct Instruction{
    ins: String,
    src: Option<Value>,
    des: Option<Value>
}

#[derive(Debug)]
pub enum ASTElem{
    Label(Label),
    Section(Section),
    Value(Value),
    Instruction(Instruction),
}

#[derive(Debug)]
pub enum Value {
    Register(Register),
    MemSIB(MemSIB),
    
    Imm8 (i8 ),
    Imm16(i16),
    Imm32(i32),
    Imm64(i64),

    Mem8WOffset(Register, i8),
    Mem8(Register),
    
    Mem16WOffset(Register, i16),
    Mem16(Register),
    
    Mem32WOffset(Register, i32),
    Mem32(Register),

    Mem64WOffset(Register, i64),
    Mem64(Register)
}
