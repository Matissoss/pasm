// rasmx86_64 - src/core/comp.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;

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
    },
};

#[inline]
pub fn make_globals(symbols: &mut [Symbol], globals: &[String]){
    for s in symbols{
        for g in globals{
            if s.name == Cow::Borrowed(g) {
                s.visibility = Visibility::Global;
                break;
            }
        }
    }
}
#[inline]
pub fn extern_trf(externs: &Vec<String>) -> Vec<Symbol>{
    let mut symbols = Vec::new();
    for extern_ in externs{
        symbols.push(Symbol{
            name        : Cow::Borrowed(extern_),
            offset      : 0,
            size        : None,
            sindex      : 0,
            stype       : SymbolType::NoType,
            visibility  : Visibility::Global,
            content     : None,
            addend      : 0,
            addt        : 0,
        });
    }
    return symbols;
}

pub fn compile_section<'a>(vars: &'a Vec<&'a Variable<'a>>, sindex: u16, addt: u8) -> (Vec<u8>, Vec<Symbol<'a>>){
    let mut buf: Vec<u8> = Vec::new();
    let mut symbols: Vec<Symbol> = Vec::new();

    let mut offset : u64 = 0;

    for v in vars{
        match v.content{
            VarContent::Uninit => {
                symbols.push(Symbol{
                    name: Cow::Borrowed(&v.name),
                    size: Some(v.size),
                    sindex,
                    stype: SymbolType::Object,
                    offset,
                    content: None,
                    visibility: v.visibility,
                    addend: 0,
                    addt
                });
                offset += v.size as u64;
            },
            _ => {
                buf.extend(v.content.bytes());
                symbols.push(Symbol{
                    name: Cow::Borrowed(&v.name),
                    size: Some(v.size),
                    sindex,
                    stype: SymbolType::Object,
                    offset,
                    content: Some(Cow::Borrowed(&v.content)),
                    visibility: v.visibility,
                    addend: 0,
                    addt
                });
                offset += v.size as u64;
            }
        }
    }

    (buf, symbols)
}

pub fn compile_label<'a>(lbl: &'a Vec<Instruction>, bits: u8) -> (Vec<u8>, Vec<Relocation<'a>>){
    let mut bytes = Vec::new();
    let mut reallocs = Vec::new();
    for ins in lbl{
        let res = compile_instruction(ins, bits);
        if let Some(mut rl) = res.1 {
            rl.offset += bytes.len() as u64;
            reallocs.push(rl);
        }
        bytes.extend(res.0);
    }
    return (bytes, reallocs);
}


