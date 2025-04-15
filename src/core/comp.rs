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
        reg::Register,
    }
};

pub fn compile_label(lbl: Label) -> Vec<u8>{
    let mut bytes = Vec::new();

    for ins in &lbl.inst{
        bytes.extend(compile_instruction(ins));
    }
    return bytes;
}

pub fn compile_instruction(ins: &Instruction) -> Vec<u8>{
    return match ins.mnem{
        Ins::RET        => vec![0xC3],
        Ins::SYSCALL    => vec![0x0F, 0x05],
        Ins::PUSH       => ins_push(&ins),
        Ins::POP        => ins_pop(&ins),
        Ins::MOV        => ins_mov(&ins),
        Ins::ADD        => ins_add(&ins),
        Ins::SUB        => ins_sub(&ins),
        Ins::IMUL       => ins_imul(&ins),
        Ins::DIV        => ins_divmul(&ins, 6),
        Ins::IDIV       => ins_divmul(&ins, 7),
        Ins::MUL        => ins_divmul(&ins, 4),
        _ => Vec::new()
    }
}

fn ins_pop(ins: &Instruction) -> Vec<u8>{
    match ins.dst().clone().unwrap() {
        Operand::Reg(r) => gen_base(ins, &[0x58 + r.to_byte()]),
        Operand::Mem(_) => vec![0x8F, gen_modrm(&ins, None, Some(0))],
        _ => invalid()
    }
}

fn ins_push(ins: &Instruction) -> Vec<u8>{
    return match ins.dst().clone().unwrap() {
        Operand::Reg(r) => gen_base(ins, &[0x50 + r.to_byte()]),
        Operand::Imm(nb) => {
            match nb.size(){
                Size::Byte => {
                    let mut opc = vec![0x6A];
                    opc.extend(nb.split_into_bytes());
                    opc
                },
                Size::Word|Size::Dword => {
                    let mut b = vec![0x68];
                    let mut x = nb.split_into_bytes();
                    extend_imm(&mut x, 4);
                    b.extend(x);
                    b
                }
                _ => invalid()
            }
        },
        Operand::Mem(_) => gen_ins(&ins, &[0xFF], (true, Some(6), None), None),
        _ => invalid()
    }
}

fn ins_mov(ins: &Instruction) -> Vec<u8>{
    let src = ins.src().clone().unwrap();
    let dst = ins.dst().clone().unwrap();
    if let Operand::Reg(r) = dst{
        match src{
            Operand::Imm(n) => {
                let size = dst.size();
                let opc = match size{
                    Size::Byte => 0xB0 + r.to_byte(),
                    Size::Word|Size::Dword|Size::Qword => 0xB8 + r.to_byte(),
                    _ => invalid()
                };
                let mut imm = n.split_into_bytes();
                extend_imm(&mut imm, size as u8 + 1);
                let mut base = gen_base(ins, &[opc]);
                base.extend(imm);
                return base;
            },
            Operand::Reg(_)|Operand::Mem(_) => {
                let opc = if let Operand::Reg(_) = src{
                    match dst.size(){
                        Size::Byte => 0x88,
                        Size::Word|Size::Dword|Size::Qword => 0x89,
                        _ => invalid()
                    }
                }
                else{
                    match dst.size(){
                        Size::Byte => 0x8A,
                        Size::Word|Size::Dword|Size::Qword => 0x8B,
                        _       => invalid()
                    }
                };
                return gen_ins(&ins, &[opc], (true, None, None), None);
            }
            _ => invalid()
        }
    }
    else if let Operand::Mem(_) = dst{
        match src {
            Operand::Reg(_) => {
                let opc = match dst.size(){
                    Size::Byte => 0x88,
                    Size::Word|Size::Dword|Size::Qword => 0x89,
                    _       => invalid()
                };
                return gen_ins(&ins, &[opc], (true, None, None), None);
            },
            Operand::Imm(n) => {
                let size = dst.size();
                let opc = match size {
                    Size::Byte => 0xC6,
                    Size::Word|Size::Dword|Size::Qword => 0xC7,
                    _ => invalid()
                };
                let mut imm = n.split_into_bytes();
                extend_imm(&mut imm, size as u8 + 1);
                return gen_ins(ins, &[opc], (true, Some(0), None), Some(imm));
            },
            _ => invalid()
        }
    }
    else {
        invalid()
    }
}

