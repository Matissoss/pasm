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
    symbol::Visibility,
    atype::{
        AType,
        ToAType
    },
    var::{
        Variable,
        VType,
    }
};

#[derive(Debug, Clone, PartialEq)]
pub enum Operand{
    Reg(Register),
    Imm(Number),
    Mem(Mem),
    SymbolRef(String),
}

#[derive(Debug, Clone)]
pub struct Instruction{
    pub mnem : Mnm,
    pub addt : Option<Vec<Mnm>>,
    pub oprs : Vec<Operand>,
    pub line : usize,
}

#[derive(Debug, Clone)]
pub enum ASTNode{
    Ins(Instruction),
    Label(String),
    Global(String),
    Section(String),
    Variable(Variable),
    Entry(String),
}

#[derive(Debug, Clone, Default)]
pub struct Label{
    pub name : String,
    pub inst : Vec<Instruction>,
    pub visibility: Visibility
}

#[derive(Debug, Clone, Default)]
pub struct AST{
    pub global: Vec<String>,
    pub labels: Vec<Label> ,
    pub vars  : Vec<Variable>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolDeclaration{
    name: String,
    size: Option<u32>,
    cont: Option<SymbolValue>,
    stype: Section,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SymbolValue{
    Number(Number),
    String(String)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Section{
    Data,
    Bss ,
    Readonly,
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
            Token::SymbolRef(val) => Ok(Self::SymbolRef(val)),
            _                    => Err(())
        }
    }
}

impl Operand{
    pub fn size(&self) -> Size{
        match self {
            Self::Imm(n) => n.size(),
            Self::Reg(r) => r.size(),
            Self::Mem(m) => m.size(),
            Self::SymbolRef(_) => Size::Unknown,
        }
    }
}

impl ToAType for Operand{
    fn atype(&self) -> AType{
        match self {
            Self::Mem(m) => m.atype(),
            Self::Reg(r) => r.atype(),
            Self::Imm(n) => n.atype(),
            Self::SymbolRef(_) => AType::Sym,
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
                if let Some(Operand::Imm(_)) = &self.src(){
                    if dst >= src{
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
    #[inline]
    pub fn dst(&self) -> Option<&Operand> {
        return self.oprs.get(0)
    }
    #[inline]
    pub fn src(&self) -> Option<&Operand> {
        return self.oprs.get(1)
    }
    #[inline]
    // operand existence
    pub fn op_ex(&self) -> (bool, bool){
        match (self.dst(), self.src()){
            (Some(_), None)     => (true, false),
            (Some(_), Some(_))  => (true, true),
            (None, Some(_))     => (false, true),
            (None, None)        => (false, false),
        }
    }
    #[inline]
    pub fn uses_sib(&self) -> bool {
        match (self.dst(), self.src()){
            (Some(Operand::Mem(
                Mem::SIB(_,_,_,_)|Mem::SIBOffset(_,_,_,_,_)|
                Mem::Index(_,_,_)|Mem::IndexOffset(_,_,_,_)
            )), _) => true,
            (_, Some(Operand::Mem(
                Mem::SIB(_,_,_,_)|Mem::SIBOffset(_,_,_,_,_)|
                Mem::Index(_,_,_)|Mem::IndexOffset(_,_,_,_)
            ))) => true,
            _ => false,
        }
    }
}

impl AST{
    pub fn make_globals(&mut self){
        for g in &self.global{
            let mut finished = false;
            for l in &mut self.labels{
                if &l.name == g{
                    finished = true;
                    l.visibility = Visibility::Global;
                    break;
                }
            }
            if !finished{
                for s in &mut self.vars{
                    if &s.name == g{
                        s.visibility = Visibility::Global;
                        break;
                    }
                }
            }
            else {continue}
        }
    }
    pub fn filter_vars(self) -> Vec<(u32, Vec<Variable>)>{
        let mut ronly = Vec::new();
        let mut consts = Vec::new();
        let mut uninits = Vec::new();
        for v in self.vars{
            match v.vtype{
                VType::Readonly => ronly.push(v),
                VType::Uninit   => uninits.push(v),
                VType::Const    => consts.push(v),
            }
        }
        let mut toret = Vec::new();
        if !consts.is_empty(){
            toret.push((0x1, consts));
        }
        if !ronly.is_empty(){
            toret.push((0x2, ronly));
        }
        if !uninits.is_empty(){
            toret.push((0x3, uninits));
        }
        return toret;
    }
}