pub fn compile_instruction(ins: &Instruction, bits: u8) -> (Vec<u8>, Option<Relocation>){
    return match ins.mnem{
        Ins::RET        => (vec![0xC3], None),
        Ins::SYSCALL    => (vec![0x0F, 0x05], None),
        Ins::PUSH       => (ins_push(&ins, bits), None),
        Ins::POP        => (ins_pop(&ins, bits), None),
        Ins::MOV        => (ins_mov(&ins, bits), None),
        Ins::ADD        => (add_like_ins(&ins, 
            &[0x04, 0x05, 0x80, 0x81, 0x83, 0x00, 0x01, 0x02, 0x03], 0, bits), None),
        Ins::OR         => (add_like_ins(&ins,
            &[0x0C, 0x0D, 0x80, 0x81, 0x83, 0x08, 0x09, 0x0A, 0x0B], 1, bits), None),
        Ins::AND        => (add_like_ins(&ins,
            &[0x24, 0x25, 0x80, 0x81, 0x83, 0x20, 0x21, 0x22, 0x23], 4, bits), None),
        Ins::SUB        => (add_like_ins(&ins,
            &[0x2C, 0x2D, 0x80, 0x81, 0x83, 0x28, 0x29, 0x2A, 0x2B], 5, bits), None),
        Ins::XOR        => (add_like_ins(&ins,
            &[0x34, 0x35, 0x80, 0x81, 0x83, 0x30, 0x31, 0x32, 0x33], 6, bits), None),
        Ins::SAL|Ins::SHL => (ins_shllike(&ins, 
            &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 4, bits), None),
        Ins::SHR        => (ins_shllike(&ins, 
            &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 5, bits), None),
        Ins::SAR        => (ins_shllike(&ins, 
            &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 7, bits), None),
        Ins::TEST       => (ins_test(&ins, bits), None),
        Ins::INC        => (ins_inclike(&ins, &[0xFE, 0xFF], 0, bits), None),
        Ins::DEC        => (ins_inclike(&ins, &[0xFE, 0xFF], 1, bits), None),
        Ins::NOT        => (ins_inclike(&ins, &[0xF6, 0xF7], 2, bits), None),
        Ins::NEG        => (ins_inclike(&ins, &[0xF6, 0xF7], 3, bits), None),
        Ins::CMP        => (ins_cmp(&ins, bits), None),
        Ins::IMUL       => (ins_imul(&ins, bits), None),
        Ins::DIV        => (ins_divmul(&ins, 6, bits), None),
        Ins::IDIV       => (ins_divmul(&ins, 7, bits), None),
        Ins::MUL        => (ins_divmul(&ins, 4, bits), None),
        Ins::JMP        => ins_jmplike(&ins, [vec![0xE9], vec![0xFF]], 4, bits),
        Ins::CALL       => ins_jmplike(&ins, [vec![0xE8], vec![0xFF]], 2, bits),
        Ins::JE |Ins::JZ         
                        => ins_jmplike(&ins, [vec![0x0F, 0x84], vec![]], 0, bits),
        Ins::JNE|Ins::JNZ        
                        => ins_jmplike(&ins, [vec![0xFF, 0x85], vec![]], 0, bits),
        Ins::JL         => ins_jmplike(&ins, [vec![0x0F, 0x8C], vec![]], 0, bits),
        Ins::JLE        => ins_jmplike(&ins, [vec![0x0F, 0x8E], vec![]], 0, bits),
        Ins::JG         => ins_jmplike(&ins, [vec![0x0F, 0x8F], vec![]], 0, bits),
        Ins::JGE        => ins_jmplike(&ins, [vec![0x0F, 0x8D], vec![]], 0, bits),

        Ins::LEA        => ins_lea(&ins, bits),
        //_ => (Vec::new(), None)
    }
}

fn ins_pop(ins: &Instruction, bits: u8) -> Vec<u8>{
    match ins.dst().unwrap() {
        Operand::Reg(r) => gen_base(ins, &[0x58 + r.to_byte()], bits),
        Operand::Mem(_) => vec![0x8F, gen_modrm(&ins, None, Some(0))],
        _ => invalid()
    }
}

fn ins_push(ins: &Instruction, bits: u8) -> Vec<u8>{
    return match ins.dst().unwrap() {
        Operand::Reg(r) => gen_base(ins, &[0x50 + r.to_byte()], bits),
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
        Operand::Mem(_) => gen_ins(&ins, &[0xFF], (true, Some(6), None), None, bits),
        _ => invalid()
    }
}

fn ins_mov(ins: &Instruction, bits: u8) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();
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
                let mut base = gen_base(ins, &[opc], bits);
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
                return gen_ins(&ins, &[opc], (true, None, None), None, bits);
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
                return gen_ins(&ins, &[opc], (true, None, None), None, bits);
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
                return gen_ins(ins, &[opc], (true, Some(0), None), Some(imm), bits);
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
fn add_like_ins(ins: &Instruction, opc: &[u8; 9], ovrreg: u8, bits: u8) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    return match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            if let Size::Dword|Size::Word = srci.size(){
                if let Register::RAX|Register::EAX = dstr{
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[opc[1]], &imm, bits);
                }
                else if let Register::AX = dstr{
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[opc[1]], &imm, bits);
                }
            }
            if let Register::AL = dstr{
                if let Size::Byte = srci.size(){
                    return bs_imm(ins, &[opc[0]], &imm, bits);
                }
            }
            else if let Register::AX = dstr{
                if let Size::Byte = srci.size(){
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[opc[1]], &imm, bits);
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
            let mut base = gen_base(ins, &[opc], bits);
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

            return gen_ins(ins, &[opc], (true, Some(ovrreg), None), Some(imm), bits);
        },
        (Operand::Reg(r), Operand::Mem(_)|Operand::Reg(_)) => {
            let opc = match r.size(){
                Size::Byte => opc[7],
                Size::Word|Size::Dword|Size::Qword => opc[6],
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits)
        },
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size(){
                Size::Byte => opc[7],
                Size::Word|Size::Dword|Size::Qword => opc[6],
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits)
        },
        _ => invalid()
    }
}

