// rasmx86_64 - comp.rs
// --------------------
// made by matissoss
// licensed under MPL

use crate::{
    core::{
        rex::gen_rex,
        modrm::gen_modrm,
        disp::gen_disp,
        sib::gen_sib,
    },
    shr::{
        ins::Mnemonic as Ins,
        ast::{
            Instruction,
            Operand,
            Label
        },
        size::Size,
    }
};

#[derive(Debug, Clone)]
pub enum CompIns{
    Compiled(Vec<u8>),
    NeedsContext(Instruction)
}

pub fn compile_label(lbl: Label) -> Vec<CompIns>{
    let mut bytes = Vec::new();

    for ins in &lbl.inst{
        bytes.push(compile_instruction(ins));
    }
    return bytes;
}

pub fn compile_instruction(ins: &Instruction) -> CompIns{
    return match ins.mnem{
        Ins::RET        => CompIns::Compiled(vec![0xC3]),
        Ins::SYSCALL    => CompIns::Compiled(vec![0x0F, 0x05]),
        Ins::PUSH       => ins_push(&ins),
        Ins::POP        => ins_pop(&ins),
        Ins::MOV        => ins_mov(&ins),
        _ => CompIns::NeedsContext(ins.clone())
    }
}

fn invalid() -> !{
    println!("src/core/comp.rs:invalid - unexpected;");
    std::process::exit(1);
}

fn ins_pop(ins: &Instruction) -> CompIns{
    match ins.dst().clone().unwrap() {
        Operand::Reg(r) => CompIns::Compiled(vec![0x58 + r.to_byte()]),
        Operand::Mem(m) => CompIns::Compiled(vec![0x8F, gen_modrm(Some(Operand::Mem(m.clone())), None, Some(0))]),
        _ => invalid()
    }
}

fn ins_push(ins: &Instruction) -> CompIns{
    match ins.dst().clone().unwrap() {
        Operand::Reg(r) => {
            if let Some(rex) = gen_rex(&ins){
                CompIns::Compiled(vec![rex, 0x50 + r.to_byte()])
            }
            else {
                CompIns::Compiled(vec![0x50 + r.to_byte()])
            }
        },
        Operand::Imm(nb) => {
            match nb.size(){
                Size::Byte => {
                    let mut opc = vec![0x6A];
                    opc.extend(nb.split_into_bytes());
                    CompIns::Compiled(opc)
                },
                Size::Word|Size::Dword => {
                    let mut b = vec![0x68];
                    let mut x = nb.split_into_bytes();
                    extend_imm(&mut x, 4);
                    b.extend(x);
                    CompIns::Compiled(b)
                }
                _ => invalid()
            }
        },
        Operand::Mem(_) => {
            CompIns::Compiled(gen_opc(&ins, &[0xFF], Some(6)))
        },
        _ => invalid()
    }
}

fn ins_mov(ins: &Instruction) -> CompIns{
    let src = ins.src().clone().unwrap();
    let dst = ins.dst().clone().unwrap();

    let rex = gen_rex(&ins);

    if let Operand::Reg(_) = dst{
        match src{
            Operand::Imm(n) => {
                let size = dst.size();
                let opc = match size as u8{
                    1 => 0xB0,
                    2|4|8 => 0xB8,
                    _ => invalid()
                };
                let mut imm = n.split_into_bytes();
                extend_imm(&mut imm, size as u8);
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
                    match dst.size() as u8{
                        1 => 0x8A,
                        2|4|8 => 0x8B,
                        _ => invalid()
                    }
                }
                else{
                    // r r/m
                    match dst.size() as u8{
                        1       => 0x88,
                        2|4|8   => 0x89,
                        _       => invalid()
                    }
                };
                return CompIns::Compiled(gen_opc(&ins, &vec![opc], None));
            }
            _ => invalid()
        }
    }
    else if let Operand::Mem(_) = dst{
        match src {
            Operand::Reg(_) => {
                let opc = match dst.size() as u8{
                    1       => 0x88,
                    2|4|8   => 0x89,
                    _       => invalid()
                };
                return CompIns::Compiled(gen_opc(&ins, &vec![opc], None));
            },
            Operand::Imm(n) => {
                let size = dst.size() as u8;
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

fn gen_opc(ctx: &Instruction, op: &[u8], ovr: Option<u8>) -> Vec<u8>{
    let mut base = if let Some(rex) = gen_rex(ctx){
        let mut x = vec![rex];
        x.extend(op.to_vec());
        x
    }
    else {
        op.to_vec()
    };
    base.push(gen_modrm(ctx.dst().cloned(), ctx.src().cloned(), ovr));
    if let Some(sib) = gen_sib(ctx.dst().expect("assertion dst == Some failed").clone()){
        base.push(sib);
    }
    if let Some(disp) = gen_disp(ctx.dst().expect("assertion dst == Some failed").clone()){
        base.extend(disp);
    }
    return base;
}
