// rasmx86_64 - ast.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use crate::pre::tok::Token;
use crate::shr::{
    reg::Register,
    mem::Mem,
    ins::Mnemonic as Mnm,
    kwd::Keyword,
    num::Number,
    size::Size,
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
    ImmFloat,
    ImmDouble,
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
pub struct Instruction{
    pub mnem : Mnm,
    pub addt : Option<Vec<Mnm>>,
    pub oprs : Vec<Operand>,
    pub line : usize,
}

#[derive(Debug, Clone, Default)]
pub struct VarDec{
    pub name: String,
    pub size: usize,
    pub bss : bool,
    pub content: Option<String>
}

#[derive(Debug, Clone)]
pub enum ASTNode{
    Ins(Instruction),
    Label(String),
    Global(String),
    Section(String),
    VarDec(VarDec),
    Entry(String),
}

#[derive(Debug, Clone, Default)]
pub struct Label{
    pub name : String,
    pub inst : Vec<Instruction>
}

#[derive(Debug, Clone, Default)]
pub struct AST{
    pub global: Vec<String>,
    pub labels: Vec<Label> ,
    pub variab: Vec<VarDec>,
    pub entry : String,
    pub bits  : u8,
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
            Self::ImmFloat => String::from("float"),
            Self::ImmDouble => String::from("double"),
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
            Self::Imm(n)        => match n {
                Number::Float(_) => AsmType::ImmFloat,
                Number::Double(_) => AsmType::ImmDouble,
                _ => AsmType::Imm,
            }
        }
    }
}

impl ToString for Instruction{
    fn to_string(&self) -> String{
        let mut mnems : String = self.mnem.to_string();
        if let Some(addt) = &self.addt{
            for mnm in addt {
                mnems.push_str(&format!(" {}", mnm.to_string()))
            }
        }
        let mut oprs = String::new();
        for operand in &self.oprs{
            oprs.push_str(&format!(" {}", operand.to_string()))
        }
        return format!("{} {}", mnems, oprs)
    }
}

impl Operand{
    pub fn size(&self) -> Size{
        match self {
            Self::Imm(n) => n.size(),
            Self::Reg(r) => r.size(),
            Self::Mem(m) => m.size(),
            Self::LabelRef(_) => Size::Unknown,
            Self::ConstRef(_) => Size::Unknown,
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
                        format!("{}{}{}{} {}{}", MEM_START, PREFIX_REG, r.to_string(), MEM_CLOSE, PREFIX_KWD, s
                    ),
                    Mem::MemAddrWOffset(r, o, s) => 
                        format!("{}{}{}{}{} {}{}", o, MEM_START, PREFIX_REG, r.to_string(), MEM_CLOSE, PREFIX_KWD, s
                        ),
                    Mem::MemSIB(base, index, scale, displacement) => 
                        format!("{}{}{}{},{}{}{} {}{}", 
                            displacement, MEM_START, PREFIX_REG, base.to_string(), 
                            PREFIX_REG, index.to_string(), MEM_CLOSE, PREFIX_KWD, scale
                    ),
                }
            },
            Self::ConstRef(r) => format!("{}{}", PREFIX_REF, r),
            Self::LabelRef(l) => format!("{}{}", PREFIX_LAB, l)
        }
    }
}

impl Instruction{
    pub fn size(&self) -> Size{
        let dst = match &self.dst(){
            Some(o) => o.size(),
            None    => Size::Unknown,
        };
        let src = match &self.src(){
            Some(o) => o.size(),
            None => Size::Unknown,
        };

        return match (dst, src) {
            (Size::Unknown, _) => src,
            (_, Size::Unknown) => dst,
            (_, _) => {
                if let Some(Operand::Imm(n)) = &self.src(){
                    if dst < n.size(){
                        return dst;
                    }
                    else {
                        return Size::Unknown;
                    }
                }
                if dst != src {
                    return Size::Unknown;
                }
                else {
                    return dst;
                }
            },
        }
    }
    pub fn dst(&self) -> Option<&Operand> {
        return self.oprs.get(0)
    }
    pub fn src(&self) -> Option<&Operand> {
        return self.oprs.get(1)
    }
}