fn ins_cmp(ins: &Instruction, bits: u8) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    return match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            if let Size::Dword|Size::Word = srci.size(){
                if let Register::RAX|Register::EAX = dstr{
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[0x3D], &imm, bits);
                }
                else if let Register::AX = dstr{
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0x3D], &imm, bits);
                }
            }
            if let Register::AL = dstr{
                if let Size::Byte = srci.size(){
                    return bs_imm(ins, &[0x3C], &imm, bits);
                }
            }
            else if let Register::AX = dstr{
                if let Size::Byte = srci.size(){
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0x3D], &imm, bits);
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
            let mut base = gen_base(ins, &[opc], bits);
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

            return gen_ins(ins, &[opc], (true, Some(7), None), Some(imm), bits);
        },
        (Operand::Reg(r), Operand::Mem(_)|Operand::Reg(_)) => {
            let opc = match r.size(){
                Size::Byte => 0x3A,
                Size::Word|Size::Dword|Size::Qword => 0x3B,
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits)
        },
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size(){
                Size::Byte => 0x38,
                Size::Word|Size::Dword|Size::Qword => 0x39,
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits)
        },
        _ => invalid()
    }

}

fn ins_test(ins: &Instruction, bits: u8) -> Vec<u8>{
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    return match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            if let Size::Dword|Size::Word = srci.size(){
                if let Register::RAX|Register::EAX = dstr{
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[0xA9], &imm, bits);
                }
                else if let Register::AX = dstr{
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0xA9], &imm, bits);
                }
            }
            if let Register::AL = dstr{
                if let Size::Byte = srci.size(){
                    return bs_imm(ins, &[0xA8], &imm, bits);
                }
            }
            else if let Register::AX = dstr{
                if let Size::Byte = srci.size(){
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0xA9], &imm, bits);
                }
            }

            let opc = match dstr.size(){
                Size::Byte => 0xF6,
                Size::Dword|Size::Qword|Size::Word => 0xF7,
                _ => invalid()
            };
            let mut base = gen_base(ins, &[opc], bits);
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

            return gen_ins(ins, &[opc], (true, Some(0), None), Some(imm), bits);
        },
        (Operand::Reg(_)|Operand::Mem(_), Operand::Reg(_)) => {
            let opc = match dst.size(){
                Size::Byte => 0x84,
                Size::Word|Size::Dword|Size::Qword => 0x85,
                _ => invalid()
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits)
        },
        _ => invalid()
    }

}

