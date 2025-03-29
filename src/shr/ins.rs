// rasmx86_64 - ins.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use std::str::FromStr;
use crate::conf::FAST_MODE;

#[derive(Debug, Clone, Copy)]
pub enum Instruction{
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
fn ins_ie(i: &str, c: &str, ins: Instruction) -> Result<Instruction, ()>{
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

impl FromStr for Instruction{
    type Err = ();
    fn from_str(str_ins: &str) -> Result<Self, <Self as FromStr>::Err>{
        let raw_ins = str_ins.as_bytes();
        return match raw_ins.len() {
            1 => Err(()),
            2 => {
                match raw_ins[1] as char {
                    'e' => ins_ie(str_ins, "je", Instruction::JE),
                    'z' => ins_ie(str_ins, "jz", Instruction::JZ),
                    'l' => ins_ie(str_ins, "jl", Instruction::JL),
                    'g' => ins_ie(str_ins, "jg", Instruction::JG),
                    'r' => ins_ie(str_ins, "or", Instruction::OR),
                    _   => Err(())
                }
            },
            3 => {
                match raw_ins[1] as char {
                    'o' => {
                        match raw_ins[0] as char {
                            'm' => ins_ie(str_ins, "mov", Instruction::MOV),
                            'n' => ins_ie(str_ins, "not", Instruction::NOT),
                            'x' => ins_ie(str_ins, "xor", Instruction::XOR),
                            'p' => ins_ie(str_ins, "pop", Instruction::POP),
                            _   => Err(())
                        }
                    },
                    'i' => ins_ie(str_ins, "div", Instruction::DIV),
                    'd' => ins_ie(str_ins, "add", Instruction::ADD),
                    'u' => {
                        match raw_ins[0] as char{
                            's' => ins_ie(str_ins, "sub", Instruction::SUB),
                            'm' => ins_ie(str_ins, "mul", Instruction::MUL),
                            _   => Err(())
                        }
                    }
                    'e' => {
                        match raw_ins[0] as char {
                            'r' => ins_ie(str_ins, "ret", Instruction::RET),
                            'd' => ins_ie(str_ins, "dec", Instruction::DEC),
                            _   => Err(())
                        }
                    },
                    'g' => ins_ie(str_ins, "jge", Instruction::JGE),
                    'l' => ins_ie(str_ins, "jle", Instruction::JLE),
                    'n' => {
                        match raw_ins[2] as char{
                            'c' => ins_ie(str_ins, "inc", Instruction::INC),
                            'd' => ins_ie(str_ins, "and", Instruction::AND),
                            'z' => ins_ie(str_ins, "jnz", Instruction::JNZ),
                            'e' => ins_ie(str_ins, "jne", Instruction::JNE),
                            _   => Err(())
                        }
                    },
                    'h' => {
                        match raw_ins[2] as char{
                            'l' => ins_ie(str_ins, "shl", Instruction::SHL),
                            'r' => ins_ie(str_ins, "shr", Instruction::SHR),
                            _   => Err(())
                        }
                    }
                    'm' => {
                        match raw_ins[0] as char{
                            'j' => ins_ie(str_ins, "jmp", Instruction::JMP),
                            'c' => ins_ie(str_ins, "cmp", Instruction::CMP),
                            _   => Err(())
                        }
                    },
                    _   => Err(())
                }
            },
            4 => {
                match raw_ins[1] as char {
                    'd' => ins_ie(str_ins, "idiv", Instruction::IDIV),
                    'm' => ins_ie(str_ins, "imul", Instruction::IMUL),
                    'u' => ins_ie(str_ins, "push", Instruction::PUSH),
                    'a' => ins_ie(str_ins, "call", Instruction::CALL),
                    'e' => ins_ie(str_ins, "test", Instruction::TEST),
                    _   => Err(())
                }
            },
            7 => ins_ie(str_ins, "syscall", Instruction::SYSCALL),
            _ => Err(())
        }
    }
}
