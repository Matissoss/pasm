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
        reloc::{
            RType,
            Relocation,
            RCategory,
        }
    },
    shr::{
        ins::Mnemonic as Ins,
        ast::{
            Instruction,
            Operand,
            Label,
        },
        size::Size,
        reg::Register,
        num::Number,
        rpanic::rpanic,
        var::{
            Variable,
            VarContent
        },
        symbol::{
            Symbol,
            SymbolType,
            Visibility
        }
    }
};

#[inline]
pub fn make_globals(symbols: &mut [Symbol], globals: &[String]){
    for s in symbols{
        for g in globals{
            if &s.name == g {
                s.visibility = Visibility::Global;
                break;
            }
        }
    }
}

pub fn compile_section(vars: Vec<Variable>, sindex: u16, addt: u8) -> (Vec<u8>, Vec<Symbol>){
    let mut buf: Vec<u8> = Vec::new();
    let mut symbols: Vec<Symbol> = Vec::new();

    let mut offset : u32 = 0;

    for v in vars{
        match v.content{
            VarContent::Uninit => {
                symbols.push(Symbol{
                    name: v.name,
                    size: Some(v.size),
                    sindex,
                    stype: SymbolType::Object,
                    offset,
                    content: None,
                    visibility: v.visibility,
                    addt
                });
                offset += v.size;
            },
            _ => {
                buf.extend(v.content.bytes());
                symbols.push(Symbol{
                    name: v.name,
                    size: Some(v.size),
                    sindex,
                    stype: SymbolType::Object,
                    offset,
                    content: Some(v.content),
                    visibility: v.visibility,
                    addt
                });
                offset += v.size;
            }
        }
    }

    (buf, symbols)
}

pub fn compile_label(lbl: Label) -> (Vec<u8>, Vec<Relocation>){
    let mut bytes = Vec::new();
    let mut reallocs = Vec::new();
    for ins in &lbl.inst{
        let res = compile_instruction(ins);
        if let Some(mut rl) = res.1 {
            rl.offset += bytes.len() as u32;
            reallocs.push(rl);
        }
        bytes.extend(res.0);
    }
    return (bytes, reallocs);
}