fn ins_imul(ins: &Instruction, bits: u8) -> Vec<u8>{
    match ins.src(){
        None => {
            let opc = match ins.dst().unwrap().size(){
                Size::Byte => &[0xF6],
                _ => &[0xF7]
            };
            gen_ins(ins, opc, (true, Some(5), None), None, bits)
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
                    gen_ins(ins, &[opc], (true, dst, src), Some(imm_b), bits)
                },
                _ => {
                    gen_ins(ins, &[0x0F, 0xAF], (true, None, None), None, bits)
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
fn ins_shllike(ins: &Instruction, opc: &[u8; 6], ovr: u8, bits: u8) -> Vec<u8>{
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
    let gen_b = gen_base(&ins, &[opcd], bits);
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

fn ins_inclike(ins: &Instruction, opc: &[u8; 2], ovr: u8, bits: u8) -> Vec<u8> {
    let opc = match ins.dst().unwrap().size(){
        Size::Byte => opc[0],
        _          => opc[1],
    };
    gen_ins(ins, &[opc], (true, Some(ovr), None), None, bits)
}

fn ins_lea(ins: &Instruction, bits: u8) -> (Vec<u8>, Option<Relocation>) {
    let mut base = gen_base(ins, &[0x8D], bits);
    let modrm = if let Operand::Reg(r) = ins.dst().unwrap(){
        0b100 + (r.to_byte() << 3)
    } else {0};
    base.push(modrm);
    base.push(0x25);
    let symbol = match ins.src().unwrap(){
        Operand::SymbolRef(s) => s,
        _ => invalid()
    };
    let blen = base.len();
    base.extend([0x00; 4]);
    (base, Some(Relocation{
        rtype: RType::S32,
        symbol: Cow::Owned(symbol),
        offset: blen as u64,
        addend: 0,
        size: 4,
        catg: RCategory::Lea,
    }))
}

// opc = opcode ONLY for rel32
// why? because i'm too lazy to implement other rel's
//
// opc[0] = rel32
// opc[1] = r/m
fn ins_jmplike(ins: &Instruction, opc: [Vec<u8>; 2], addt: u8, bits: u8) -> (Vec<u8>, Option<Relocation>){
    match ins.dst().unwrap(){
        Operand::SymbolRef(s) => {
            let rel = Relocation{
                rtype: RType::PCRel32,
                symbol: Cow::Owned(s),
                addend: -4,
                offset: opc[0].len() as u64,
                size  : 4,
                catg  : RCategory::Jump,
            };
            let mut opc = opc[0].clone();
            opc.extend([0; 4]);
            return (opc, Some(rel))
        },
        Operand::Reg(_)|Operand::Mem(_) => {
            return (gen_ins(ins, &opc[1], (true, Some(addt), None), None, bits), None)
        }
        _ => invalid()
    }
}

// ==============================
// Utils

fn bs_imm(ins: &Instruction, opc: &[u8], imm: &[u8], bits: u8) -> Vec<u8>{
    let mut b = gen_base(ins, opc, bits);
    b.extend(imm);
    b
}

fn extend_imm(imm: &mut Vec<u8>, size: u8){
    let size = size as usize;
    while imm.len() < size{
        imm.push(0)
    }
}

fn gen_ins(ins: &Instruction, opc: &[u8], modrm: (bool, Option<u8>, Option<u8>), imm: Option<Vec<u8>>, bits: u8) -> Vec<u8> {
    let mut base = gen_base(ins, opc, bits);
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

fn ins_divmul(ins: &Instruction, ovr: u8, bits: u8) -> Vec<u8>{
    let opc = match ins.dst().unwrap().size(){
        Size::Byte => [0xF6],
        _ => [0xF7]
    };
    return gen_ins(ins, &opc, (true, Some(ovr), None), None, bits);
}

fn gen_base(ins: &Instruction, opc: &[u8], bits: u8) -> Vec<u8>{
    // how does this even work? (probably doesn't)
    let (rex_bool, rex) = 
    if bits == 64 {
        if let Some(rex) = gen_rex(ins){
            (rex & 0x08 == 8, Some(rex))
        } else {
            (ins.size() == Size::Qword, None)
        }
    } else {
        (false, None)
    };

    let mut used_66 = false;
    let mut size_ovr = if let Some(dst) = ins.dst(){
        if let Some(s) = gen_size_ovr(ins, dst, bits, rex_bool){
            used_66 = s == 0x66;
            vec![s]
        } else {Vec::new()}
    } else {Vec::new()};

    if let Some(src) = ins.src(){
        if let Some(s) = gen_size_ovr(ins, src, bits, rex_bool){
            if !used_66 && !s == 0x66{
                size_ovr.push(s);
            }
        }
    }
    let mut base = size_ovr;
    
    if let Some(rex) = rex{
        base.push(rex);
    }

    base.extend(opc);
    base
}

fn gen_size_ovr(ins: &Instruction, op: &Operand, bits: u8, rexw: bool) -> Option<u8>{
    let (size, is_mem) = match op{
        Operand::Reg(r) => (r.size(), false),
        Operand::Mem(m) => (m.size(), false),
        Operand::Segment(s) => (s.address.x86_addr_size(), true),
        _ => return None
    };
    if size == Size::Byte {
        return None;
    }
    match bits{
        16 => {
            match (size, is_mem){
                (Size::Word, _) => return None,
                (Size::Dword, true)  => return Some(0x67),
                (Size::Dword, false) => return Some(0x66),
                _ => inv_osop(&format!("{:?}", op)),
            }
        }
        32 => {
            match (size, is_mem){
                (Size::Word, false)  => return Some(0x66),
                (Size::Word, true )  => return Some(0x67),
                (Size::Dword, _)  => return None,
                _ => inv_osop(&format!("{:?}", op)),
            }
        },
        64 => {
            match (size, is_mem){
                (Size::Qword, false) => if ins.mnem.defaults_to_64bit() || rexw{
                    return None;
                } else { return Some(0x66) },
                (Size::Dword, false)|(Size::Qword, true) => return None,
                (Size::Word, false) => return Some(0x66),
                (Size::Word,  true) => return Some(0x67),
                (Size::Dword, true) => return Some(0x67),
                _ => inv_osop(&format!("{:?}", op)),
            }
        },
        _ => invalid()
    }
}

fn inv_osop(s: &str) -> !{
    rpanic("comp.rs", "gen_size_ovr", 1, s)
}

fn invalid() -> !{
    rpanic("comp.rs", "some function", 1, "Unexpected thing that should not happen")
}
