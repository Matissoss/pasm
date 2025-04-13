// rasmx86_64 - ins.rs
// -------------------
// made by matissoss
// licensed under MPL

use std::str::FromStr;
use crate::shr::size::Size;
use crate::conf::FAST_MODE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mnemonic{
    MOV,
    ADD,
    SUB,
    IMUL,
    MUL,
    DIV,
    IDIV,

    AND,
    OR,
    NOT,
    XOR,
    SHR,
    SHL,
    LEA,

    INC,
    DEC,

    POP,
    PUSH,

    CMP,
    TEST,

    JMP,
    CALL,
    JE,
    JZ,
    JNZ,
    JNE,
    JL,
    JLE,
    JG,
    JGE,

    SYSCALL,
    RET
}

#[inline(always)]
fn ins_ie(i: &str, c: &str, ins: Mnemonic) -> Result<Mnemonic, ()>{
    if FAST_MODE {
        return Ok(ins);
    }
    else {
        if i == c {
            return Ok(ins);
        }
        else {
            return Err(());
        }
    }
}

impl FromStr for Mnemonic{
    type Err = ();
    fn from_str(str_ins: &str) -> Result<Self, <Self as FromStr>::Err>{
        let raw_ins = str_ins.as_bytes();
        return match raw_ins.len() {
            1 => Err(()),
            2 => {
                match raw_ins[1] as char {
                    'e' => ins_ie(str_ins, "je", Self::JE),
                    'z' => ins_ie(str_ins, "jz", Self::JZ),
                    'l' => ins_ie(str_ins, "jl", Self::JL),
                    'g' => ins_ie(str_ins, "jg", Self::JG),
                    'r' => ins_ie(str_ins, "or", Self::OR),
                    _   => Err(())
                }
            },
            3 => {
                match raw_ins[1] as char {
                    'o' => {
                        match raw_ins[0] as char {
                            'm' => ins_ie(str_ins, "mov", Self::MOV),
                            'n' => ins_ie(str_ins, "not", Self::NOT),
                            'x' => ins_ie(str_ins, "xor", Self::XOR),
                            'p' => ins_ie(str_ins, "pop", Self::POP),
                            _   => Err(())
                        }
                    },
                    'i' => ins_ie(str_ins, "div", Self::DIV),
                    'd' => ins_ie(str_ins, "add", Self::ADD),
                    'u' => {
                        match raw_ins[0] as char{
                            's' => ins_ie(str_ins, "sub", Self::SUB),
                            'm' => ins_ie(str_ins, "mul", Self::MUL),
                            _   => Err(())
                        }
                    }
                    'e' => {
                        match raw_ins[0] as char {
                            'r' => ins_ie(str_ins, "ret", Self::RET),
                            'd' => ins_ie(str_ins, "dec", Self::DEC),
                            'l' => ins_ie(str_ins, "lea", Self::LEA),
                            _   => Err(())
                        }
                    },
                    'g' => ins_ie(str_ins, "jge", Self::JGE),
                    'l' => ins_ie(str_ins, "jle", Self::JLE),
                    'n' => {
                        match raw_ins[2] as char{
                            'c' => ins_ie(str_ins, "inc", Self::INC),
                            'd' => ins_ie(str_ins, "and", Self::AND),
                            'z' => ins_ie(str_ins, "jnz", Self::JNZ),
                            'e' => ins_ie(str_ins, "jne", Self::JNE),
                            _   => Err(())
                        }
                    },
                    'h' => {
                        match raw_ins[2] as char{
                            'l' => ins_ie(str_ins, "shl", Self::SHL),
                            'r' => ins_ie(str_ins, "shr", Self::SHR),
                            _   => Err(())
                        }
                    }
                    'm' => {
                        match raw_ins[0] as char{
                            'j' => ins_ie(str_ins, "jmp", Self::JMP),
                            'c' => ins_ie(str_ins, "cmp", Self::CMP),
                            _   => Err(())
                        }
                    },
                    _   => Err(())
                }
            },
            4 => {
                match raw_ins[1] as char {
                    'd' => ins_ie(str_ins, "idiv", Self::IDIV),
                    'm' => ins_ie(str_ins, "imul", Self::IMUL),
                    'u' => ins_ie(str_ins, "push", Self::PUSH),
                    'a' => ins_ie(str_ins, "call", Self::CALL),
                    'e' => ins_ie(str_ins, "test", Self::TEST),
                    _   => Err(())
                }
            },
            7 => ins_ie(str_ins, "syscall", Self::SYSCALL),
            _ => Err(())
        }
    }
}

impl ToString for Mnemonic{
    fn to_string(&self) -> String{
        format!("{:?}", self).to_lowercase()
    }
}

impl Mnemonic{
    pub fn allows_diff_size(&self, _left: Option<Size>, _right: Option<Size>) -> bool{
        return false;
    }
    pub fn allows_mem_mem(&self) -> bool{
        return false;
    }
    pub fn is_jump(&self) -> bool{
        format!("{:?}", self).starts_with('J')
    }
}
