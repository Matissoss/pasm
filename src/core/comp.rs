// rasmx86_64 - comp.rs
// --------------------
// made by matissoss
// licensed under MPL

use crate::{
    core::{
        rex::gen_rex,
        modrm::gen_modrm
    },
    shr::{
        ins::Instruction as Ins,
        ast::{
            ASTInstruction,
            Operand,
            Label
        }
    }
};

#[derive(Debug, Clone)]
pub enum CompIns{
    Compiled(Vec<u8>),
    NeedsContext(ASTInstruction)
}

pub fn compile_label(lbl: Label) -> Vec<CompIns>{
    let mut bytes = Vec::new();

    for ins in &lbl.inst{
        bytes.push(compile_instruction(ins));
    }
    return bytes;
}

pub fn compile_instruction(ins: &ASTInstruction) -> CompIns{
    return match ins.ins{
        Ins::RET        => CompIns::Compiled(vec![0xC3]),
        Ins::SYSCALL    => CompIns::Compiled(vec![0x0F, 0x05]),
        Ins::PUSH       => ins_push(&ins),
        Ins::POP        => ins_pop(&ins),
        Ins::MOV        => ins_mov(&ins),
        _ => invalid()
    }
}

fn invalid() -> !{
    println!("src/core/comp.rs:invalid - unexpected;");
    std::process::exit(1);
}

fn ins_pop(ins: &ASTInstruction) -> CompIns{
    match ins.dst.clone().unwrap() {
        Operand::Reg(r) => CompIns::Compiled(vec![0x58 + r.to_byte()]),
        Operand::Mem(m) => CompIns::Compiled(vec![0x8F, gen_modrm(Some(Operand::Mem(m)), None, Some(0))]),
        _ => invalid()
    }
}

fn ins_push(ins: &ASTInstruction) -> CompIns{
    match ins.dst.clone().unwrap() {
        Operand::Reg(r) => {
            let rex = gen_rex(&ins);
            return if let Some(rex) = rex{
                CompIns::Compiled(vec![rex, 0x50 + r.to_byte()])
            }
            else {
                CompIns::Compiled(vec![0x50 + r.to_byte()])
            }
        },
        Operand::Imm(nb) => {
            match nb.size_bytes(){
                1 => {
                    CompIns::Compiled(vec![0x6A + nb.split_into_bytes()[0]])
                },
                2|4 => {
                    let mut b = vec![0x68];
                    let mut x = nb.split_into_bytes();
                    extend_imm(&mut x, nb.size_bytes());
                    b.extend(x);
                    CompIns::Compiled(b)
                }
                _ => invalid()
            }
        },
        Operand::Mem(m) => {
            if let Some(rex) = gen_rex(&ins){
                CompIns::Compiled(vec![rex, 0xFF, gen_modrm(Some(Operand::Mem(m)), None, Some(6))])
            }
            else {
                CompIns::Compiled(vec![0xFF, gen_modrm(Some(Operand::Mem(m)), None, Some(6))])
            }
        },
        _ => invalid()
    }
}

fn ins_mov(ins: &ASTInstruction) -> CompIns{
    let src = ins.src.clone().unwrap();
    let dst = ins.dst.clone().unwrap();

    let rex = gen_rex(&ins);

    if let Operand::Reg(_) = dst{
        match src{
            Operand::Imm(n) => {
                let size = dst.size_bytes();
                let opc = match size{
                    1 => 0xB0,
                    2|4|8 => 0xB8,
                    _ => invalid()
                };
                let mut imm = n.split_into_bytes();
                extend_imm(&mut imm, size);
                let mut toret = if let Some(rex) = rex{
                    vec![rex, opc]
                }
                else {
                    vec![opc]
                };
                toret.extend(imm);
                return CompIns::Compiled(toret);
            },
            Operand::Reg(_)|Operand::Mem(_) => {
                let opc = if let Operand::Reg(_) = src{
                    // r r
                    match dst.size_bytes(){
                        1 => 0x8A,
                        2|4|8 => 0x8B,
                        _ => invalid()
                    }
                }
                else{
                    // r r/m
                    match dst.size_bytes(){
                        1       => 0x88,
                        2|4|8   => 0x89,
                        _       => invalid()
                    }
                };
                let modrm = gen_modrm(Some(dst.clone()), Some(src.clone()), None);
                if let Some(rex) = rex{
                    return CompIns::Compiled(vec![rex, opc, modrm])
                }
                else {
                    return CompIns::Compiled(vec![opc, modrm])
                }
            }
            _ => invalid()
        }
    }
    else if let Operand::Mem(_) = dst{
        match src {
            Operand::Reg(_) => {
                let opc = match dst.size_bytes(){
                    1       => 0x88,
                    2|4|8   => 0x89,
                    _       => invalid()
                };
                let modrm = gen_modrm(Some(dst.clone()), Some(src.clone()), None);
                if let Some(rex) = rex{
                    return CompIns::Compiled(vec![rex, opc, modrm])
                }
                else {
                    return CompIns::Compiled(vec![opc, modrm])
                }
            },
            Operand::Imm(n) => {
                let size = dst.size_bytes();
                let opc = match size {
                    1 => 0xC6,
                    2|4|8 => 0xC7,
                    _ => invalid()
                };
                let mut imm = n.split_into_bytes();
                extend_imm(&mut imm, size);
                let modrm = gen_modrm(Some(dst.clone()), Some(src.clone()), Some(0));
                
                let mut toret = if let Some(rex) = rex{
                    vec![rex, opc, modrm]
                }
                else {
                    vec![opc, modrm]
                };
                toret.extend(imm);
                return CompIns::Compiled(toret);
            },
            _ => invalid()
        }
    }
    else {
        invalid()
    }
}

fn extend_imm(imm: &mut Vec<u8>, size: u8){
    let size = size as usize;
    while imm.len() < size{
        imm.push(0)
    }
}
