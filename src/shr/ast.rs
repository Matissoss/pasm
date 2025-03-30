// rasmx86_64 - ast.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use crate::pre::tok::Token;
use crate::shr::{
    reg::Register,
    mem::Mem,
    ins::Instruction
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum AsmType{
    Imm,

    Imm8,
    Imm16,
    Imm32,
    Imm64,

    Reg,
    
    Reg8,
    Reg16,
    Reg32,
    Reg64,

    Mem,
    Mem8,
    Mem16,
    Mem32,
    Mem64,

    // register/memory
    RM,
    RM8,
    RM16,
    RM32,
    RM64,

    ConstString
}

pub struct AsmTypes(pub Vec<AsmType>);

#[allow(unused)]
pub trait ToAsmType{
    fn asm_type(&self) -> AsmType;
}

#[allow(unused)]
#[derive(Debug)]
pub enum Operand{
    Reg(Register),
    Imm(i64),
    Mem(Mem),
    LabelRef(String),
    ConstRef(String),
}

#[allow(unused)]
#[derive(Debug)]
pub struct AstInstruction{
    pub ins: Instruction,
    pub src: Option<Operand>,
    pub dst: Option<Operand>
}

#[allow(unused)]
#[derive(Debug)]
pub struct VarDec{
    pub name: String,
    pub bss: bool,
    pub size: u8,
    pub content: String
}

#[allow(unused)]
#[derive(Debug)]
pub enum ASTNode{
    Ins(AstInstruction),
    Label(String),
    Global(String),
    Section(String),
    VarDec(VarDec)
}

#[allow(unused)]
#[derive(Debug)]
pub enum ExtASTNode{
    Section(String, VarDec),
    Label(String, AstInstruction),
}

#[allow(unused)]
#[derive(Debug)]
pub struct Label{
    pub name : String,
    pub inst : Vec<AstInstruction>
}

#[allow(unused)]
#[derive(Debug)]
pub struct Section{
    pub name : String,
    pub vars : Option<Vec<VarDec>>
}

#[derive(Debug)]
pub struct AST{
    pub sections: Vec<Section>,
    pub global: Vec<String>,
    pub labels: Vec<Label>
}

impl TryFrom<Token> for Operand{
    type Error = ();
    fn try_from(tok: Token) -> Result<Self, <Self as TryFrom<Token>>::Error>{
        match tok {
            Token::Register(reg) => Ok(Self::Reg(reg)),
            Token::Immediate(nm) => Ok(Self::Imm(nm )),
            //Token::MemAddr(mm)   => Ok(Self::Mem(mm )),
            Token::ConstRef(val) => Ok(Self::ConstRef(val)),
            Token::LabelRef(val) => Ok(Self::LabelRef(val)),
            _                    => Err(())
        }
    }
}

impl ToString for AsmType{
    fn to_string(&self) -> String{
        match self {
            Self::Imm   => String::from("immX"),
            Self::Imm8  => String::from("imm8"),
            Self::Imm16 => String::from("imm16"),
            Self::Imm32 => String::from("imm32"),
            Self::Imm64 => String::from("imm64"),

            Self::Mem   => String::from("memX"),
            Self::Mem8  => String::from("mem8"),
            Self::Mem16 => String::from("mem16"),
            Self::Mem32 => String::from("mem32"),
            Self::Mem64 => String::from("mem64"),

            Self::Reg   => String::from("regX"),
            Self::Reg8  => String::from("reg8"),
            Self::Reg16 => String::from("reg16"),
            Self::Reg32 => String::from("reg32"),
            Self::Reg64 => String::from("reg64"),

            Self::RM    => String::from("regX/memX"),
            Self::RM8   => String::from("reg8/mem8"),
            Self::RM16  => String::from("reg16/mem16"),
            Self::RM32  => String::from("reg32/mem32"),
            Self::RM64  => String::from("reg64/mem64"),

            Self::ConstString => String::from("(comptime string)")
        }
    }
}

impl ToString for AsmTypes{
    fn to_string(&self) -> String{
        let mut ret = String::new();
        ret.push_str("[");
        for (i, t) in self.0.iter().enumerate(){
            ret.push_str(&t.to_string());
            if i+1 < self.0.len(){
                ret.push_str(", ");
            }
        }
        ret.push_str("]");
        return ret;
    }
}
