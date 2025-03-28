// rasmx86_64 - ins.rs
// -------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    shr::{
        reg::Register,
        mem::Mem
    }
};

pub enum Operand{
    Imm8(i8),
    Imm16(i16),
    Imm32(i32),
    Imm64(i64),

    Reg (Register),

    Mem(Mem),
}

pub enum Instruction{
    MOV (Operand, Operand),
    ADD (Operand, Operand),
    SUB (Operand, Operand),
    MUL (Operand, Operand),
    DIV (Operand, Operand),
    AND (Operand, Operand),
    OR  (Operand, Operand),
    NOT (Operand, Operand),
    XOR (Operand, Operand),
    SHR (Operand, Operand),
    SHL (Operand, Operand),
    INC (Operand),
    DEC (Operand),

    POP (Operand),
    PUSH(Operand),

    CMP(Operand, Operand),
    TEST(Operand, Operand),

    JMP(String),
    CALL(String),
    JE (String),
    JZ (String),
    JNZ(String),
    JNE(String),
    JL (String),
    JLE(String),
    JG (String),
    JGE(String),

    SYSCALL,
    RET
}