pub fn compile_instruction(ins: &Instruction) -> (Vec<u8>, Option<Relocation>){
    return match ins.mnem{
        Ins::RET        => (vec![0xC3], None),
        Ins::SYSCALL    => (vec![0x0F, 0x05], None),
        Ins::PUSH       => (ins_push(&ins), None),
        Ins::POP        => (ins_pop(&ins), None),
        Ins::MOV        => (ins_mov(&ins), None),
        Ins::ADD        => (add_like_ins(&ins, 
            &[0x04, 0x05, 0x80, 0x81, 0x83, 0x00, 0x01, 0x02, 0x03], 0), None),
        Ins::OR         => (add_like_ins(&ins,
            &[0x0C, 0x0D, 0x80, 0x81, 0x83, 0x08, 0x09, 0x0A, 0x0B], 1), None),
        Ins::AND        => (add_like_ins(&ins,
            &[0x24, 0x25, 0x80, 0x81, 0x83, 0x20, 0x21, 0x22, 0x23], 4), None),
        Ins::SUB        => (add_like_ins(&ins,
            &[0x2C, 0x2D, 0x80, 0x81, 0x83, 0x28, 0x29, 0x2A, 0x2B], 5), None),
        Ins::XOR        => (add_like_ins(&ins,
            &[0x34, 0x35, 0x80, 0x81, 0x83, 0x30, 0x31, 0x32, 0x33], 6), None),
        Ins::SAL|Ins::SHL => (ins_shllike(&ins, 
            &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 4), None),
        Ins::SHR        => (ins_shllike(&ins, 
            &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 5), None),
        Ins::SAR        => (ins_shllike(&ins, 
            &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 7), None),
        Ins::TEST       => (ins_test(&ins), None),
        Ins::INC        => (ins_inclike(&ins, &[0xFE, 0xFF], 0), None),
        Ins::DEC        => (ins_inclike(&ins, &[0xFE, 0xFF], 1), None),
        Ins::NOT        => (ins_inclike(&ins, &[0xF6, 0xF7], 2), None),
        Ins::NEG        => (ins_inclike(&ins, &[0xF6, 0xF7], 3), None),
        Ins::CMP        => (ins_cmp(&ins), None),
        Ins::IMUL       => (ins_imul(&ins), None),
        Ins::DIV        => (ins_divmul(&ins, 6), None),
        Ins::IDIV       => (ins_divmul(&ins, 7), None),
        Ins::MUL        => (ins_divmul(&ins, 4), None),
        Ins::JMP        => ins_jmplike(&ins, vec![0xE9]),
        Ins::CALL       => ins_jmplike(&ins, vec![0xE8]),
        Ins::JE |Ins::JZ         
                        => ins_jmplike(&ins, vec![0x0F, 0x84]),
        Ins::JNE|Ins::JNZ        
                        => ins_jmplike(&ins, vec![0xFF, 0x85]),
        Ins::JL         => ins_jmplike(&ins, vec![0x0F, 0x8C]),
        Ins::JLE        => ins_jmplike(&ins, vec![0x0F, 0x8E]),
        Ins::JG         => ins_jmplike(&ins, vec![0x0F, 0x8F]),
        Ins::JGE        => ins_jmplike(&ins, vec![0x0F, 0x8D]),

        Ins::LEA        => ins_lea(&ins),
        //_ => (Vec::new(), None)
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
                if size == Size::Qword{
                    extend_imm(&mut imm, 4);
                }
                else{
                    extend_imm(&mut imm, size.into());
                }
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

// opc[0]  = AL, imm8
// opc[1]  = AX/EAX/RAX, imm32
// opc[2]  = r/m8, imm8
// opc[3]  = r/m16/32/64, imm16/32
// opc[4] =  r/m16/32/64, imm8
// opc[5]  = r/m8, r8
// opc[6]  = r/m16/32/64, r16/32/64
// opc[7]  = r8, r/m8
// opc[8]  = r16/32/64, r/m16/32/64
fn add_like_ins(ins: &Instruction, opc: &[u8; 9], ovrreg: u8) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    return match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            if let Size::Dword|Size::Word = srci.size(){
                if let Register::RAX|Register::EAX = dstr{
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[opc[1]], &imm);
                }
                else if let Register::AX = dstr{
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[opc[1]], &imm);
                }
            }
            if let Register::AL = dstr{
                if let Size::Byte = srci.size(){
                    return bs_imm(ins, &[opc[0]], &imm);
                }
            }
            else if let Register::AX = dstr{
                if let Size::Byte = srci.size(){
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[opc[1]], &imm);
                }
            }

            let opc = match dstr.size(){
                Size::Byte => opc[2],
                Size::Dword|Size::Qword|Size::Word => {
                    if imm.len() == 1 {
                        opc[4]
                    }
                    else{
                        opc[3]
                    }
                },
                _ => invalid()
            };
            let mut base = gen_base(ins, &[opc]);
            base.push(gen_modrm(ins, Some(ovrreg), None));
            extend_imm(&mut imm, 1);
            base.extend(imm);
            return base;
        },
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.size(){
                Size::Byte => opc[2],
                Size::Word => opc[3],
                Size::Dword => opc[3],
                Size::Qword => {
                    if imm.len() == 1 {
                        opc[4]
                    }
                    else{
                        opc[3]
                    }
                },
                _ => invalid()
            };
            if let (Size::Word|Size::Byte, Size::Word) = (srci.size(), dstm.size()){
                extend_imm(&mut imm, 2);
            }
            else if let (Size::Byte, Size::Dword) = (srci.size(), dstm.size()){
                extend_imm(&mut imm, 4);
            }
            else if let (crate::shr::ins::Mnemonic::CMP, Size::Byte, Size::Qword) = (ins.mnem, srci.size(), dstm.size()){
                extend_imm(&mut imm, 4);
            }
            else if srci.size() != Size::Byte{
                extend_imm(&mut imm, 4);
            }

            return gen_ins(ins, &[opc], (true, Some(ovrreg), None), Some(imm));
        },
        (Operand::Reg(r), Operand::Mem(_)|Operand::Reg(_)) => {
            let opc = match r.size(){
                Size::Byte => opc[7],
                Size::Word|Size::Dword|Size::Qword => opc[6],
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None)
        },
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size(){
                Size::Byte => opc[7],
                Size::Word|Size::Dword|Size::Qword => opc[6],
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None)
        },
        _ => invalid()
    }
}