fn ins_add(ins: &Instruction) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    return match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            if let Size::Dword|Size::Word = srci.size(){
                if let Register::RAX|Register::EAX = dstr{
                    let mut imm = srci.split_into_bytes();
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[0x05], &imm);
                }
                else if let Register::AX = dstr{
                    let mut imm = srci.split_into_bytes();
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0x05], &imm);
                }
                else if let Register::AL = dstr{
                    let imm = srci.split_into_bytes();
                    return bs_imm(ins, &[0x04], &imm);
                }
            }
            let mut imm = srci.split_into_bytes();
            let opc = match dstr.size(){
                Size::Byte => 0x80,
                Size::Word|Size::Dword|Size::Qword => {
                    if imm.len() == 1 {
                        0x83
                    }
                    else{
                        0x81
                    }
                },
                _ => invalid()
            };
            let mut base = gen_base(ins, &[opc]);
            base.push(gen_modrm(ins, None, Some(0)));
            extend_imm(&mut imm, 1);
            base.extend(imm);
            return base;
        },
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.size(){
                Size::Byte => 0x80,
                Size::Word|Size::Dword|Size::Qword => {
                    if imm.len() == 1 {
                        0x83
                    }
                    else{
                        0x81
                    }
                },
                _ => invalid()
            };
            if srci.size() != Size::Byte{
                extend_imm(&mut imm, 4);
            }
            return gen_ins(ins, &[opc], (true, None, None), Some(imm));
        },
        (Operand::Reg(r), Operand::Mem(_)|Operand::Reg(_)) => {
            let opc = match r.size(){
                Size::Byte => 0x02,
                Size::Word|Size::Dword|Size::Qword => 0x01,
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None)
        },
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size(){
                Size::Byte => 0x00,
                Size::Word|Size::Dword|Size::Qword => 0x01,
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None)
        },
        _ => invalid()
    }
}
fn ins_sub(ins: &Instruction) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    return match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            if let Size::Dword|Size::Word = srci.size(){
                if let Register::RAX|Register::EAX = dstr{
                    let mut imm = srci.split_into_bytes();
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[0x2D], &imm);
                }
                else if let Register::AX = dstr{
                    let mut imm = srci.split_into_bytes();
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0x2D], &imm);
                }
                else if let Register::AL = dstr{
                    let imm = srci.split_into_bytes();
                    return bs_imm(ins, &[0x2C], &imm);
                }
            }
            let mut imm = srci.split_into_bytes();
            let opc = match dstr.size(){
                Size::Byte => 0x80,
                Size::Word|Size::Dword|Size::Qword => {
                    if imm.len() == 1 {
                        0x83
                    }
                    else{
                        0x81
                    }
                },
                _ => invalid()
            };
            let mut base = gen_base(ins, &[opc]);
            base.push(gen_modrm(ins, Some(5), None));
            extend_imm(&mut imm, 1);
            base.extend(imm);
            return base;
        },
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.size(){
                Size::Byte => 0x80,
                Size::Word|Size::Dword|Size::Qword => {
                    if imm.len() == 1 {
                        0x83
                    }
                    else{
                        0x81
                    }
                },
                _ => invalid()
            };
            if srci.size() != Size::Byte{
                extend_imm(&mut imm, 4);
            }
            return gen_ins(ins, &[opc], (true, Some(5), None), Some(imm));
        },
        (Operand::Reg(r), Operand::Mem(_)|Operand::Reg(_)) => {
            let opc = match r.size(){
                Size::Byte => 0x2A,
                Size::Word|Size::Dword|Size::Qword => 0x29,
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None)
        },
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size(){
                Size::Byte => 0x28,
                Size::Word|Size::Dword|Size::Qword => 0x29,
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None)
        },
        _ => invalid()
    }
}

fn ins_imul(ins: &Instruction) -> Vec<u8>{
    match ins.src(){
        None => {
            let opc = match ins.dst().unwrap().size(){
                Size::Byte => &[0xF6],
                _ => &[0xF7]
            };
            gen_ins(ins, opc, (true, Some(5), None), None)
        }
        Some(_) => {
            match ins.oprs.get(2){
                Some(Operand::Imm(imm)) => {
                    let (opc, size) = match imm.size(){
                        Size::Byte => (0x6B, 1),
                        Size::Word => (0x69, 2),
                        _          => (0x69, 4)
                    };
                    let mut imm_b = imm.split_into_bytes();
                    extend_imm(&mut imm_b, size);
                    let (dst, src) = if let (Some(Operand::Reg(r)), Some(Operand::Reg(r1))) = (ins.dst(), ins.src()) {
                        (Some(r.to_byte()), Some(r1.to_byte()))
                    } else {(None, None)};
                    gen_ins(ins, &[opc], (true, dst, src), Some(imm_b))
                },
                _ => {
                    gen_ins(ins, &[0x0F, 0xAF], (true, None, None), None)
                }
            }
        }
    }
}

fn bs_imm(ins: &Instruction, opc: &[u8], imm: &[u8]) -> Vec<u8>{
    let mut b = gen_base(ins, opc);
    b.extend(imm);
    b
}

fn extend_imm(imm: &mut Vec<u8>, size: u8){
    let size = size as usize;
    while imm.len() < size{
        imm.push(0)
    }
}

fn gen_ins(ins: &Instruction, opc: &[u8], modrm: (bool, Option<u8>, Option<u8>), imm: Option<Vec<u8>>) -> Vec<u8> {
    let mut base = gen_base(ins, opc);
    if modrm.0{
        base.push(gen_modrm(ins, modrm.1, modrm.2));

        if let Some(dst) = ins.dst(){
            if let Some(sib) = gen_sib(dst){
                base.push(sib);
            }
        }
        if let Some(src) = ins.src(){
            if let Some(sib) = gen_sib(src){
                base.push(sib);
            }
        }
    }

    if let Some(dst) = ins.dst(){
        if let Some(disp) = gen_disp(dst){
            base.extend(disp);
        }
    }
    if let Some(src) = ins.src(){
        if let Some(disp) = gen_disp(src){
            base.extend(disp);
        }
    }
    if let Some(imm) = imm{
        base.extend(imm);
    }
    return base;
}

fn ins_divmul(ins: &Instruction, ovr: u8) -> Vec<u8>{
    let opc = match ins.dst().unwrap().size(){
        Size::Byte => [0xF6],
        _ => [0xF7]
    };
    return gen_ins(ins, &opc, (true, Some(ovr), None), None);
}

#[inline]
fn gen_base(ins: &Instruction, opc: &[u8]) -> Vec<u8>{
    if ins.size() < Size::Dword && ins.size() != Size::Byte{
        if let Some(rex) = gen_rex(ins){
            let mut v = vec![0x66, rex];
            v.extend(opc);
            v
        }
        else {
            let mut i = vec![0x66];
            i.extend(opc);
            return i;
        }
    }
    else if let Some(rex) = gen_rex(ins){
        let mut v = vec![rex];
        v.extend(opc);
        v
    }
    else {
        opc.to_vec()
    }
}

fn invalid() -> !{
    println!("src/core/comp.rs:invalid - unexpected;");
    std::process::exit(1);
}

