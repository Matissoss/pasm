// rasmx86_64 - opcode.rs
// ----------------------
// made by matissoss
// licensed under MPL

use crate::shr::ast::ASTInstruction;
use crate::shr::ins::Instruction as Ins;

enum Status{
    Complete,
    NotCompleted
}

enum Returned{
    Final(Vec<u8>),
    NeedsContext(ASTInstruction)
}

fn opcode(ins: &ASTInstruction) -> (Returned, Status){
    match ins.ins{
        Ins::SYSCALL => (Returned::Final(vec![0x0F, 0x05]), Status::Complete),
        Ins::RET     => (Returned::Final(vec![0xC3]), Status::NotCompleted)  ,
        Ins::JMP|Ins::JE|Ins::JNE|Ins::JZ|Ins::JNZ|Ins::JL|Ins::JLE|Ins::CALL|Ins::JG|Ins::JGE => {
            (Returned::NeedsContext(ins.clone()), Status::NotCompleted)
        },
        _ => todo!()
    }
}