fn ins_cmp(ins: &Instruction) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    return match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            if let Size::Dword|Size::Word = srci.size(){
                if let Register::RAX|Register::EAX = dstr{
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[0x3D], &imm);
                }
                else if let Register::AX = dstr{
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0x3D], &imm);
                }
            }
            if let Register::AL = dstr{
                if let Size::Byte = srci.size(){
                    return bs_imm(ins, &[0x3C], &imm);
                }
            }
            else if let Register::AX = dstr{
                if let Size::Byte = srci.size(){
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0x3D], &imm);
                }
            }

            let opc = match dstr.size(){
                Size::Byte => 0x80,
                Size::Dword|Size::Qword|Size::Word => {
                    if imm.len() == 1 {
                        if imm[0] <= 127{
                            0x83
                        }
                        else {
                            0x80
                        }
                    }
                    else{
                        0x80
                    }
                },
                _ => invalid()
            };
            let mut base = gen_base(ins, &[opc]);
            base.push(gen_modrm(ins, Some(7), None));
            extend_imm(&mut imm, 1);
            base.extend(imm);
            return base;
        },
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.size(){
                Size::Byte => 0x80,
                Size::Qword|Size::Word|Size::Dword => { 
                    if imm.len() == 1 {
                        if imm[0] <= 127{
                            0x83
                        }
                        else {
                            0x81
                        }
                    }
                    else{
                        0x81
                    }
                },
                _ => invalid()
            };
            if let (Size::Word|Size::Byte, Size::Word) = (srci.size(), dstm.size()){
                extend_imm(&mut imm, 2);
            }
            else if let (Size::Byte, Size::Dword|Size::Qword) = (srci.size(), dstm.size()){
                extend_imm(&mut imm, 4);
            }
            else if srci.size() != Size::Byte{
                extend_imm(&mut imm, 4);
            }

            return gen_ins(ins, &[opc], (true, Some(7), None), Some(imm));
        },
        (Operand::Reg(r), Operand::Mem(_)|Operand::Reg(_)) => {
            let opc = match r.size(){
                Size::Byte => 0x3A,
                Size::Word|Size::Dword|Size::Qword => 0x3B,
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None)
        },
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size(){
                Size::Byte => 0x38,
                Size::Word|Size::Dword|Size::Qword => 0x39,
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None)
        },
        _ => invalid()
    }

}

