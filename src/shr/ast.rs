// rasmx86_64 - ast.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use crate::pre::tok::Token;
use crate::shr::{
    reg::Register,
    mem::Mem,
    ins::Instruction,
    kwd::Keyword,
    num::Number
};
use crate::conf::{
    PREFIX_LAB,
    PREFIX_VAL,
    PREFIX_REG,
    PREFIX_KWD,
    PREFIX_REF,
    MEM_START,
    MEM_CLOSE,
};

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
    Imm(Number),
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
                if let Ok(mem) = Mem::new(&mm, Some(Keyword::Byte)){
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

impl ToString for ASTInstruction{
    fn to_string(&self) -> String{
        format!("{}{}{}",
            format!("{:?}", self.ins).to_lowercase(),
            if let Some(dst) = &self.dst{
                format!(" {}", dst.to_string())
            }
            else{
                "".to_string()
            },
            if let Some(src) = &self.src{
                format!(" {}", src.to_string())
            }
            else{
                "".to_string()
            },
        )
    }
}

impl Operand{
    pub fn size_bytes(&self) -> u8{
        match self {
            Self::Imm(n) => n.size_bytes(),
            Self::Reg(r) => r.size_bytes(),
            Self::Mem(m) => m.size_bytes(),
            Self::LabelRef(_) => 0,
            Self::ConstRef(_) => 0,
        }
    }
}

impl ToString for Operand{
    fn to_string(&self) -> String{
        match self {
            Self::Imm(n) => format!("{}{}", PREFIX_VAL, n.to_string()),
            Self::Reg(r) => format!("{}{}", PREFIX_REG, r.to_string()),
            Self::Mem(m) => {
                match m {
                    Mem::MemAddr(r, s) =>
                        format!("{}{}{}{} {}{}", MEM_START, PREFIX_REG, r.to_string(), MEM_CLOSE, PREFIX_KWD, 
                        match s{
                            1 => "byte",
                            2 => "word",
                            4 => "dword",
                            8 => "qword",
                            _ => "{unknown}"
                        }
                    ),
                    Mem::MemAddrWOffset(r, o, s) => 
                        format!("{}{}{}{}{} {}{}", o, MEM_START, PREFIX_REG, r.to_string(), MEM_CLOSE, PREFIX_KWD, 
                            match s{
                                1 => "byte",
                                2 => "word",
                                4 => "dword",
                                8 => "qword",
                                _ => "{unknown}"
                            }
                        ),
                    Mem::MemSIB(base, index, scale, displacement) => 
                        format!("{}{}{}{},{}{}{} {}{}", 
                            displacement, MEM_START, PREFIX_REG, base.to_string(), 
                            PREFIX_REG, index.to_string(), MEM_CLOSE, PREFIX_KWD,
                            match scale{
                                1 => "byte",
                                2 => "word",
                                4 => "dword",
                                8 => "qword",
                                _ => "{unknown}"
                            }
                    ),
                }
            },
            Self::ConstRef(r) => format!("{}{}", PREFIX_REF, r),
            Self::LabelRef(l) => format!("{}{}", PREFIX_LAB, l)
        }
    }
}
