// rasmx86_64 - ast.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use crate::pre::tok::Token;
use crate::shr::{
    reg::Register,
    mem::Mem,
    ins::Instruction,
    kwd::Keyword
};

use std::convert::TryInto;

#[derive(Debug, Clone, PartialEq)]
pub enum AsmType{
    Imm,
    Reg,
    Mem,
    ConstRef,
    LabelRef,
}

pub struct AsmTypes(pub Vec<AsmType>);

pub trait ToAsmType{
    fn asm_type(&self) -> AsmType;
}

#[derive(Debug, Clone)]
pub enum Operand{
    Reg(Register),
    Imm(i64),
    Mem(Mem),
    LabelRef(String),
    ConstRef(String),
}

#[derive(Debug, Clone)]
pub struct ASTInstruction{
    pub ins : Instruction,
    pub src : Option<Operand>,
    pub dst : Option<Operand>,
    pub lin : usize,
}

#[derive(Debug, Clone)]
pub struct VarDec{
    pub name: String,
    pub size: u8,
    pub content: String
}

#[derive(Debug, Clone)]
pub enum ASTNode{
    Ins(ASTInstruction),
    Label(String),
    Global(String),
    Section(String),
    VarDec(VarDec),
    End
}

#[derive(Debug, Clone)]
pub struct Label{
    pub name : String,
    pub inst : Vec<ASTInstruction>
}

#[derive(Debug, Clone)]
pub struct Section{
    pub name : String,
    pub vars : Option<Vec<VarDec>>
}

#[derive(Debug, Clone)]
pub struct AST{
    pub sections: Vec<Section>,
    pub text: Vec<String>,
    pub labels: Vec<Label>
}

impl TryFrom<Token> for Operand{
    type Error = ();
    fn try_from(tok: Token) -> Result<Self, <Self as TryFrom<Token>>::Error>{
        match tok {
            Token::Register(reg) => Ok(Self::Reg(reg)),
            Token::Immediate(nm) => Ok(Self::Imm(nm )),
            Token::MemAddr(mm)   => {
                if let Some(mem) = Mem::create(&mm, Some(Keyword::Byte)){
                    return Ok(Self::Mem(mem));
                }
                else{
                    return Err(())
                }
            }
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
            Self::Mem   => String::from("memX"),
            Self::Reg   => String::from("regX"),
            Self::LabelRef => String::from("labelref"),
            Self::ConstRef => String::from("constref")
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

impl ToAsmType for Operand{
    fn asm_type(&self) -> AsmType{
        match self {
            Self::Reg(_)        => AsmType::Reg,
            Self::Mem(_)        => AsmType::Mem,
            Self::LabelRef(_)   => AsmType::LabelRef,
            Self::ConstRef(_)   => AsmType::ConstRef,
            Self::Imm(_)        => AsmType::Imm,
        }
    }
}

// if returns 0 then error
impl Operand{
    pub fn size_bytes(&self) -> u8{
        match self{
            Self::Reg(rg) => rg.size_bytes(),
            Self::Imm(im) => {
                if let Ok(_) = <i64 as TryInto<i8>>::try_into(*im){
                    return 1;
                }
                if let Ok(_) = <i64 as TryInto<i16>>::try_into(*im){
                    return 2;
                }
                if let Ok(_) = <i64 as TryInto<i32>>::try_into(*im){
                    return 4;
                }
                return 8;
            },
            Self::Mem(mem) => {
                match mem {
                    Mem::MemAddr(_, size)           => *size,
                    Mem::MemSIB(_,_,_,size)         => *size,
                    Mem::MemAddrWOffset(_,_,size)   => *size,
                }
            },
            Self::ConstRef(_) => 0,
            Self::LabelRef(_) => 0,
        }
    }
    pub fn needs_rex(&self) -> bool{
        if let Self::Reg(r) = self{
            return r.needs_rex();
        }
        return self.size_bytes() == 8;
    }
}