fn ins_test(ins: &Instruction) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    return match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            if let Size::Dword|Size::Word = srci.size(){
                if let Register::RAX|Register::EAX = dstr{
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[0xA9], &imm);
                }
                else if let Register::AX = dstr{
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0xA9], &imm);
                }
            }
            if let Register::AL = dstr{
                if let Size::Byte = srci.size(){
                    return bs_imm(ins, &[0xA8], &imm);
                }
            }
            else if let Register::AX = dstr{
                if let Size::Byte = srci.size(){
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0xA9], &imm);
                }
            }

            let opc = match dstr.size(){
                Size::Byte => 0xF6,
                Size::Dword|Size::Qword|Size::Word => 0xF7,
                _ => invalid()
            };
            let mut base = gen_base(ins, &[opc]);
            base.push(gen_modrm(ins, Some(0), None));
            extend_imm(&mut imm, 1);
            base.extend(imm);
            return base;
        },
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.size(){
                Size::Byte => 0xF6,
                Size::Qword|Size::Word|Size::Dword => 0xF7,
                _ => invalid()
            };
            if let (Size::Word|Size::Byte, Size::Word) = (srci.size(), dstm.size()){
                extend_imm(&mut imm, 2);
            }
            else if let (Size::Byte, Size::Dword|Size::Qword) = (srci.size(), dstm.size()){
                extend_imm(&mut imm, 4);
            }
            else if srci.size() != Size::Byte{
                extend_imm(&mut imm, 4);
            }

            return gen_ins(ins, &[opc], (true, Some(0), None), Some(imm));
        },
        (Operand::Reg(_)|Operand::Mem(_), Operand::Reg(_)) => {
            let opc = match dst.size(){
                Size::Byte => 0x84,
                Size::Word|Size::Dword|Size::Qword => 0x85,
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

// opc[0] = r/m8, 1
// opc[1] = r/m8, cl
// opc[2] = r/m8, imm8
// opc[3] = r/m16/32/64, 1
// opc[4] = r/m16/32/64, cl
// opc[5] = r/m16/32/64, imm8
fn ins_shllike(ins: &Instruction, opc: &[u8; 6], ovr: u8) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();
    let (opcd, imm) = match src{
        Operand::Reg(Register::CL) => {
            match dst.size(){
                Size::Byte => (opc[1], None),
                Size::Word|Size::Dword|Size::Qword => (opc[4], None),
                _ => invalid(),
            }
        },
        Operand::Imm(Number::UInt8(1)|Number::Int8(1)) => {
            match dst.size(){
                Size::Byte => (opc[0], None),
                Size::Word|Size::Dword|Size::Qword => (opc[3], None),
                _ => invalid(),
            }
        },
        Operand::Imm(imm) => {
            match dst.size(){
                Size::Byte => (opc[2], Some(imm.split_into_bytes())),
                Size::Word|Size::Dword|Size::Qword => (opc[5], Some(imm.split_into_bytes())),
                _ => invalid()
            }
        },
        _ => invalid()
    };
    let mut base = if dst.size() == Size::Word {vec![0x66]} else {vec![]};
    let gen_b = gen_base(&ins, &[opcd]);
    if gen_b[0] == 0x66{
        base = gen_b;
    }
    else {
        base.extend(gen_b);
    }
    base.push(gen_modrm(&ins, Some(ovr), None));
    if let Some(sib) = gen_sib(dst){
        base.push(sib);
    }
    if let Some(dsp) = gen_disp(dst){
        base.extend(dsp);
    }
    if let Some(imm) = imm{
        base.extend(imm);
    }
    base
}

fn ins_inclike(ins: &Instruction, opc: &[u8; 2], ovr: u8) -> Vec<u8> {
    let opc = match ins.dst().unwrap().size(){
        Size::Byte => opc[0],
        _          => opc[1],
    };
    gen_ins(ins, &[opc], (true, Some(ovr), None), None)
}

fn ins_lea(ins: &Instruction) -> (Vec<u8>, Option<Relocation>) {
    let mut base = gen_base(ins, &[0x8D]);
    let modrm = if let Operand::Reg(r) = ins.dst().unwrap(){
        4 + r.to_byte()
    } else {0};
    base.push(modrm);
    base.push(0x25);
    let symbol = match ins.src().unwrap(){
        Operand::SymbolRef(s) => s.to_string(),
        _ => invalid()
    };
    let blen = base.len();
    base.extend([0x00; 4]);
    (base, Some(Relocation{
        rtype: RType::PCRel32,
        symbol,
        offset: blen as u32,
        addend: 0,
        size: 4,
        catg: RCategory::Lea,
    }))
}

// opc = opcode ONLY for rel32
fn ins_jmplike(ins: &Instruction, opc: Vec<u8>) -> (Vec<u8>, Option<Relocation>){
    if let Operand::SymbolRef(s) = ins.dst().unwrap(){
        let mut rel = Relocation{
            rtype: RType::PCRel32,
            symbol: s.to_string(),
            addend: 0,
            offset: 1,
            size  : 4,
            catg  : RCategory::Jump,
        };
        if opc.len() == 1{
            (vec![opc[0], 0, 0, 0, 0], Some(rel))
        }
        else {
            let len = opc.len();
            let mut bs = opc;
            bs.extend([0, 0, 0, 0]);
            rel.offset     = len as u32 + 1;
            rel.size       = bs.len() as u8;
            (bs, Some(rel))
        }
    }
    else {
        invalid()
    }
}

// ==============================
// Utils

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
    rpanic("comp.rs", "some function", 1, "Unexpected thing that should not happen")
}
