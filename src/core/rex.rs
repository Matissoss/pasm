// rasmx86_64 - rex.rs
// -------------------
// made by matissoss
// licensed under MPL

use crate::shr::{
    ast::{
        ASTInstruction,
        Operand,
    },
    ins::Instruction as Ins,
};

fn calc_size(ins: &ASTInstruction) -> usize{
    let size_dst = match &ins.dst{
        Some(o) => o.size_bytes() as usize,
        None    => 0
    };
    // we can assert that both sides have equal size
    //  check src/pre/chk.rs
    return size_dst;
}

fn needs_rex(ins: &ASTInstruction) -> bool{
    if calc_size(&ins) != 8 {
        return false;
    }
    match &ins.ins{
        Ins::ADD|Ins::MOV => true,
        _        => {
            if let Some(Operand::Reg(dst)) = ins.dst{
                if dst.needs_rex(){
                    return true;
                }
            }
            if let Some(Operand::Reg(src)) = ins.src{
                if src.needs_rex(){
                    return true;
                }
            }
            return false;
        },
    }
}

fn defaults_to_64bit(ins: &Ins) -> bool{
    return match ins {
        Ins::PUSH|Ins::POP => true,
        _ => false
    }
}

fn calc_rex(ins: &ASTInstruction) -> u8{
    // fixed pattern
    let base = 0b0100_0000;
    let w : u8 = if defaults_to_64bit(&ins.ins) {0} else {1};
    let r : u8 = if let Some(Operand::Reg(reg)) = ins.src{
        if reg.needs_rex() {1} else {0} 
    } else {0};
    // does some things in SIB.index, but that is for later
    let x : u8 = 0;
    // will be modified in sometime, because REX.B can also mean SIB.base :)
    let b : u8 = if let Some(Operand::Reg(reg)) = ins.dst{
        if reg.needs_rex() {1} else {0} 
    } else {0};

    return base + (w << 3) + (r << 2) + (x << 1) + b;
}

pub fn gen_rex(ins: &ASTInstruction) -> Option<u8>{
    if needs_rex(ins) {
        return Some(calc_rex(ins));
    }
    else {
        return None;
    }
}

#[cfg(test)]
mod tests{
    #[allow(unused)]
    use super::*;
    #[allow(unused)]
    use crate::shr::reg::Register;
    #[test]
    fn gen_rex_t(){
        let instruction = ASTInstruction{
            ins: Ins::MOV,
            dst: Some(Operand::Reg(Register::R8)),
            src: Some(Operand::Reg(Register::R9)),
            lin: 0
        };
        // 64-bit operand used, source = extended register and destination = extended register
        assert!(Some(0b100_1101) == gen_rex(&instruction));
        drop(instruction);
        let instruction = ASTInstruction{
            ins: Ins::MOV,
            dst: Some(Operand::Reg(Register::R8)),
            src: Some(Operand::Reg(Register::RAX)),
            lin: 0
        };
        // 64-bit operand used and destination = extended register
        assert!(Some(0b100_1001) == gen_rex(&instruction));
        drop(instruction);
        let instruction = ASTInstruction{
            ins: Ins::MOV,
            dst: Some(Operand::Reg(Register::RAX)),
            src: Some(Operand::Reg(Register::RBX)),
            lin: 0
        };
        //      only W = 1, 64-bit operand used
        assert!(Some(0b100_1000) == gen_rex(&instruction));
        drop(instruction);
        let instruction = ASTInstruction{
            ins: Ins::MOV,
            dst: Some(Operand::Reg(Register::RAX)),
            src: Some(Operand::Reg(Register::R8)),
            lin: 0
        };
        //      only W = 1, 64-bit operand used and src is extended register
        assert!(Some(0b100_1100) == gen_rex(&instruction));
    }
}
