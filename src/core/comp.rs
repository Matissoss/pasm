// rasmx86_64 - src/core/comp.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use std::borrow::Cow;

use crate::{
    core::{api::*, avx, disp, modrm, rex, sib, sse2, vex},
    shr::{
        ast::{IVariant, Instruction, Label, Operand},
        ins::Mnemonic as Ins,
        num::Number,
        reg::{Purpose as RPurpose, Register},
        reloc::{RCategory, RType, Relocation},
        size::Size,
        symbol::{Symbol, SymbolType, Visibility},
        var::{VarContent, Variable},
    },
};
use OpOrd::*;

#[inline]
pub fn make_globals(symbols: &mut [Symbol], globals: &[String]) {
    for s in symbols {
        for g in globals {
            if s.name == Cow::Borrowed(g) {
                s.visibility = Visibility::Global;
                break;
            }
        }
    }
}
#[inline]
pub fn extern_trf(externs: &Vec<String>) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    for extern_ in externs {
        symbols.push(Symbol {
            name: Cow::Borrowed(extern_),
            offset: 0,
            size: None,
            sindex: 0,
            stype: SymbolType::NoType,
            visibility: Visibility::Global,
            content: None,
            addend: 0,
            addt: 0,
        });
    }
    symbols
}

pub fn compile_section<'a>(
    vars: &'a Vec<&'a Variable<'a>>,
    sindex: u16,
    addt: u8,
) -> (Vec<u8>, Vec<Symbol<'a>>) {
    let mut buf: Vec<u8> = Vec::new();
    let mut symbols: Vec<Symbol> = Vec::new();

    let mut offset: u64 = 0;

    for v in vars {
        match v.content {
            VarContent::Uninit => {
                symbols.push(Symbol {
                    name: Cow::Borrowed(&v.name),
                    size: Some(v.size),
                    sindex,
                    stype: SymbolType::Object,
                    offset,
                    content: None,
                    visibility: v.visibility,
                    addend: 0,
                    addt,
                });
                offset += v.size as u64;
            }
            _ => {
                buf.extend(v.content.bytes());
                symbols.push(Symbol {
                    name: Cow::Borrowed(&v.name),
                    size: Some(v.size),
                    sindex,
                    stype: SymbolType::Object,
                    offset,
                    content: Some(Cow::Borrowed(&v.content)),
                    visibility: v.visibility,
                    addend: 0,
                    addt,
                });
                offset += v.size as u64;
            }
        }
    }
    (buf, symbols)
}

pub fn compile_label<'a>(lbl: &'a Label, offset: usize) -> (Vec<u8>, Vec<Relocation<'a>>) {
    let mut bytes = Vec::new();
    let mut reallocs = Vec::new();
    let bits = lbl.bits;
    for ins in &lbl.inst {
        let res = compile_instruction(ins, bits);
        // we do not want situation, where label is entry and we place padding before it -
        // preventing UB
        if offset != 0 {
            let align = if lbl.align == 0 {
                1
            } else {
                lbl.align as usize
            };
            let mut padding = align - (offset % align);
            while padding > 0 {
                bytes.push(0x0);
                padding -= 1;
            }
        }
        if let Some(mut rl) = res.1 {
            rl.offset += bytes.len() as u64;
            reallocs.push(rl);
        }
        bytes.extend(res.0);
    }
    (bytes, reallocs)
}

pub fn compile_instruction(ins: &'_ Instruction, bits: u8) -> (Vec<u8>, Option<Relocation<'_>>) {
    match ins.mnem {
        Ins::__LAST => (vec![], None),
        Ins::CPUID => (vec![0x0F, 0xA2], None),
        Ins::RET => (vec![0xC3], None),
        Ins::SYSCALL => (vec![0x0F, 0x05], None),
        Ins::PUSH => (ins_push(ins, bits), None),
        Ins::POP => (ins_pop(ins, bits), None),
        Ins::MOV => (ins_mov(ins, bits), None),
        Ins::ADD => (
            add_like_ins(
                ins,
                &[0x04, 0x05, 0x80, 0x81, 0x83, 0x00, 0x01, 0x02, 0x03],
                0,
                bits,
            ),
            None,
        ),
        Ins::OR => (
            add_like_ins(
                ins,
                &[0x0C, 0x0D, 0x80, 0x81, 0x83, 0x08, 0x09, 0x0A, 0x0B],
                1,
                bits,
            ),
            None,
        ),
        Ins::AND => (
            add_like_ins(
                ins,
                &[0x24, 0x25, 0x80, 0x81, 0x83, 0x20, 0x21, 0x22, 0x23],
                4,
                bits,
            ),
            None,
        ),
        Ins::SUB => (
            add_like_ins(
                ins,
                &[0x2C, 0x2D, 0x80, 0x81, 0x83, 0x28, 0x29, 0x2A, 0x2B],
                5,
                bits,
            ),
            None,
        ),
        Ins::XOR => (
            add_like_ins(
                ins,
                &[0x34, 0x35, 0x80, 0x81, 0x83, 0x30, 0x31, 0x32, 0x33],
                6,
                bits,
            ),
            None,
        ),
        Ins::SAL | Ins::SHL => (
            ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 4, bits),
            None,
        ),
        Ins::SHR => (
            ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 5, bits),
            None,
        ),
        Ins::SAR => (
            ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 7, bits),
            None,
        ),
        Ins::TEST => (ins_test(ins, bits), None),
        Ins::INC => (ins_inclike(ins, &[0xFE, 0xFF], 0, bits), None),
        Ins::DEC => (ins_inclike(ins, &[0xFE, 0xFF], 1, bits), None),
        Ins::NOT => (ins_inclike(ins, &[0xF6, 0xF7], 2, bits), None),
        Ins::NEG => (ins_inclike(ins, &[0xF6, 0xF7], 3, bits), None),
        Ins::CMP => (ins_cmp(ins, bits), None),
        Ins::IMUL => (ins_imul(ins, bits), None),
        Ins::DIV => (ins_divmul(ins, 6, bits), None),
        Ins::IDIV => (ins_divmul(ins, 7, bits), None),
        Ins::MUL => (ins_divmul(ins, 4, bits), None),
        Ins::JMP => ins_jmplike(ins, [vec![0xE9], vec![0xFF]], 4, bits),
        Ins::CALL => ins_jmplike(ins, [vec![0xE8], vec![0xFF]], 2, bits),

        // jcc
        Ins::JA => ins_jmplike(ins, [vec![0x0F, 0x87], vec![]], 0, bits),
        Ins::JB => ins_jmplike(ins, [vec![0x0F, 0x82], vec![]], 0, bits),
        Ins::JC => ins_jmplike(ins, [vec![0x0F, 0x82], vec![]], 0, bits),
        Ins::JO => ins_jmplike(ins, [vec![0x0F, 0x80], vec![]], 0, bits),
        Ins::JP => ins_jmplike(ins, [vec![0x0F, 0x8A], vec![]], 0, bits),
        Ins::JS => ins_jmplike(ins, [vec![0x0F, 0x88], vec![]], 0, bits),
        Ins::JL => ins_jmplike(ins, [vec![0x0F, 0x8C], vec![]], 0, bits),
        Ins::JG => ins_jmplike(ins, [vec![0x0F, 0x8F], vec![]], 0, bits),
        Ins::JE | Ins::JZ => ins_jmplike(ins, [vec![0x0F, 0x84], vec![]], 0, bits),
        Ins::JAE => ins_jmplike(ins, [vec![0x0F, 0x83], vec![]], 0, bits),
        Ins::JBE => ins_jmplike(ins, [vec![0x0F, 0x86], vec![]], 0, bits),
        Ins::JNA => ins_jmplike(ins, [vec![0x0F, 0x86], vec![]], 0, bits),
        Ins::JNB => ins_jmplike(ins, [vec![0x0F, 0x83], vec![]], 0, bits),
        Ins::JNC => ins_jmplike(ins, [vec![0x0F, 0x83], vec![]], 0, bits),
        Ins::JNG => ins_jmplike(ins, [vec![0x0F, 0x8E], vec![]], 0, bits),
        Ins::JNL => ins_jmplike(ins, [vec![0x0F, 0x8D], vec![]], 0, bits),
        Ins::JNO => ins_jmplike(ins, [vec![0x0F, 0x81], vec![]], 0, bits),
        Ins::JNP => ins_jmplike(ins, [vec![0x0F, 0x8B], vec![]], 0, bits),
        Ins::JNS => ins_jmplike(ins, [vec![0x0F, 0x89], vec![]], 0, bits),
        Ins::JPE => ins_jmplike(ins, [vec![0x0F, 0x8A], vec![]], 0, bits),
        Ins::JPO => ins_jmplike(ins, [vec![0x0F, 0x8B], vec![]], 0, bits),
        Ins::JNE | Ins::JNZ => ins_jmplike(ins, [vec![0xFF, 0x85], vec![]], 0, bits),
        Ins::JLE => ins_jmplike(ins, [vec![0x0F, 0x8E], vec![]], 0, bits),
        Ins::JGE => ins_jmplike(ins, [vec![0x0F, 0x8D], vec![]], 0, bits),
        Ins::JNAE => ins_jmplike(ins, [vec![0x0F, 0x82], vec![]], 0, bits),
        Ins::JNBE => ins_jmplike(ins, [vec![0x0F, 0x87], vec![]], 0, bits),
        Ins::JNGE => ins_jmplike(ins, [vec![0x0F, 0x8C], vec![]], 0, bits),
        Ins::JNLE => ins_jmplike(ins, [vec![0x0F, 0x8F], vec![]], 0, bits),

        Ins::LEA => ins_lea(ins, bits),

        Ins::NOP => (vec![0x90], None),

        Ins::PUSHF | Ins::PUSHFD | Ins::PUSHFQ => (vec![0x9C], None),
        Ins::POPF | Ins::POPFD | Ins::POPFQ => (vec![0x9D], None),

        Ins::CLFLUSH => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(7), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::PAUSE => (vec![0xF3, 0x90], None),
        Ins::MWAIT => (vec![0x0F, 0x01, 0xC9], None),

        Ins::CMOVA => (ins_cmovcc(ins, &[0x0F, 0x47], bits), None),
        Ins::CMOVAE => (ins_cmovcc(ins, &[0x0F, 0x43], bits), None),
        Ins::CMOVB => (ins_cmovcc(ins, &[0x0F, 0x42], bits), None),
        Ins::CMOVBE => (ins_cmovcc(ins, &[0x0F, 0x46], bits), None),
        Ins::CMOVC => (ins_cmovcc(ins, &[0x0F, 0x42], bits), None),
        Ins::CMOVE => (ins_cmovcc(ins, &[0x0F, 0x44], bits), None),
        Ins::CMOVG => (ins_cmovcc(ins, &[0x0F, 0x4F], bits), None),
        Ins::CMOVGE => (ins_cmovcc(ins, &[0x0F, 0x4D], bits), None),
        Ins::CMOVL => (ins_cmovcc(ins, &[0x0F, 0x4C], bits), None),
        Ins::CMOVLE => (ins_cmovcc(ins, &[0x0F, 0x4E], bits), None),
        Ins::CMOVNA => (ins_cmovcc(ins, &[0x0F, 0x46], bits), None),
        Ins::CMOVNB => (ins_cmovcc(ins, &[0x0F, 0x43], bits), None),
        Ins::CMOVNBE => (ins_cmovcc(ins, &[0x0F, 0x47], bits), None),
        Ins::CMOVNC => (ins_cmovcc(ins, &[0x0F, 0x43], bits), None),
        Ins::CMOVNE => (ins_cmovcc(ins, &[0x0F, 0x45], bits), None),
        Ins::CMOVNG => (ins_cmovcc(ins, &[0x0F, 0x4E], bits), None),
        Ins::CMOVNGE => (ins_cmovcc(ins, &[0x0F, 0x4C], bits), None),
        Ins::CMOVNL => (ins_cmovcc(ins, &[0x0F, 0x4D], bits), None),
        Ins::CMOVNLE => (ins_cmovcc(ins, &[0x0F, 0x4F], bits), None),
        Ins::CMOVNAE => (ins_cmovcc(ins, &[0x0F, 0x42], bits), None),
        Ins::CMOVNO => (ins_cmovcc(ins, &[0x0F, 0x41], bits), None),
        Ins::CMOVNP => (ins_cmovcc(ins, &[0x0F, 0x4B], bits), None),
        Ins::CMOVNS => (ins_cmovcc(ins, &[0x0F, 0x49], bits), None),
        Ins::CMOVNZ => (ins_cmovcc(ins, &[0x0F, 0x45], bits), None),
        Ins::CMOVO => (ins_cmovcc(ins, &[0x0F, 0x40], bits), None),
        Ins::CMOVP => (ins_cmovcc(ins, &[0x0F, 0x4A], bits), None),
        Ins::CMOVPO => (ins_cmovcc(ins, &[0x0F, 0x4B], bits), None),
        Ins::CMOVS => (ins_cmovcc(ins, &[0x0F, 0x48], bits), None),
        Ins::CMOVZ => (ins_cmovcc(ins, &[0x0F, 0x44], bits), None),
        Ins::CMOVPE => (ins_cmovcc(ins, &[0x0F, 0x4A], bits), None),

        // SSE
        Ins::MOVSS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true).prefix(0xF3);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVHLPS => {
            let api = GenAPI::new()
                .modrm(true, None, None)
                .rex(true)
                .opcode(&[0x0F, 0x12])
                .ord(&[MODRM_REG, MODRM_RM]);
            (api.assemble(ins, bits), None)
        }
        Ins::MOVLHPS => {
            let api = GenAPI::new()
                .modrm(true, None, None)
                .rex(true)
                .opcode(&[0x0F, 0x16])
                .ord(&[MODRM_REG, MODRM_RM]);
            (api.assemble(ins, bits), None)
        }
        Ins::MOVAPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x29]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVUPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVLPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x13]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x12]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVHPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x17]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x16]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::ADDPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x58])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ADDSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x58])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::SUBPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x5C])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::SUBSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x5C])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MULPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x59])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MULSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x59])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::DIVPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x5E])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::DIVSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x5E])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MINPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x5D])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MINSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x5D])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MAXPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x5F])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MAXSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x5F])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::RSQRTPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x52])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::RSQRTSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x52])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::SHUFPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0xC6])
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::SQRTPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x51])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::SQRTSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x51])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::CMPPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0xC2])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::CMPSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0xC2])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::RCPPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x53])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::RCPSS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x53])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::UCOMISS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x2E])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::COMISS => (
            GenAPI::new()
                .modrm(true, None, None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0x2F])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ORPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x56])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ANDPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x54])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ANDNPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x55])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::XORPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x57])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::UNPCKLPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x14])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::UNPCKHPS => (
            GenAPI::new()
                .modrm(true, None, None)
                .opcode(&[0x0F, 0x15])
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        // SSE2
        Ins::MOVNTI => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC3])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::MFENCE => (vec![0xF0, 0xAE, 0xF0], None),
        Ins::LFENCE => (vec![0xF0, 0xAE, 0xE8], None),

        Ins::MOVNTPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2B])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVNTDQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0xE7])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVAPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x29]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVUPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVLPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x13]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x12]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVHPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x17]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x16]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVSD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0xF2).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVDQA => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x7F]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x6F]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MOVDQ2Q => (
            GenAPI::new()
                .opcode(&[0x0F, 0xD6])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVQ2DQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0xD6])
                .prefix(0xF3)
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::MOVMSKPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x50])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::ADDPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x58])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ADDSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x58])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::SUBPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5C])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::SUBSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5C])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MULPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x59])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MULSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x59])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::DIVPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5E])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::DIVSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5E])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MINPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5D])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MINSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5D])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MAXPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5F])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MAXSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5F])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::SQRTPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x51])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::SQRTSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x51])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::CMPPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC2])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::CMPSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC2])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::COMISD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2F])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::UCOMISD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2E])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ORPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x56])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ANDPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x54])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ANDNPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x55])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::XORPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x57])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PSHUFLW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x70])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PSHUFHW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x70])
                .prefix(0xF3)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PSHUFD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x70])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::PSLLDQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x73])
                .prefix(0x66)
                .modrm(true, Some(7), None)
                .rex(true)
                .imm_atindex(1, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::PSRLDQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x73])
                .prefix(0x66)
                .modrm(true, Some(3), None)
                .rex(true)
                .imm_atindex(1, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::PUNPCKHQDQ => {
            let api = GenAPI::new()
                .opcode(&[0x0F, 0x6D])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .prefix(0x66)
                .rex(true);
            (api.assemble(ins, bits), None)
        }
        Ins::PUNPCKLQDQ => {
            let api = GenAPI::new()
                .opcode(&[0x0F, 0x6C])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .prefix(0x66)
                .rex(true);
            (api.assemble(ins, bits), None)
        }
        // MMX/SSE2
        Ins::MOVD | Ins::MOVQ => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66);
            }
            if let Some(Operand::Reg(r)) = ins.dst() {
                if r.size() == Size::Xword || r.purpose() == RPurpose::Mmx {
                    api = api.opcode(&[0x0F, 0x6E]);
                } else {
                    api = api.opcode(&[0x0F, 0x7E]);
                }
            } else {
                api = api.opcode(&[0x0F, 0x6E]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PADDB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PADDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFD])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PADDD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFE])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PADDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD4])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PADDUSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PADDUSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDD])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PADDSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PADDSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xED])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSUBUSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD8])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSUBUSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PSUBB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF8])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSUBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSUBD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFA])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSUBQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::MASKMOVDQU => (
            GenAPI::new()
                .opcode(&[0x0F, 0xF7])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::PSUBSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE8])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PMULLW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD5])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PMULHW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE5])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PMULUDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF4])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PMADDWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF5])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PCMPEQB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x74])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PCMPEQW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x75])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PCMPEQD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x76])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PCMPGTB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x64])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PCMPGTW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x65])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PCMPGTD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x66])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PACKUSWB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x67])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PACKSSWB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x63])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PACKSSDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x6B])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PUNPCKLBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x60])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PUNPCKLWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x61])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PUNPCKLDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x62])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PUNPCKHBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x68])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PUNPCKHWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x69])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PUNPCKHDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x6A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PSLLQ => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x73])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(6), None);
            } else {
                api = api
                    .opcode(&[0x0F, 0xF3])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSLLD => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x72])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(6), None);
            } else {
                api = api
                    .opcode(&[0x0F, 0xF2])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSLLW => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x71])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(6), None);
            } else {
                api = api
                    .opcode(&[0x0F, 0xF1])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSRLW => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x71])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(2), None);
            } else {
                api = api
                    .opcode(&[0x0F, 0xD1])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSRLD => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x72])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(2), None);
            } else {
                api = api
                    .opcode(&[0x0F, 0xD2])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSRLQ => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x73])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(2), None);
            } else {
                api = api
                    .opcode(&[0x0F, 0xD3])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSRAW => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x71])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(4), None);
            } else {
                api = api
                    .opcode(&[0x0F, 0xE1])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSRAD => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x72])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(4), None);
            } else {
                api = api
                    .opcode(&[0x0F, 0xE2])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::POR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PAND => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PANDN => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDF])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PXOR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEF])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::EMMS => (vec![0x0F, 0x77], None),

        // sse3
        Ins::ADDSUBPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0xD0])
                .prefix(0x66)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::ADDSUBPS => (
            GenAPI::new()
                .opcode(&[0x0F, 0xD0])
                .prefix(0xF2)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::HADDPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x7C])
                .prefix(0x66)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::HADDPS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x7C])
                .prefix(0xF2)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::HSUBPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x7D])
                .prefix(0x66)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::HSUBPS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x7D])
                .prefix(0xF2)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::MOVSLDUP => (
            GenAPI::new()
                .opcode(&[0x0F, 0x12])
                .modrm(true, None, None)
                .prefix(0xF3)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVSHDUP => (
            GenAPI::new()
                .opcode(&[0x0F, 0x16])
                .modrm(true, None, None)
                .prefix(0xF3)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVDDUP => (
            GenAPI::new()
                .opcode(&[0x0F, 0x12])
                .modrm(true, None, None)
                .prefix(0xF2)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::LDDQU => (
            GenAPI::new()
                .opcode(&[0x0F, 0xF0])
                .modrm(true, None, None)
                .prefix(0xF2)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::MONITOR => (vec![0x0F, 0x01, 0xC8], None),

        // ssse3
        Ins::PABSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1C])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PABSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1D])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PABSD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1E])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PSIGNB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x08])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSIGNW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x09])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PSIGND => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x0A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }

        Ins::PSHUFB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x00])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PHADDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x01])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PHADDD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x02])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PHADDSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x03])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PHSUBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x05])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PHSUBD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x06])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PHSUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x07])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PALIGNR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x0F])
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PMULHRSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x0B])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PMADDUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x04])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        // sse4
        Ins::DPPS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x40])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::DPPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x41])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PTEST => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x17])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PEXTRW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x15])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::PEXTRB => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x14])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::PEXTRD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x16])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::PEXTRQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x16])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::PINSRB => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x20])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PINSRD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x22])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PINSRQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x22])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PMAXSB => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x3C])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PMAXSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x3D])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PMAXUW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x3E])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PMINSB => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x38])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PMINSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x39])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PMINUW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x3A])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PMULDQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x28])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PMULLD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x40])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::BLENDPS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x0C])
                .prefix(0x66)
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::BLENDPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x0D])
                .prefix(0x66)
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PBLENDW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x0E])
                .prefix(0x66)
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PCMPEQQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x29])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ROUNDPS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x08])
                .prefix(0x66)
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ROUNDPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x09])
                .prefix(0x66)
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ROUNDSS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x0A])
                .prefix(0x66)
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ROUNDSD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x0B])
                .prefix(0x66)
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MPSADBW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x42])
                .prefix(0x66)
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PCMPGTQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x37])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::BLENDVPS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x14])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::BLENDVPD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x15])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PBLENDVB => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x10])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::INSERTPS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x21])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PACKUSDW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x2B])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVNTDQA => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x2A])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PCMPESTRM => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x60])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PCMPESTRI => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x61])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PCMPISTRM => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x62])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PCMPISTRI => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x63])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::EXTRACTPS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x17])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PHMINPOSUW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x41])
                .prefix(0x66)
                .modrm(true, None, None)
                .rex(true)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::CRC32 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xF0])
                .prefix(0xF2)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::POPCNT => (
            GenAPI::new()
                .opcode(&[0x0F, 0xB8])
                .prefix(0xF3)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        // AVX
        Ins::VMOVDQA => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x7F]);
            } else {
                api = api.opcode(&[0x6F]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVSLDUP => (
            GenAPI::new()
                .opcode(&[0x12])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VLDDQU => (
            GenAPI::new()
                .opcode(&[0xF0])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMOVDDUP => (
            GenAPI::new()
                .opcode(&[0x12])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMOVSHDUP => (
            GenAPI::new()
                .opcode(&[0x16])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMOVMSKPD => (
            GenAPI::new()
                .opcode(&[0x50])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMOVAPS => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x29]);
            } else {
                api = api.opcode(&[0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVAPD => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x29]);
            } else {
                api = api.opcode(&[0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVUPS => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x11]);
            } else {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVUPD => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x11]);
            } else {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VADDPS => (
            GenAPI::new()
                .opcode(&[0x58])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VADDSUBPS => (
            GenAPI::new()
                .opcode(&[0xD0])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VADDSUBPD => (
            GenAPI::new()
                .opcode(&[0xD0])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VHADDPS => (
            GenAPI::new()
                .opcode(&[0x7C])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VHADDPD => (
            GenAPI::new()
                .opcode(&[0x7C])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VHSUBPS => (
            GenAPI::new()
                .opcode(&[0x7D])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VHSUBPD => (
            GenAPI::new()
                .opcode(&[0x7D])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VADDPD => (
            GenAPI::new()
                .opcode(&[0x58])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VADDSS => (
            GenAPI::new()
                .opcode(&[0x58])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VADDSD => (
            GenAPI::new()
                .opcode(&[0x58])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VSUBPS => (
            GenAPI::new()
                .opcode(&[0x5C])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VSUBPD => (
            GenAPI::new()
                .opcode(&[0x5C])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VSUBSS => (
            GenAPI::new()
                .opcode(&[0x5C])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VSUBSD => (
            GenAPI::new()
                .opcode(&[0x5C])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VMULPS => (
            GenAPI::new()
                .opcode(&[0x59])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMULPD => (
            GenAPI::new()
                .opcode(&[0x59])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMULSS => (
            GenAPI::new()
                .opcode(&[0x59])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMULSD => (
            GenAPI::new()
                .opcode(&[0x59])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VDIVPS => (
            GenAPI::new()
                .opcode(&[0x5E])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VDIVPD => (
            GenAPI::new()
                .opcode(&[0x5E])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VDIVSS => (
            GenAPI::new()
                .opcode(&[0x5E])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VDIVSD => (
            GenAPI::new()
                .opcode(&[0x5E])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VRCPPS => (
            GenAPI::new()
                .opcode(&[0x53])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VRCPSS => (
            GenAPI::new()
                .opcode(&[0x53])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VSQRTPS => (
            GenAPI::new()
                .opcode(&[0x51])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VSQRTPD => (
            GenAPI::new()
                .opcode(&[0x51])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VSQRTSS => (
            GenAPI::new()
                .opcode(&[0x51])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VSQRTSD => (
            GenAPI::new()
                .opcode(&[0x51])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VRSQRTPS => (
            GenAPI::new()
                .opcode(&[0x52])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VRSQRTSS => (
            GenAPI::new()
                .opcode(&[0x52])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMULDQ => (
            GenAPI::new()
                .opcode(&[0x28])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMULLD => (
            GenAPI::new()
                .opcode(&[0x40])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMINSB => (
            GenAPI::new()
                .opcode(&[0x38])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMINSD => (
            GenAPI::new()
                .opcode(&[0x39])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMINUB => (
            GenAPI::new()
                .opcode(&[0xDA])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMINUW => (
            GenAPI::new()
                .opcode(&[0x3A])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMAXSB => (
            GenAPI::new()
                .opcode(&[0x3C])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMAXSD => (
            GenAPI::new()
                .opcode(&[0x3D])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMAXUB => (
            GenAPI::new()
                .opcode(&[0xDE])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMAXUW => (
            GenAPI::new()
                .opcode(&[0x3E])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VMINPS => (
            GenAPI::new()
                .opcode(&[0x5D])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMINPD => (
            GenAPI::new()
                .opcode(&[0x5D])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMINSS => (
            GenAPI::new()
                .opcode(&[0x5D])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMINSD => (
            GenAPI::new()
                .opcode(&[0x5D])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMAXPS => (
            GenAPI::new()
                .opcode(&[0x5F])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMAXPD => (
            GenAPI::new()
                .opcode(&[0x5F])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMAXSS => (
            GenAPI::new()
                .opcode(&[0x5F])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMAXSD => (
            GenAPI::new()
                .opcode(&[0x5F])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VORPS => (
            GenAPI::new()
                .opcode(&[0x56])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VORPD => (
            GenAPI::new()
                .opcode(&[0x56])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VANDPS => (
            GenAPI::new()
                .opcode(&[0x54])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VANDPD => (
            GenAPI::new()
                .opcode(&[0x54])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VANDNPD => (
            GenAPI::new()
                .opcode(&[0x55])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VXORPD => (
            GenAPI::new()
                .opcode(&[0x57])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VBLENDVPS => (
            GenAPI::new()
                .opcode(&[0x4A])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPBLENDVB => (
            GenAPI::new()
                .opcode(&[0x4C])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VBLENDVPD => (
            GenAPI::new()
                .opcode(&[0x4B])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),

        Ins::VPHMINPOSUW => (
            GenAPI::new()
                .opcode(&[0x41])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VEXTRACTPS => (
            GenAPI::new()
                .opcode(&[0x17])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_RM, MODRM_REG])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),

        Ins::VMOVNTDQA => (
            GenAPI::new()
                .opcode(&[0x2A])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPACKUSDW => (
            GenAPI::new()
                .opcode(&[0x2B])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPESTRM => (
            GenAPI::new()
                .opcode(&[0x60])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPESTRI => (
            GenAPI::new()
                .opcode(&[0x61])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPISTRM => (
            GenAPI::new()
                .opcode(&[0x62])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPISTRI => (
            GenAPI::new()
                .opcode(&[0x63])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VINSERTPS => (
            GenAPI::new()
                .opcode(&[0x21])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VBLENDPS => (
            GenAPI::new()
                .opcode(&[0x0C])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VBLENDPD => (
            GenAPI::new()
                .opcode(&[0x0D])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPGTQ => (
            GenAPI::new()
                .opcode(&[0x37])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPEQQ => (
            GenAPI::new()
                .opcode(&[0x29])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMPSADBW => (
            GenAPI::new()
                .opcode(&[0x42])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VROUNDSS => (
            GenAPI::new()
                .opcode(&[0x0A])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VROUNDSD => (
            GenAPI::new()
                .opcode(&[0x0B])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VROUNDPS => (
            GenAPI::new()
                .opcode(&[0x08])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VROUNDPD => (
            GenAPI::new()
                .opcode(&[0x09])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPBLENDW => (
            GenAPI::new()
                .opcode(&[0x0E])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VCMPPD => (
            GenAPI::new()
                .opcode(&[0xC2])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VANDNPS => (
            GenAPI::new()
                .opcode(&[0x55])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VXORPS => (
            GenAPI::new()
                .opcode(&[0x57])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPTEST => (
            GenAPI::new()
                .opcode(&[0x17])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VDPPS => (
            GenAPI::new()
                .opcode(&[0x40])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VDPPD => (
            GenAPI::new()
                .opcode(&[0x41])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VCMPPS => (
            GenAPI::new()
                .opcode(&[0xC2])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VCMPSS => (
            GenAPI::new()
                .opcode(&[0xC2])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VCMPSD => (
            GenAPI::new()
                .opcode(&[0xC2])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VUCOMISS => (
            GenAPI::new()
                .opcode(&[0x2E])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VUCOMISD => (
            GenAPI::new()
                .opcode(&[0x2E])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCOMISS => (
            GenAPI::new()
                .opcode(&[0x2F])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCOMISD => (
            GenAPI::new()
                .opcode(&[0x2F])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VUNPCKLPS => (
            GenAPI::new()
                .opcode(&[0x14])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VUNPCKHPS => (
            GenAPI::new()
                .opcode(&[0x15])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VSHUFPS => (
            GenAPI::new()
                .opcode(&[0xC6])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VMOVSS => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0xF3).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x11]);
            } else {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVSD => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0xF2).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x11]);
            } else {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVLPS => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x13]);
            } else {
                api = api.opcode(&[0x12]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVLPD => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x13]);
            } else {
                api = api.opcode(&[0x12]).ord(&[MODRM_RM, VEX_VVVV, MODRM_REG]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVHPS => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x17]);
            } else {
                api = api.opcode(&[0x16]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVHPD => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x17]);
            } else {
                api = api.opcode(&[0x16]).ord(&[MODRM_RM, VEX_VVVV, MODRM_REG]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VMOVLHPS => (
            GenAPI::new()
                .opcode(&[0x16])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMOVHLPS => (
            GenAPI::new()
                .opcode(&[0x12])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPEXTRB => (
            GenAPI::new()
                .opcode(&[0x14])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_RM, MODRM_REG])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPEXTRW => (
            GenAPI::new()
                .opcode(&[0xC5])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_RM, MODRM_REG])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPEXTRD => (
            GenAPI::new()
                .opcode(&[0x16])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_RM, MODRM_REG])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPEXTRQ => (
            GenAPI::new()
                .opcode(&[0x16])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(true))
                .modrm(true, None, None)
                .ord(&[MODRM_RM, MODRM_REG])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPINSRB => (
            GenAPI::new()
                .opcode(&[0x20])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPINSRD => (
            GenAPI::new()
                .opcode(&[0x22])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::VPINSRQ => (
            GenAPI::new()
                .opcode(&[0x22])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(true))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .imm_atindex(3, 1)
                .assemble(ins, bits),
            None,
        ),

        // MMX derived part 1
        Ins::VPOR => (
            avx::avx_ins(ins, &[0xEB], &[0xEB], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VMOVD => (
            avx::avx_ins_movx(ins, &[0x7E], &[0x6E], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VMOVQ => (
            avx::avx_ins_movx(ins, &[0x7E], &[0x6E], None, 0x66, 0x0F, true),
            None,
        ),
        Ins::VPAND => (
            avx::avx_ins(ins, &[0xDB], &[0xDB], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPXOR => (
            avx::avx_ins(ins, &[0xEF], &[0xEF], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPADDB => (
            avx::avx_ins(ins, &[0xFC], &[0xFC], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPADDW => (
            avx::avx_ins(ins, &[0xFD], &[0xFD], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPADDD => (
            avx::avx_ins(ins, &[0xFE], &[0xFE], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPADDQ => (
            avx::avx_ins(ins, &[0xD4], &[0xD4], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSUBB => (
            avx::avx_ins(ins, &[0xF8], &[0xF8], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSUBW => (
            avx::avx_ins(ins, &[0xF9], &[0xF9], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSUBD => (
            avx::avx_ins(ins, &[0xFA], &[0xFA], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSUBQ => (
            avx::avx_ins(ins, &[0xFB], &[0xFB], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPANDN => (
            avx::avx_ins(ins, &[0xDF], &[0xDF], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSLLW => (
            avx::avx_ins_shift(ins, &[0xF1], &[0x71], Some(6), 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSLLD => (
            avx::avx_ins_shift(ins, &[0xF2], &[0x72], Some(6), 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSLLQ => (
            avx::avx_ins_shift(ins, &[0xF3], &[0x73], Some(6), 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSRLW => (
            avx::avx_ins_shift(ins, &[0xD1], &[0x71], Some(2), 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSRLD => (
            avx::avx_ins_shift(ins, &[0xD2], &[0x72], Some(2), 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSRLQ => (
            avx::avx_ins_shift(ins, &[0xD3], &[0x73], Some(2), 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSRAW => (
            avx::avx_ins_shift(ins, &[0xE1], &[0x71], Some(4), 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSRAD => (
            avx::avx_ins_shift(ins, &[0xE2], &[0x72], Some(4), 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSUBSB => (
            avx::avx_ins(ins, &[0xE8], &[0xE8], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSUBSW => (
            avx::avx_ins(ins, &[0xE9], &[0xE9], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPADDSB => (
            avx::avx_ins(ins, &[0xEC], &[0xEC], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPADDSW => (
            avx::avx_ins(ins, &[0xED], &[0xED], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPMULHW => (
            avx::avx_ins(ins, &[0xE5], &[0xE5], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPMULLW => (
            avx::avx_ins(ins, &[0xD5], &[0xD5], None, 0x66, 0x0F, false),
            None,
        ),
        // part 2
        Ins::VPADDUSB => (
            avx::avx_ins(ins, &[0xDC], &[0xDC], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPADDUSW => (
            avx::avx_ins(ins, &[0xDD], &[0xDD], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSUBUSB => (
            avx::avx_ins(ins, &[0xD8], &[0xD8], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPSUBUSW => (
            avx::avx_ins(ins, &[0xD9], &[0xD9], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPMADDWD => (
            avx::avx_ins(ins, &[0xF5], &[0xF5], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPCMPEQB => (
            avx::avx_ins(ins, &[0x74], &[0x74], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPCMPEQW => (
            avx::avx_ins(ins, &[0x75], &[0x75], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPCMPEQD => (
            avx::avx_ins(ins, &[0x76], &[0x76], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPCMPGTB => (
            avx::avx_ins(ins, &[0x64], &[0x64], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPCMPGTW => (
            avx::avx_ins(ins, &[0x65], &[0x65], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPCMPGTD => (
            avx::avx_ins(ins, &[0x66], &[0x66], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPACKUSWB => (
            avx::avx_ins(ins, &[0x67], &[0x67], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPACKSSWB => (
            avx::avx_ins(ins, &[0x63], &[0x63], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPACKSSDW => (
            avx::avx_ins(ins, &[0x6B], &[0x6B], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPUNPCKLBW => (
            avx::avx_ins(ins, &[0x60], &[0x60], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPUNPCKLWD => (
            avx::avx_ins(ins, &[0x61], &[0x61], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPUNPCKLDQ => (
            avx::avx_ins(ins, &[0x62], &[0x62], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPUNPCKHBW => (
            avx::avx_ins(ins, &[0x68], &[0x68], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPUNPCKHWD => (
            avx::avx_ins(ins, &[0x69], &[0x69], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPUNPCKHDQ => (
            avx::avx_ins(ins, &[0x6A], &[0x6A], None, 0x66, 0x0F, false),
            None,
        ),

        // part2a
        Ins::PAVGB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE0])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PAVGW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE3])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPAVGB => (
            avx::avx_ins(ins, &[0xE0], &[0xE0], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPAVGW => (
            avx::avx_ins(ins, &[0xE3], &[0xE3], None, 0x66, 0x0F, false),
            None,
        ),
        Ins::VPHADDW => (
            avx::avx_ins(ins, &[0x01], &[0x01], None, 0x66, 0x38, false),
            None,
        ),
        Ins::VPHADDD => (
            avx::avx_ins(ins, &[0x02], &[0x02], None, 0x66, 0x38, false),
            None,
        ),
        Ins::VPHSUBW => (
            avx::avx_ins(ins, &[0x05], &[0x05], None, 0x66, 0x38, false),
            None,
        ),
        Ins::VPHSUBD => (
            avx::avx_ins(ins, &[0x06], &[0x06], None, 0x66, 0x38, false),
            None,
        ),
        Ins::VZEROUPPER => (vec![0xC5, 0xF8, 0x77], None),
        Ins::VZEROALL => (vec![0xC5, 0xFC, 0x77], None),
        Ins::VPALIGNR => (
            avx::avx_ins_wimm3(ins, &[0x0F], &[0x0F], None, 0x66, 0x3A, false),
            None,
        ),
        Ins::VINSERTF128 => (
            GenAPI::new()
                .opcode(&[0x18])
                .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                .modrm(true, None, None)
                .imm_atindex(3, 1)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VEXTRACTF128 => (
            avx::avx_ins_wimm2(ins, &[0x19], &[0x19], None, 0x66, 0x3A, false),
            None,
        ),
        Ins::VBROADCASTSS => (
            GenAPI::new()
                .opcode(&[0x18])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VBROADCASTSD => (
            GenAPI::new()
                .opcode(&[0x19])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VBROADCASTF128 => (
            GenAPI::new()
                .opcode(&[0x1A])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .assemble(ins, bits),
            None,
        ),
        Ins::STMXCSR => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(3), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::LDMXCSR => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(2), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::VSTMXCSR => (
            GenAPI::new()
                .opcode(&[0xAE])
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false))
                .modrm(true, Some(3), None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VLDMXCSR => (
            GenAPI::new()
                .opcode(&[0xAE])
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false))
                .modrm(true, Some(2), None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMOVMSKPS => (
            GenAPI::new()
                .opcode(&[0x50])
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPERMILPS => {
            if let Some(Operand::Imm(_)) = ins.src2() {
                (
                    GenAPI::new()
                        .modrm(true, None, None)
                        .opcode(&[0x04])
                        .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                        .ord(&[MODRM_REG, MODRM_RM])
                        .imm_atindex(2, 1)
                        .assemble(ins, bits),
                    None,
                )
            } else {
                (
                    GenAPI::new()
                        .modrm(true, None, None)
                        .opcode(&[0x0C])
                        .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                        .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                        .assemble(ins, bits),
                    None,
                )
            }
        }
        Ins::VPERMILPD => {
            if let Some(Operand::Imm(_)) = ins.src2() {
                (
                    GenAPI::new()
                        .modrm(true, None, None)
                        .opcode(&[0x05])
                        .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                        .ord(&[MODRM_REG, MODRM_RM])
                        .imm_atindex(2, 1)
                        .assemble(ins, bits),
                    None,
                )
            } else {
                (
                    GenAPI::new()
                        .modrm(true, None, None)
                        .opcode(&[0x0D])
                        .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                        .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                        .assemble(ins, bits),
                    None,
                )
            }
        }
        Ins::PCLMULQDQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x44])
                .prefix(0x66)
                .rex(true)
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCLMULQDQ => (
            GenAPI::new()
                .opcode(&[0x44])
                .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                .imm_atindex(3, 1)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPERM2F128 => (
            GenAPI::new()
                .opcode(&[0x06])
                .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                .imm_atindex(3, 1)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPERM2I128 => (
            GenAPI::new()
                .opcode(&[0x46])
                .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                .imm_atindex(3, 1)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        // part2c
        Ins::VPINSRW => (
            GenAPI::new()
                .opcode(&[0xC4])
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
                .imm_atindex(3, 1)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMAXSW => (
            GenAPI::new()
                .opcode(&[0xEE])
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMINSW => (
            GenAPI::new()
                .opcode(&[0xEA])
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSRLDQ => (
            GenAPI::new()
                .opcode(&[0x73])
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
                .imm_atindex(2, 1)
                .modrm(true, Some(3), None)
                .ord(&[VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSIGNB => (
            GenAPI::new()
                .opcode(&[0x08])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSIGNW => (
            GenAPI::new()
                .opcode(&[0x09])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSIGND => (
            GenAPI::new()
                .opcode(&[0x0A])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMULUDQ => (
            GenAPI::new()
                .opcode(&[0xF4])
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMULHUW => (
            GenAPI::new()
                .opcode(&[0xE4])
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMULHRSW => (
            GenAPI::new()
                .opcode(&[0x0B])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        // part2c-ext
        Ins::PMAXSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEE])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PINSRW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xC4])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PMINSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEA])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::PMAXUD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x3F])
                .prefix(0x66)
                .rex(true)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMAXUD => (
            GenAPI::new()
                .opcode(&[0x3F])
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PMULHUW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE4])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66);
            }
            (api.assemble(ins, bits), None)
        }
        // fma-part1
        Ins::VFMADD132PS => (
            GenAPI::new()
                .opcode(&[0x98])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD213PS => (
            GenAPI::new()
                .opcode(&[0xA8])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD231PS => (
            GenAPI::new()
                .opcode(&[0xB8])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD132PD => (
            GenAPI::new()
                .opcode(&[0x98])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD213PD => (
            GenAPI::new()
                .opcode(&[0xA8])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD231PD => (
            GenAPI::new()
                .opcode(&[0xB8])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD132SD => (
            GenAPI::new()
                .opcode(&[0x99])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD213SD => (
            GenAPI::new()
                .opcode(&[0xA9])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD231SD => (
            GenAPI::new()
                .opcode(&[0xB9])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD132SS => (
            GenAPI::new()
                .opcode(&[0x99])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD213SS => (
            GenAPI::new()
                .opcode(&[0xA9])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADD231SS => (
            GenAPI::new()
                .opcode(&[0xB9])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFMSUB132PS => (
            GenAPI::new()
                .opcode(&[0x9A])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB213PS => (
            GenAPI::new()
                .opcode(&[0xAA])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB231PS => (
            GenAPI::new()
                .opcode(&[0xBA])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFMSUB132PD => (
            GenAPI::new()
                .opcode(&[0x9A])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB213PD => (
            GenAPI::new()
                .opcode(&[0xAA])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB231PD => (
            GenAPI::new()
                .opcode(&[0xBA])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB132SD => (
            GenAPI::new()
                .opcode(&[0x9B])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB213SD => (
            GenAPI::new()
                .opcode(&[0xAB])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB231SD => (
            GenAPI::new()
                .opcode(&[0xBB])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB132SS => (
            GenAPI::new()
                .opcode(&[0x9B])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB213SS => (
            GenAPI::new()
                .opcode(&[0xAB])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUB231SS => (
            GenAPI::new()
                .opcode(&[0xBB])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        // fma-part2
        Ins::VFNMADD132PS => (
            GenAPI::new()
                .opcode(&[0x9C])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMADD213PS => (
            GenAPI::new()
                .opcode(&[0xAC])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMADD231PS => (
            GenAPI::new()
                .opcode(&[0xBC])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFNMADD132PD => (
            GenAPI::new()
                .opcode(&[0x9C])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMADD213PD => (
            GenAPI::new()
                .opcode(&[0xAC])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMADD231PD => (
            GenAPI::new()
                .opcode(&[0xBC])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFNMADD132SS => (
            GenAPI::new()
                .opcode(&[0x9D])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMADD213SS => (
            GenAPI::new()
                .opcode(&[0xAD])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMADD231SS => (
            GenAPI::new()
                .opcode(&[0xBD])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFNMADD132SD => (
            GenAPI::new()
                .opcode(&[0x9D])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMADD213SD => (
            GenAPI::new()
                .opcode(&[0xAD])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMADD231SD => (
            GenAPI::new()
                .opcode(&[0xBD])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFNMSUB132PS => (
            GenAPI::new()
                .opcode(&[0x9E])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMSUB213PS => (
            GenAPI::new()
                .opcode(&[0xAE])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMSUB231PS => (
            GenAPI::new()
                .opcode(&[0xBE])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFNMSUB132PD => (
            GenAPI::new()
                .opcode(&[0x9E])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMSUB213PD => (
            GenAPI::new()
                .opcode(&[0xAE])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMSUB231PD => (
            GenAPI::new()
                .opcode(&[0xBE])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFNMSUB132SS => (
            GenAPI::new()
                .opcode(&[0x9F])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMSUB213SS => (
            GenAPI::new()
                .opcode(&[0xAF])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMSUB231SS => (
            GenAPI::new()
                .opcode(&[0xBF])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFNMSUB132SD => (
            GenAPI::new()
                .opcode(&[0x9F])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMSUB213SD => (
            GenAPI::new()
                .opcode(&[0xAF])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFNMSUB231SD => (
            GenAPI::new()
                .opcode(&[0xBF])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        // fma-part3
        Ins::VFMADDSUB132PS => (
            GenAPI::new()
                .opcode(&[0x96])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADDSUB213PS => (
            GenAPI::new()
                .opcode(&[0xA6])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADDSUB231PS => (
            GenAPI::new()
                .opcode(&[0xB6])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADDSUB132PD => (
            GenAPI::new()
                .opcode(&[0x96])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADDSUB213PD => (
            GenAPI::new()
                .opcode(&[0xA6])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMADDSUB231PD => (
            GenAPI::new()
                .opcode(&[0xB6])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::VFMSUBADD132PS => (
            GenAPI::new()
                .opcode(&[0x97])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUBADD213PS => (
            GenAPI::new()
                .opcode(&[0xA7])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUBADD231PS => (
            GenAPI::new()
                .opcode(&[0xB7])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUBADD132PD => (
            GenAPI::new()
                .opcode(&[0x97])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUBADD213PD => (
            GenAPI::new()
                .opcode(&[0xA7])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VFMSUBADD231PD => (
            GenAPI::new()
                .opcode(&[0xB7])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        // aes
        Ins::AESDEC => (
            GenAPI::new()
                .prefix(0x66)
                .opcode(&[0x0F, 0x38, 0xDE])
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::AESENC => (
            GenAPI::new()
                .prefix(0x66)
                .opcode(&[0x0F, 0x38, 0xDC])
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::AESIMC => (
            GenAPI::new()
                .prefix(0x66)
                .opcode(&[0x0F, 0x38, 0xDB])
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::AESDECLAST => (
            GenAPI::new()
                .prefix(0x66)
                .opcode(&[0x0F, 0x38, 0xDF])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::AESENCLAST => (
            GenAPI::new()
                .prefix(0x66)
                .opcode(&[0x0F, 0x38, 0xDD])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::VAESDEC => (
            GenAPI::new()
                .opcode(&[0xDE])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VAESENC => (
            GenAPI::new()
                .opcode(&[0xDC])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VAESIMC => (
            GenAPI::new()
                .opcode(&[0xDB])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VAESENCLAST => (
            GenAPI::new()
                .opcode(&[0xDD])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VAESDECLAST => (
            GenAPI::new()
                .opcode(&[0xDF])
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VAESKEYGENASSIST => (
            GenAPI::new()
                .opcode(&[0xDF])
                .imm_atindex(2, 1)
                .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::AESKEYGENASSIST => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0xDF])
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .prefix(0x66)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        // cvt-part1
        Ins::CVTPD2PI => (
            GenAPI::new()
                .prefix(0x66)
                .opcode(&[0x0F, 0x2D])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTSS2SD => (
            GenAPI::new()
                .prefix(0xF3)
                .opcode(&[0x0F, 0x5A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTPD2PS => (
            GenAPI::new()
                .prefix(0x66)
                .opcode(&[0x0F, 0x5A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTPS2PD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTPI2PD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2A])
                .prefix(0x66)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTPD2DQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0xE6])
                .prefix(0xF2)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTSD2SS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5A])
                .prefix(0xF2)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTPS2DQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5B])
                .prefix(0x66)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTDQ2PS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5B])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTDQ2PD => (
            GenAPI::new()
                .opcode(&[0x0F, 0xE6])
                .modrm(true, None, None)
                .prefix(0xF3)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTSD2SI => (sse2::sgen_ins_wrev(ins, bits, false, &[0x0F, 0x2D]), None),
        Ins::CVTSI2SD => (sse2::sgen_ins_wrev(ins, bits, false, &[0x0F, 0x2A]), None),
        Ins::CVTTPS2DQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x5B])
                .prefix(0xF3)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTTSD2SI => (
            sse2::sgen_ins_wrev(ins, bits, false, &[0x66, 0x0F, 0x2C]),
            None,
        ),
        Ins::CVTTPD2PI => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2C])
                .modrm(true, None, None)
                .prefix(0x66)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTSI2SS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2A])
                .prefix(0xF3)
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTPS2PI => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2D])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTTPS2PI => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2C])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTPI2PS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTTPD2DQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0xE6])
                .modrm(true, None, None)
                .prefix(0x66)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTTSS2SI => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2C])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .prefix(0xF3)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTSS2SI => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2D])
                .prefix(0xF3)
                .rex(true)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        // cvt-part2
        Ins::VCVTPD2DQ => (
            GenAPI::new()
                .opcode(&[0xE6])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTPD2PS => (
            GenAPI::new()
                .opcode(&[0x5A])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTPS2DQ => (
            GenAPI::new()
                .opcode(&[0x5B])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTPS2PD => (
            GenAPI::new()
                .opcode(&[0x5A])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTSD2SI => (
            GenAPI::new()
                .opcode(&[0x2D])
                .vex(
                    VexDetails::new()
                        .map_select(0x0F)
                        .pp(0xF2)
                        .vex_we(ins.dst().unwrap().size() == Size::Qword),
                )
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTSD2SS => (
            GenAPI::new()
                .opcode(&[0x5A])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTSI2SD => (
            GenAPI::new()
                .opcode(&[0x2A])
                .vex(
                    VexDetails::new()
                        .map_select(0x0F)
                        .pp(0xF2)
                        .vex_we(ins.src2().unwrap().size() == Size::Qword),
                )
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .modrm(true, None, None)
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTSI2SS => (
            GenAPI::new()
                .opcode(&[0x2A])
                .vex(
                    VexDetails::new()
                        .map_select(0x0F)
                        .pp(0xF3)
                        .vex_we(ins.src2().unwrap().size() == Size::Qword),
                )
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTSS2SD => (
            GenAPI::new()
                .opcode(&[0x5A])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTSS2SI => (
            GenAPI::new()
                .opcode(&[0x2D])
                .vex(
                    VexDetails::new()
                        .map_select(0x0F)
                        .pp(0xF3)
                        .vex_we(ins.dst().unwrap().size() == Size::Qword),
                )
                .ord(&[MODRM_REG, MODRM_RM])
                .modrm(true, None, None)
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTDQ2PD => (
            GenAPI::new()
                .opcode(&[0xE6])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTDQ2PS => (
            GenAPI::new()
                .opcode(&[0x5B])
                .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTTPD2DQ => (
            GenAPI::new()
                .opcode(&[0xE6])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTTPS2DQ => (
            GenAPI::new()
                .opcode(&[0x5B])
                .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTTSD2SI => (
            GenAPI::new()
                .opcode(&[0x2C])
                .vex(
                    VexDetails::new()
                        .map_select(0x0F)
                        .pp(0xF2)
                        .vex_we(ins.dst().unwrap().size() == Size::Qword),
                )
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VCVTTSS2SI => (
            GenAPI::new()
                .opcode(&[0x2C])
                .vex(
                    VexDetails::new()
                        .map_select(0x0F)
                        .pp(0xF3)
                        .vex_we(ins.dst().unwrap().size() == Size::Qword),
                )
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        // norm-part1a
        Ins::BT => (ins_bt(ins, &[0x0F, 0xA3], &[0x0F, 0xBA], bits, 4), None),
        Ins::BTS => (ins_bt(ins, &[0x0F, 0xAB], &[0x0F, 0xBA], bits, 5), None),
        Ins::BTC => (ins_bt(ins, &[0x0F, 0xBB], &[0x0F, 0xBA], bits, 7), None),
        Ins::BTR => (ins_bt(ins, &[0x0F, 0xB3], &[0x0F, 0xBA], bits, 6), None),
        Ins::CLC => (vec![0xF8], None),
        Ins::CMC => (vec![0xF5], None),
        Ins::CWD => (vec![0x66, 0x99], None),
        Ins::CDQ => (vec![0x99], None),
        Ins::CQO => (vec![0b0100_1000, 0x99], None),
        Ins::DAA => (vec![0x27], None),
        Ins::DAS => (vec![0x2F], None),
        Ins::CLD => (vec![0xFC], None),
        Ins::CBW => (vec![0x66, 0x98], None),
        Ins::CLI => (vec![0xFA], None),
        Ins::AAA => (vec![0x37], None),
        Ins::AAS => (vec![0x3F], None),
        Ins::AAD => (
            vec![
                0xD5,
                if let Some(Operand::Imm(n)) = ins.dst() {
                    n.split_into_bytes()[0]
                } else {
                    0x0A
                },
            ],
            None,
        ),
        Ins::AAM => (
            vec![
                0xD4,
                if let Some(Operand::Imm(n)) = ins.dst() {
                    n.split_into_bytes()[0]
                } else {
                    0x0A
                },
            ],
            None,
        ),
        Ins::ADC => (
            add_like_ins(
                ins,
                &[0x14, 0x15, 0x80, 0x81, 0x83, 0x10, 0x11, 0x12, 0x13],
                2,
                bits,
            ),
            None,
        ),
        Ins::BSF => (
            GenAPI::new()
                .opcode(&[0x0F, 0xBC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::BSR => (
            GenAPI::new()
                .opcode(&[0x0F, 0xBD])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        // part b
        Ins::ADCX => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xF6])
                .modrm(true, None, None)
                .prefix(0x66)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::ADOX => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xF6])
                .modrm(true, None, None)
                .prefix(0xF3)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::ANDN => (
            GenAPI::new()
                .opcode(&[0xF2])
                .modrm(true, None, None)
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .pp(0x00)
                        .vex_we(ins.size() == Size::Qword),
                )
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::CWDE => (vec![0x98], None),
        Ins::CDQE => (vec![0b0100_1000, 0x98], None),
        Ins::CLAC => (vec![0x0F, 0x01, 0xCA], None),
        Ins::CLTS => (vec![0x0F, 0x06], None),
        Ins::CLUI => (vec![0xF3, 0x0F, 0x01, 0xEE], None),
        Ins::CLWB => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .prefix(0x66)
                .modrm(true, Some(6), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::ARPL => (
            GenAPI::new()
                .opcode(&[0x63])
                .modrm(true, None, None)
                .assemble(ins, bits),
            None,
        ),

        Ins::BLSR => (
            GenAPI::new()
                .opcode(&[0xF3])
                .modrm(true, Some(1), None)
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .vex_we(ins.size() == Size::Qword),
                )
                .ord(&[VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::BLSI => (
            GenAPI::new()
                .opcode(&[0xF3])
                .modrm(true, Some(3), None)
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .vex_we(ins.size() == Size::Qword),
                )
                .ord(&[VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::BZHI => (
            GenAPI::new()
                .opcode(&[0xF5])
                .modrm(true, None, None)
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .vex_we(ins.size() == Size::Qword),
                )
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::BEXTR => (
            GenAPI::new()
                .opcode(&[0xF7])
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .vex_we(ins.size() == Size::Qword),
                )
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::BLSMSK => (
            GenAPI::new()
                .opcode(&[0xF3])
                .modrm(true, Some(2), None)
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .vex_we(ins.size() == Size::Qword),
                )
                .ord(&[VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::BSWAP => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC8 + ins.reg_byte(0).unwrap_or(0)])
                .modrm(false, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        // part c
        Ins::CMPSTRB => (vec![0xA6], None),
        Ins::CMPSTRW => (
            GenAPI::new()
                .opcode(&[0xA7])
                .fixed_size(Size::Word)
                .assemble(ins, bits),
            None,
        ),
        Ins::CMPSTRD => (
            GenAPI::new()
                .opcode(&[0xA7])
                .fixed_size(Size::Dword)
                .assemble(ins, bits),
            None,
        ),
        Ins::CMPSTRQ => (vec![0b0100_1000, 0xA7], None),
        Ins::ENDBR64 => (vec![0xF3, 0x0F, 0x1E, 0xFA], None),
        Ins::ENDBR32 => (vec![0xF3, 0x0F, 0x1E, 0xFB], None),
        Ins::CMPXCHG => (
            GenAPI::new()
                .opcode(&[0x0F, (0xB1 - ((ins.size() == Size::Byte) as u8))])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CLDEMOTE => (
            GenAPI::new()
                .opcode(&[0x0F, 0x1C])
                .modrm(true, Some(0), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::CLRSSBSY => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .prefix(0xF3)
                .modrm(true, Some(6), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::CMPXCHG8B => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC7])
                .modrm(true, Some(1), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::CMPXCHG16B => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC7])
                .modrm(true, Some(1), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        // part 3
        Ins::ENTER => (ins_enter(ins, bits), None),
        Ins::HLT => (vec![0xF4], None),
        Ins::HRESET => (
            GenAPI::new()
                .opcode(&[0xF3, 0x0F, 0x3A, 0xF0, 0xC0])
                .modrm(false, None, None)
                .imm_atindex(0, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::INPORTB => (
            GenAPI::new()
                .opcode(&[0xE4])
                .fixed_size(Size::Byte)
                .modrm(false, None, None)
                .imm_atindex(0, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::INPORTW => (
            GenAPI::new()
                .opcode(&[0xE5])
                .fixed_size(Size::Word)
                .imm_atindex(0, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::INPORTD => (
            GenAPI::new()
                .opcode(&[0xE5])
                .fixed_size(Size::Dword)
                .imm_atindex(0, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::INDXB => (vec![0xEC], None),
        Ins::INDXW => (
            GenAPI::new()
                .opcode(&[0xED])
                .fixed_size(Size::Word)
                .assemble(ins, bits),
            None,
        ),
        Ins::INDXD => (
            GenAPI::new()
                .opcode(&[0xED])
                .fixed_size(Size::Dword)
                .assemble(ins, bits),
            None,
        ),
        Ins::INSB => (vec![0x6C], None),
        Ins::INSW => (vec![0x66, 0x6D], None),
        Ins::INSD => (vec![0x6D], None),

        Ins::INT => (
            vec![
                0xCC,
                if let Some(Operand::Imm(imm)) = ins.dst() {
                    imm.split_into_bytes()[0]
                } else {
                    0x00
                },
            ],
            None,
        ),
        Ins::INTO => (vec![0xCE], None),
        Ins::INT3 => (vec![0xCC], None),
        Ins::INT1 => (vec![0xF1], None),
        Ins::INVD => (vec![0x0F, 0x08], None),
        Ins::INVLPG => (vec![0x0F, 0x01, 0b11_111_000], None),
        Ins::INVPCID => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x82])
                .prefix(0x66)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::IRET | Ins::IRETD => (vec![0xCF], None),
        Ins::IRETQ => (vec![0b0100_1000, 0xCF], None),
        Ins::LAHF => (vec![0x9F], None),
        Ins::LAR => (
            GenAPI::new()
                .opcode(&[0x0F, 0x02])
                .ord(&[MODRM_REG, MODRM_RM])
                .modrm(true, None, None)
                .assemble(ins, bits),
            None,
        ),
        Ins::LEAVE => (vec![0xC9], None),
        Ins::LLDT => (
            GenAPI::new()
                .opcode(&[0x0F, 0x00])
                .modrm(true, Some(2), None)
                .can_h66(false)
                .assemble(ins, bits),
            None,
        ),
        Ins::LMSW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x01])
                .modrm(true, Some(6), None)
                .can_h66(false)
                .assemble(ins, bits),
            None,
        ),
        Ins::LODSB => (vec![0xAC], None),
        Ins::LODSW => (
            GenAPI::new()
                .fixed_size(Size::Word)
                .opcode(&[0xAD])
                .assemble(ins, bits),
            None,
        ),
        Ins::LODSD => (
            GenAPI::new()
                .fixed_size(Size::Dword)
                .opcode(&[0xAD])
                .assemble(ins, bits),
            None,
        ),
        Ins::LODSQ => (vec![0x48, 0xAD], None),

        // part 3
        Ins::LOOP => ins_shrtjmp(ins, vec![0xE2]),
        Ins::LOOPE => ins_shrtjmp(ins, vec![0xE1]),
        Ins::LOOPNE => ins_shrtjmp(ins, vec![0xE0]),

        Ins::LSL => (
            GenAPI::new()
                .opcode(&[0x0F, 0x03])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::LTR => (
            GenAPI::new()
                .opcode(&[0x0F, 0x00])
                .modrm(true, Some(3), None)
                .can_h66(false)
                .assemble(ins, bits),
            None,
        ),
        Ins::LZCNT => (
            GenAPI::new()
                .opcode(&[0x0F, 0xBD])
                .prefix(0xF3)
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVBE => (
            {
                let mut api = GenAPI::new().modrm(true, None, None).rex(true);
                if let Some(Operand::Reg(_)) = ins.dst() {
                    api = api.opcode(&[0x0F, 0x38, 0xF0]);
                    api = api.ord(&[MODRM_REG, MODRM_RM]);
                } else {
                    api = api.opcode(&[0x0F, 0x38, 0xF1]);
                }
                api.assemble(ins, bits)
            },
            None,
        ),
        Ins::MOVZX => (
            GenAPI::new()
                .opcode(&[
                    0x0F,
                    (0xB6 + ((ins.src().unwrap().size() == Size::Word) as u8)),
                ])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVDIRI => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xF9])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVSTRB => (vec![0xA4], None),
        Ins::MOVSTRW => (
            GenAPI::new()
                .opcode(&[0xA5])
                .fixed_size(Size::Word)
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVSTRD => (
            GenAPI::new()
                .opcode(&[0xA5])
                .fixed_size(Size::Dword)
                .assemble(ins, bits),
            None,
        ),
        Ins::MOVSTRQ => (vec![0x48, 0xA5], None),
        Ins::MULX => (
            GenAPI::new()
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .pp(0xF2)
                        .vex_we(ins.size() == Size::Qword),
                )
                .opcode(&[0xF6])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),

        Ins::OUTIB => (
            vec![
                0xE6,
                if let Operand::Imm(i) = ins.dst().unwrap() {
                    i.split_into_bytes()[0]
                } else {
                    0x00
                },
            ],
            None,
        ),
        Ins::OUTID => (
            GenAPI::new()
                .opcode(&[0xE7])
                .fixed_size(Size::Dword)
                .imm_atindex(0, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::OUTIW => (
            GenAPI::new()
                .opcode(&[0xE7])
                .fixed_size(Size::Word)
                .imm_atindex(0, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::OUTRB => (vec![0xEE], None),
        Ins::OUTRW => (
            GenAPI::new()
                .opcode(&[0xEF])
                .fixed_size(Size::Word)
                .assemble(ins, bits),
            None,
        ),
        Ins::OUTRD => (
            GenAPI::new()
                .opcode(&[0xEF])
                .fixed_size(Size::Dword)
                .assemble(ins, bits),
            None,
        ),
        Ins::OUTSB => (vec![0x6E], None),
        Ins::OUTSW => (
            GenAPI::new()
                .opcode(&[0x6F])
                .fixed_size(Size::Word)
                .assemble(ins, bits),
            None,
        ),
        Ins::OUTSD => (
            GenAPI::new()
                .opcode(&[0x6F])
                .fixed_size(Size::Dword)
                .assemble(ins, bits),
            None,
        ),
        Ins::PEXT => (
            GenAPI::new()
                .opcode(&[0xF5])
                .modrm(true, None, None)
                .vex(
                    VexDetails::new()
                        .pp(0xF3)
                        .map_select(0x38)
                        .vex_we(ins.size() == Size::Qword),
                )
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PDEP => (
            GenAPI::new()
                .opcode(&[0xF5])
                .modrm(true, None, None)
                .vex(
                    VexDetails::new()
                        .pp(0xF2)
                        .map_select(0x38)
                        .vex_we(ins.size() == Size::Qword),
                )
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::PREFETCHW => (
            GenAPI::new()
                .opcode(&[0x0F, 0x0D])
                .modrm(true, Some(1), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::PREFETCH0 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x18])
                .modrm(true, Some(1), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::PREFETCH1 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x18])
                .modrm(true, Some(2), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::PREFETCH2 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x18])
                .modrm(true, Some(3), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::PREFETCHA => (
            GenAPI::new()
                .opcode(&[0x0F, 0x18])
                .modrm(true, Some(0), None)
                .assemble(ins, bits),
            None,
        ),

        Ins::ROL => (
            ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 0, bits),
            None,
        ),
        Ins::ROR => (
            ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 1, bits),
            None,
        ),
        Ins::RCL => (
            ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 2, bits),
            None,
        ),
        Ins::RCR => (
            ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 3, bits),
            None,
        ),
        // part 4
        Ins::RDMSR => (vec![0x0F, 0x32], None),
        Ins::RDPID => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC7])
                .prefix(0xF3)
                .modrm(true, Some(7), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::RDPKRU => (vec![0x0F, 0x01, 0xEE], None),
        Ins::RDPMC => (vec![0x0F, 0x33], None),
        Ins::RDRAND => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC7])
                .modrm(true, Some(6), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::RDSEED => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC7])
                .modrm(true, Some(7), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::RDSSPD | Ins::RDSSPQ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x1E])
                .modrm(true, Some(1), None)
                .prefix(0xF3)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::RDTSC => (vec![0x0F, 0x31], None),
        Ins::RDTSCP => (vec![0x0F, 0x01, 0xF9], None),
        Ins::RORX => (
            GenAPI::new()
                .opcode(&[0xF0])
                .vex(
                    VexDetails::new()
                        .map_select(0x3A)
                        .pp(0xF2)
                        .vex_we(ins.size() == Size::Qword)
                        .vlength(Some(false)),
                )
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::RSM => (vec![0x0F, 0xAA], None),
        Ins::RSTORSSP => (
            GenAPI::new()
                .opcode(&[0x0F, 0x01])
                .modrm(true, Some(5), None)
                .prefix(0xF3)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::SAHF => (vec![0x9E], None),
        Ins::SHLX => (
            GenAPI::new()
                .opcode(&[0xF7])
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .pp(0x66)
                        .vex_we(ins.size() == Size::Qword),
                )
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::SHRX => (
            GenAPI::new()
                .opcode(&[0xF7])
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .pp(0xF2)
                        .vex_we(ins.size() == Size::Qword),
                )
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::SARX => (
            GenAPI::new()
                .opcode(&[0xF7])
                .vex(
                    VexDetails::new()
                        .map_select(0x38)
                        .pp(0xF3)
                        .vex_we(ins.size() == Size::Qword),
                )
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::SBB => (
            add_like_ins(
                ins,
                &[0x1C, 0x1D, 0x80, 0x81, 0x83, 0x18, 0x19, 0x1A, 0x1B],
                3,
                bits,
            ),
            None,
        ),
        Ins::SCASB => (vec![0xAE], None),
        Ins::SCASW => (
            GenAPI::new()
                .fixed_size(Size::Word)
                .opcode(&[0xAF])
                .assemble(ins, bits),
            None,
        ),
        Ins::SCASD => (
            GenAPI::new()
                .fixed_size(Size::Dword)
                .opcode(&[0xAF])
                .assemble(ins, bits),
            None,
        ),
        Ins::SCASQ => (vec![0x48, 0xAF], None),
        Ins::SENDUIPI => (
            GenAPI::new()
                .prefix(0xF3)
                .opcode(&[0x0F, 0xC7])
                .modrm(true, Some(6), None)
                .can_h66(false)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::SERIALIZE => (vec![0x0F, 0x01, 0xE8], None),
        // for some reason NASM generates this as no opcode at all?
        Ins::SETSSBY => (vec![], None),

        // setcc
        Ins::SETO => (
            GenAPI::new()
                .opcode(&[0x0F, 0x90])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::SETNO => (
            GenAPI::new()
                .opcode(&[0x0F, 0x91])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::SETB | Ins::SETC | Ins::SETNAE => (
            GenAPI::new()
                .opcode(&[0x0F, 0x92])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETAE | Ins::SETNB | Ins::SETNC => (
            GenAPI::new()
                .opcode(&[0x0F, 0x93])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETE | Ins::SETZ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x94])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::SETNE | Ins::SETNZ => (
            GenAPI::new()
                .opcode(&[0x0F, 0x95])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETBE | Ins::SETNA => (
            GenAPI::new()
                .opcode(&[0x0F, 0x96])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETA | Ins::SETNBE => (
            GenAPI::new()
                .opcode(&[0x0F, 0x97])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x98])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::SETNS => (
            GenAPI::new()
                .opcode(&[0x0F, 0x99])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETP | Ins::SETPE => (
            GenAPI::new()
                .opcode(&[0x0F, 0x9A])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETNP | Ins::SETPO => (
            GenAPI::new()
                .opcode(&[0x0F, 0x9B])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETL | Ins::SETNGE => (
            GenAPI::new()
                .opcode(&[0x0F, 0x9C])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETGE | Ins::SETNL => (
            GenAPI::new()
                .opcode(&[0x0F, 0x9D])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETLE | Ins::SETNG => (
            GenAPI::new()
                .opcode(&[0x0F, 0x9E])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        Ins::SETG | Ins::SETNLE => (
            GenAPI::new()
                .opcode(&[0x0F, 0x9F])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

        // norm-part5
        Ins::SFENCE => (vec![0x0F, 0xAE, 0xF8], None),
        Ins::STAC => (vec![0x0F, 0x01, 0xCB], None),
        Ins::STC => (vec![0xF9], None),
        Ins::STD => (vec![0xFD], None),
        Ins::STI => (vec![0xFB], None),
        Ins::STUI => (vec![0xF3, 0x0F, 0x01, 0xEF], None),
        Ins::STOSB => (vec![0xAA], None),
        Ins::STOSW => (vec![0x66, 0xAB], None),
        Ins::STOSD => (vec![0xAB], None),
        Ins::STOSQ => (vec![0x48, 0xAB], None),
        Ins::SYSENTER => (vec![0x0F, 0x34], None),
        Ins::SYSEXIT => (vec![0x0F, 0x35], None),
        Ins::SYSRET => (vec![0x0F, 0x07], None),
        Ins::TESTUI => (vec![0xF3, 0x0F, 0x01, 0xED], None),
        Ins::UD2 => (vec![0x0F, 0x0B], None),
        Ins::UD0 => (
            GenAPI::new()
                .opcode(&[0x0F, 0xFF])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::UD1 => (
            GenAPI::new()
                .opcode(&[0x0F, 0xB9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::TPAUSE => (
            GenAPI::new()
                .modrm(true, Some(6), None)
                .prefix(0x66)
                .opcode(&[0x0F, 0xAE])
                .assemble(ins, bits),
            None,
        ),
        Ins::UMWAIT => (
            GenAPI::new()
                .modrm(true, Some(6), None)
                .prefix(0xF2)
                .opcode(&[0x0F, 0xAE])
                .assemble(ins, bits),
            None,
        ),
        Ins::UMONITOR => (
            GenAPI::new()
                .modrm(true, Some(6), None)
                .prefix(0xF3)
                .opcode(&[0x0F, 0xAE])
                .assemble(ins, bits),
            None,
        ),
        Ins::SMSW => (
            GenAPI::new()
                .modrm(true, Some(4), None)
                .opcode(&[0x0F, 0x01])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::STR => (
            GenAPI::new()
                .modrm(true, Some(1), None)
                .opcode(&[0x0F, 0x00])
                .assemble(ins, bits),
            None,
        ),
        Ins::VERR => (
            GenAPI::new()
                .modrm(true, Some(4), None)
                .opcode(&[0x0F, 0x00])
                .can_h66(false)
                .assemble(ins, bits),
            None,
        ),
        Ins::VERW => (
            GenAPI::new()
                .modrm(true, Some(5), None)
                .opcode(&[0x0F, 0x00])
                .can_h66(false)
                .assemble(ins, bits),
            None,
        ),
        Ins::SHLD => (
            ins_shlx(ins, &[0x0F, 0xA4], &[0x0F, 0xA5]).assemble(ins, bits),
            None,
        ),
        Ins::SHRD => (
            ins_shlx(ins, &[0x0F, 0xAC], &[0x0F, 0xAD]).assemble(ins, bits),
            None,
        ),
        Ins::UIRET => (vec![0xF3, 0x0F, 0x01, 0xEC], None),
        Ins::WAIT | Ins::FWAIT => (vec![0x9B], None),
        Ins::WBINVD => (vec![0x0F, 0x09], None),
        Ins::WRMSR => (vec![0x0F, 0x30], None),
        Ins::WRPKRU => (vec![0x0F, 0x01, 0xEF], None),

        // norm-part6
        Ins::XABORT => (
            GenAPI::new()
                .imm_atindex(0, 1)
                .opcode(&[0xC6, 0xF8])
                .assemble(ins, bits),
            None,
        ),
        Ins::XACQUIRE => (vec![0xF2], None),
        Ins::XRELEASE => (vec![0xF3], None),
        Ins::XADD => (
            GenAPI::new()
                .opcode(&[0x0F, (0xC0 + ((ins.size() != Size::Byte) as u8))])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::XBEGIN => ins_xbegin(ins),
        Ins::XCHG => (ins_xchg(ins).assemble(ins, bits), None),
        Ins::XEND => (vec![0x0F, 0x01, 0xD5], None),
        Ins::XGETBV => (vec![0x0F, 0x01, 0xD0], None),
        Ins::XLAT | Ins::XLATB => (vec![0xD7], None),
        Ins::XLATB64 => (vec![0x48, 0xD7], None),
        Ins::XRESLDTRK => (vec![0xF2, 0x0F, 0x01, 0xE9], None),

        Ins::XRSTOR | Ins::XRSTOR64 => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(5), None)
                .assemble(ins, bits),
            None,
        ),
        Ins::XRSTORS | Ins::XRSTORS64 => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC7])
                .modrm(true, Some(3), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::XSAVE | Ins::XSAVE64 => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(4), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::XSAVEC | Ins::XSAVEC64 => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC7])
                .modrm(true, Some(4), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::XSAVEOPT | Ins::XSAVEOPT64 => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(6), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::XSAVES | Ins::XSAVES64 => (
            GenAPI::new()
                .opcode(&[0x0F, 0xC7])
                .modrm(true, Some(5), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::XSETBV => (vec![0x0F, 0x01, 0xD1], None),
        Ins::XSUSLDTRK => (vec![0xF2, 0x0F, 0x01, 0xE8], None),
        Ins::XTEST => (vec![0x0F, 0x01, 0xD6], None),
        // sha.asm
        Ins::SHA1MSG1 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xC9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::SHA1NEXTE => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xC8])
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::SHA1MSG2 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xCA])
                .modrm(true, None, None)
                .rex(true)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .assemble(ins, bits),
            None,
        ),
        Ins::SHA1RNDS4 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0xCC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .rex(true)
                .imm_atindex(2, 1)
                .assemble(ins, bits),
            None,
        ),
        Ins::SHA256RNDS2 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xCB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::SHA256MSG2 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xCD])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::SHA256MSG1 => (
            GenAPI::new()
                .opcode(&[0x0F, 0x38, 0xCC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        // other
        _ => todo!("Instruction unsupported in src/core/comp.rs: {:?}", ins),
    }
}

// #  #   #   ####  #####  #####  #   #   ####  #####  #   ###   #   #   ####
// #  ##  #  #        #    #   #  #   #  #        #    #  #   #  ##  #  #
// #  # # #   ###     #    ####   #   #  #        #    #  #   #  # # #   ###
// #  #  ##      #    #    #   #  #   #  #        #    #  #   #  #  ##      #
// #  #   #  ####     #    #   #   ###    ####    #    #   ###   #   #  ####
// (Instructions)

fn ins_xchg(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new().rex(true);
    match ins.size() {
        Size::Byte => {
            api = api.opcode(&[0x86]);
            if let Some(Operand::Reg(_)) = ins.dst() {
                api = api.ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            } else {
                api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV])
            }
            api = api.modrm(true, None, None);
        }
        Size::Word => {
            if let Some(Operand::Reg(r)) = ins.dst() {
                if r == &Register::AX {
                    let s = if let Some(Operand::Reg(r1)) = ins.src() {
                        r1.to_byte()
                    } else {
                        0
                    };
                    api = api.opcode(&[(0x90 + s)]);
                } else {
                    api = api.opcode(&[0x87]);
                    api = api.modrm(true, None, None);
                    api = api.ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                }
            } else {
                api = api.opcode(&[0x87]);
                api = api.modrm(true, None, None);
                api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV]);
            }
        }
        Size::Dword | Size::Qword => {
            if let Some(Operand::Reg(r)) = ins.dst() {
                if r == &Register::EAX || r == &Register::RAX {
                    let s = if let Some(Operand::Reg(r1)) = ins.src() {
                        r1.to_byte()
                    } else {
                        0
                    };
                    api = api.opcode(&[(0x90 + s)]);
                } else {
                    if let Some(Operand::Reg(Register::EAX | Register::RAX)) = ins.src() {
                        api = api.opcode(&[(0x90 + r.to_byte())]);
                    } else {
                        api = api.opcode(&[0x87]);
                        api = api.modrm(true, None, None);
                        api = api.ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                    }
                }
            } else {
                if let Some(Operand::Reg(_)) = ins.src() {
                    api = api.opcode(&[0x87]);
                    api = api.modrm(true, None, None);
                    api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV]);
                } else {
                    api = api.opcode(&[0x87]);
                    api = api.modrm(true, None, None);
                    api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV]);
                }
            }
        }
        _ => invalid(3977),
    }
    api
}

fn ins_xbegin(ins: &Instruction) -> (Vec<u8>, Option<Relocation>) {
    let symb_name = if let Some(Operand::SymbolRef(str)) = ins.dst() {
        str
    } else {
        invalid(3923)
    };

    (
        vec![0xC7, 0xF8],
        Some(Relocation {
            symbol: Cow::Owned(symb_name),
            rtype: RType::PCRel32,
            addend: -4,
            catg: RCategory::Jump,
            offset: 2,
            size: 4,
        }),
    )
}

fn ins_shlx(ins: &Instruction, opc_imm: &[u8], opc_rm: &[u8]) -> GenAPI {
    let mut api = GenAPI::new().rex(true).modrm(true, None, None);
    if let Some(Operand::Imm(_)) = ins.src2() {
        api = api.opcode(opc_imm).imm_atindex(2, 1);
    } else {
        api = api.opcode(opc_rm);
    }
    api
}

fn ins_shrtjmp(ins: &Instruction, mut opc: Vec<u8>) -> (Vec<u8>, Option<Relocation>) {
    match ins.dst().unwrap() {
        Operand::SymbolRef(s) => {
            let rel = Relocation {
                symbol: Cow::Owned(s),
                rtype: RType::PCRel32,
                offset: 1,
                addend: -1,
                catg: RCategory::Jump,
                size: 1,
            };
            opc.push(0x00);
            (opc, Some(rel))
        }
        _ => panic!("unexpected at shrtjmp"),
    }
}

fn ins_enter(ins: &Instruction, bits: u8) -> Vec<u8> {
    let mut immw = if let Some(Operand::Imm(w)) = ins.dst() {
        let mut vec = w.split_into_bytes();
        extend_imm(&mut vec, 2);
        vec![vec[0], vec[1]]
    } else {
        vec![0x00, 0x00]
    };
    let immb = if let Some(Operand::Imm(w)) = ins.src() {
        let vec = w.split_into_bytes();
        if vec[0] == 0x00 {
            vec![0x00]
        } else if vec[0] == 0x01 {
            vec![0x01]
        } else {
            vec![vec[0]]
        }
    } else {
        vec![0x00]
    };
    immw.extend(immb);
    gen_ins(ins, &[0xC8], (false, None, None), Some(immw), bits, false)
}

fn ins_bt(ins: &Instruction, opc_noimm: &[u8], opc_imm: &[u8], bits: u8, modrm: u8) -> Vec<u8> {
    let imm = if let Some(Operand::Imm(n)) = ins.src() {
        Some(vec![n.split_into_bytes()[0]])
    } else {
        None
    };

    let (opc, modrm) = if imm.is_some() {
        (opc_imm, Some(modrm))
    } else {
        (opc_noimm, None)
    };

    gen_ins(ins, opc, (true, modrm, None), imm, bits, false)
}

fn ins_cmovcc(ins: &Instruction, opc: &[u8], bits: u8) -> Vec<u8> {
    gen_ins(ins, opc, (true, None, None), None, bits, true)
}

fn ins_pop(ins: &Instruction, bits: u8) -> Vec<u8> {
    match ins.dst().unwrap() {
        Operand::Reg(r) => gen_base(ins, &[0x58 + r.to_byte()], bits, false),
        Operand::SegReg(r) => match r {
            Register::DS => vec![0x1F],
            Register::ES => vec![0x07],
            Register::SS => vec![0x17],
            Register::FS => vec![0x0F, 0xA1],
            Register::GS => vec![0x0F, 0xA9],
            Register::CS => vec![0x90],
            _ => invalid(34),
        },
        Operand::Mem(_) | Operand::Segment(_) => {
            vec![0x8F, modrm::gen_modrm(ins, None, Some(0), false)]
        }
        _ => invalid(33),
    }
}

fn ins_push(ins: &Instruction, bits: u8) -> Vec<u8> {
    match ins.dst().unwrap() {
        Operand::Reg(r) => gen_base(ins, &[0x50 + r.to_byte()], bits, false),
        Operand::SegReg(r) => match r {
            Register::CS => vec![0x0E],
            Register::SS => vec![0x16],
            Register::DS => vec![0x1E],
            Register::ES => vec![0x06],
            Register::FS => vec![0x0F, 0xA0],
            Register::GS => vec![0x0F, 0xA8],
            _ => invalid(32),
        },
        Operand::Imm(nb) => match nb.size() {
            Size::Byte => {
                let mut opc = vec![0x6A];
                opc.extend(nb.split_into_bytes());
                opc
            }
            Size::Word | Size::Dword => {
                let mut b = vec![0x68];
                let mut x = nb.split_into_bytes();
                extend_imm(&mut x, 4);
                b.extend(x);
                b
            }
            _ => invalid(31),
        },
        Operand::Mem(_) | Operand::Segment(_) => {
            gen_ins(ins, &[0xFF], (true, Some(6), None), None, bits, false)
        }
        _ => invalid(30),
    }
}

fn ins_mov(ins: &Instruction, bits: u8) -> Vec<u8> {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();
    if let Operand::Reg(r) = dst {
        match src {
            Operand::SegReg(_) => gen_ins(ins, &[0x8C], (true, None, None), None, bits, false),
            Operand::CtrReg(_) => {
                gen_ins(ins, &[0x0F, 0x20], (true, None, None), None, bits, false)
            }
            Operand::DbgReg(_) => {
                gen_ins(ins, &[0x0F, 0x21], (true, None, None), None, bits, false)
            }
            Operand::Imm(n) => {
                let size = dst.size();
                let opc = match size {
                    Size::Byte => 0xB0 + r.to_byte(),
                    Size::Word | Size::Dword | Size::Qword => 0xB8 + r.to_byte(),
                    _ => invalid(29),
                };
                let mut imm = n.split_into_bytes();
                if size == Size::Qword {
                    extend_imm(&mut imm, 4);
                } else {
                    extend_imm(&mut imm, size.into());
                }
                let mut base = gen_base(ins, &[opc], bits, false);
                base.extend(imm);
                base
            }
            Operand::Reg(_) | Operand::Mem(_) | Operand::Segment(_) => {
                let opc = if let Operand::Reg(_) = src {
                    match dst.size() {
                        Size::Byte => 0x88,
                        Size::Word | Size::Dword | Size::Qword => 0x89,
                        _ => invalid(28),
                    }
                } else {
                    match dst.size() {
                        Size::Byte => 0x8A,
                        Size::Word | Size::Dword | Size::Qword => 0x8B,
                        _ => invalid(27),
                    }
                };
                gen_ins(ins, &[opc], (true, None, None), None, bits, false)
            }
            _ => invalid(26),
        }
    } else if let Operand::CtrReg(_) = dst {
        GenAPI::new()
            .opcode(&[0x0F, 0x22])
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
            .rex(true)
            .assemble(ins, bits)
    } else if let Operand::DbgReg(_) = dst {
        GenAPI::new()
            .opcode(&[0x0F, 0x23])
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
            .rex(true)
            .assemble(ins, bits)
    } else if let Operand::SegReg(_) = dst {
        match src {
            Operand::Reg(_) | Operand::Mem(_) => GenAPI::new()
                .opcode(&[0x8E])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            _ => invalid(25),
        }
    } else if let Operand::Mem(_) | Operand::Segment(_) = dst {
        match src {
            Operand::Reg(_) => {
                let opc = match dst.size() {
                    Size::Byte => 0x88,
                    Size::Word | Size::Dword | Size::Qword => 0x89,
                    _ => invalid(24),
                };
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, None, None)
                    .rex(true)
                    .assemble(ins, bits)
            }
            Operand::Imm(_) => {
                let size = dst.size();
                let opc = match size {
                    Size::Byte => 0xC6,
                    Size::Word | Size::Dword | Size::Qword => 0xC7,
                    _ => invalid(23),
                };
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, Some(0), None)
                    .rex(true)
                    .imm_atindex(1, size as u16 + 1)
                    .assemble(ins, bits)
            }
            _ => invalid(22),
        }
    } else {
        invalid(21)
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
fn add_like_ins(ins: &Instruction, opc: &[u8; 9], ovrreg: u8, bits: u8) -> Vec<u8> {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            if let Size::Dword | Size::Word = srci.size() {
                if let Register::RAX | Register::EAX = dstr {
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[opc[1]], &imm, bits, false);
                } else if let Register::AX = dstr {
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[opc[1]], &imm, bits, false);
                }
            }
            if let Register::AL = dstr {
                if let Size::Byte = srci.size() {
                    return bs_imm(ins, &[opc[0]], &imm, bits, false);
                }
            } else if let Register::AX = dstr {
                if let Size::Byte = srci.size() {
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[opc[1]], &imm, bits, false);
                }
            }

            let opc = match dstr.size() {
                Size::Byte => opc[2],
                Size::Dword | Size::Qword | Size::Word => {
                    if imm.len() == 1 {
                        opc[4]
                    } else {
                        opc[3]
                    }
                }
                _ => invalid(20),
            };
            let mut base = gen_base(ins, &[opc], bits, false);
            base.push(modrm::gen_modrm(ins, Some(ovrreg), None, false));
            extend_imm(&mut imm, 1);
            base.extend(imm);
            base
        }
        (Operand::Segment(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.address.size().unwrap_or_default() {
                Size::Byte => opc[2],
                Size::Word => opc[3],
                Size::Dword => opc[3],
                Size::Qword => {
                    if imm.len() == 1 {
                        opc[4]
                    } else {
                        opc[3]
                    }
                }
                _ => invalid(19),
            };
            if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dstm.address.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 2);
            } else if let (Size::Byte, Size::Dword) =
                (srci.size(), dstm.address.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 4);
            } else if let (crate::shr::ins::Mnemonic::CMP, Size::Byte, Size::Qword) = (
                ins.mnem,
                srci.size(),
                dstm.address.size().unwrap_or_default(),
            ) {
                extend_imm(&mut imm, 4);
            } else if srci.size() != Size::Byte {
                extend_imm(&mut imm, 4);
            }

            gen_ins(
                ins,
                &[opc],
                (true, Some(ovrreg), None),
                Some(imm),
                bits,
                false,
            )
        }
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.size().unwrap_or_default() {
                Size::Byte => opc[2],
                Size::Word => opc[3],
                Size::Dword => opc[3],
                Size::Qword => {
                    if imm.len() == 1 {
                        opc[4]
                    } else {
                        opc[3]
                    }
                }
                _ => invalid(18),
            };
            if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 2);
            } else if let (Size::Byte, Size::Dword) = (srci.size(), dstm.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 4);
            } else if let (crate::shr::ins::Mnemonic::CMP, Size::Byte, Size::Qword) =
                (ins.mnem, srci.size(), dstm.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 4);
            } else if srci.size() != Size::Byte {
                extend_imm(&mut imm, 4);
            }

            gen_ins(
                ins,
                &[opc],
                (true, Some(ovrreg), None),
                Some(imm),
                bits,
                false,
            )
        }
        (Operand::Reg(r), Operand::Segment(_) | Operand::Mem(_) | Operand::Reg(_)) => {
            let opc = match r.size() {
                Size::Byte => opc[7],
                Size::Word | Size::Dword | Size::Qword => opc[6],
                _ => invalid(17),
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits, false)
        }
        (Operand::Segment(m), Operand::Reg(_)) => {
            let opc = match m.address.size().unwrap_or_default() {
                Size::Byte => opc[7],
                Size::Word | Size::Dword | Size::Qword => opc[6],
                _ => invalid(16),
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits, false)
        }
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size().unwrap_or_default() {
                Size::Byte => opc[7],
                Size::Word | Size::Dword | Size::Qword => opc[6],
                _ => invalid(15),
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits, false)
        }
        _ => invalid(14),
    }
}

fn ins_cmp(ins: &Instruction, bits: u8) -> Vec<u8> {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            if let Size::Dword | Size::Word = srci.size() {
                if let Register::RAX | Register::EAX = dstr {
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[0x3D], &imm, bits, false);
                } else if let Register::AX = dstr {
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0x3D], &imm, bits, false);
                }
            }
            if let Register::AL = dstr {
                if let Size::Byte = srci.size() {
                    return bs_imm(ins, &[0x3C], &imm, bits, false);
                }
            } else if let Register::AX = dstr {
                if let Size::Byte = srci.size() {
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0x3D], &imm, bits, false);
                }
            }

            let opc = match dstr.size() {
                Size::Byte => 0x80,
                Size::Dword | Size::Qword | Size::Word => {
                    if imm.len() == 1 {
                        if imm[0] <= 127 {
                            0x83
                        } else {
                            0x80
                        }
                    } else {
                        0x80
                    }
                }
                _ => invalid(13),
            };
            let mut base = gen_base(ins, &[opc], bits, false);
            base.push(modrm::gen_modrm(ins, Some(7), None, false));
            extend_imm(&mut imm, 1);
            base.extend(imm);
            base
        }
        (Operand::Segment(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.address.size().unwrap_or_default() {
                Size::Byte => 0x80,
                Size::Qword | Size::Word | Size::Dword => {
                    if imm.len() == 1 {
                        if imm[0] <= 127 {
                            0x83
                        } else {
                            0x81
                        }
                    } else {
                        0x81
                    }
                }
                _ => invalid(12),
            };
            if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dstm.address.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 2);
            } else if let (Size::Byte, Size::Dword | Size::Qword) =
                (srci.size(), dstm.address.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 4);
            } else if srci.size() != Size::Byte {
                extend_imm(&mut imm, 4);
            }

            gen_ins(ins, &[opc], (true, Some(7), None), Some(imm), bits, false)
        }
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.size().unwrap_or_default() {
                Size::Byte => 0x80,
                Size::Qword | Size::Word | Size::Dword => {
                    if imm.len() == 1 {
                        if imm[0] <= 127 {
                            0x83
                        } else {
                            0x81
                        }
                    } else {
                        0x81
                    }
                }
                _ => invalid(11),
            };
            if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 2);
            } else if let (Size::Byte, Size::Dword | Size::Qword) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 4);
            } else if srci.size() != Size::Byte {
                extend_imm(&mut imm, 4);
            }

            gen_ins(ins, &[opc], (true, Some(7), None), Some(imm), bits, false)
        }
        (Operand::Reg(r), Operand::Segment(_) | Operand::Mem(_) | Operand::Reg(_)) => {
            let opc = match r.size() {
                Size::Byte => 0x3A,
                Size::Word | Size::Dword | Size::Qword => 0x3B,
                _ => invalid(10),
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits, false)
        }
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size().unwrap_or_default() {
                Size::Byte => 0x38,
                Size::Word | Size::Dword | Size::Qword => 0x39,
                _ => invalid(9),
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits, false)
        }
        (Operand::Segment(m), Operand::Reg(_)) => {
            let opc = match m.address.size().unwrap_or_default() {
                Size::Byte => 0x38,
                Size::Word | Size::Dword | Size::Qword => 0x39,
                _ => invalid(8),
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits, false)
        }
        _ => invalid(7),
    }
}

fn ins_test(ins: &Instruction, bits: u8) -> Vec<u8> {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            if let Size::Dword | Size::Word = srci.size() {
                if let Register::RAX | Register::EAX = dstr {
                    extend_imm(&mut imm, 4);
                    return bs_imm(ins, &[0xA9], &imm, bits, false);
                } else if let Register::AX = dstr {
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0xA9], &imm, bits, false);
                }
            }
            if let Register::AL = dstr {
                if let Size::Byte = srci.size() {
                    return bs_imm(ins, &[0xA8], &imm, bits, false);
                }
            } else if let Register::AX = dstr {
                if let Size::Byte = srci.size() {
                    extend_imm(&mut imm, 2);
                    return bs_imm(ins, &[0xA9], &imm, bits, false);
                }
            }

            let opc = match dstr.size() {
                Size::Byte => 0xF6,
                Size::Dword | Size::Qword | Size::Word => 0xF7,
                _ => invalid(6),
            };
            let mut base = gen_base(ins, &[opc], bits, false);
            base.push(modrm::gen_modrm(ins, Some(0), None, false));
            extend_imm(&mut imm, 1);
            base.extend(imm);
            base
        }
        (Operand::Segment(dsts), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dsts.address.size().unwrap_or_default() {
                Size::Byte => 0xF6,
                Size::Qword | Size::Word | Size::Dword => 0xF7,
                _ => invalid(5),
            };
            if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dsts.address.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 2);
            } else if let (Size::Byte, Size::Dword | Size::Qword) =
                (srci.size(), dsts.address.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 4);
            } else if srci.size() != Size::Byte {
                extend_imm(&mut imm, 4);
            }

            gen_ins(ins, &[opc], (true, Some(0), None), Some(imm), bits, false)
        }
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let mut imm = srci.split_into_bytes();
            let opc = match dstm.size().unwrap_or_default() {
                Size::Byte => 0xF6,
                Size::Qword | Size::Word | Size::Dword => 0xF7,
                _ => invalid(4),
            };
            if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 2);
            } else if let (Size::Byte, Size::Dword | Size::Qword) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                extend_imm(&mut imm, 4);
            } else if srci.size() != Size::Byte {
                extend_imm(&mut imm, 4);
            }

            gen_ins(ins, &[opc], (true, Some(0), None), Some(imm), bits, false)
        }
        (Operand::Reg(_) | Operand::Mem(_) | Operand::Segment(_), Operand::Reg(_)) => {
            let opc = match dst.size() {
                Size::Byte => 0x84,
                Size::Word | Size::Dword | Size::Qword => 0x85,
                _ => invalid(3),
            };
            gen_ins(ins, &[opc], (true, None, None), None, bits, false)
        }
        _ => invalid(2),
    }
}

fn ins_imul(ins: &Instruction, bits: u8) -> Vec<u8> {
    match ins.src() {
        None => {
            let opc = match ins.dst().unwrap().size() {
                Size::Byte => &[0xF6],
                _ => &[0xF7],
            };
            GenAPI::new()
                .opcode(opc)
                .modrm(true, Some(5), None)
                .rex(true)
                // this is just temporary solution
                .fixed_size(ins.size())
                .assemble(ins, bits)
        }
        Some(_) => match ins.oprs.get(2) {
            Some(Operand::Imm(imm)) => {
                let (opc, size) = match imm.size() {
                    Size::Byte => (0x6B, 1),
                    Size::Word => (0x69, 2),
                    _ => (0x69, 4),
                };
                let mut imm_b = imm.split_into_bytes();
                extend_imm(&mut imm_b, size);
                let (dst, src) = if let (Some(Operand::Reg(r)), Some(Operand::Reg(r1))) =
                    (ins.dst(), ins.src())
                {
                    (Some(r.to_byte()), Some(r1.to_byte()))
                } else {
                    (None, None)
                };
                gen_ins(ins, &[opc], (true, dst, src), Some(imm_b), bits, false)
            }
            _ => gen_ins(ins, &[0x0F, 0xAF], (true, None, None), None, bits, false),
        },
    }
}

// opc[0] = r/m8, 1
// opc[1] = r/m8, cl
// opc[2] = r/m8, imm8
// opc[3] = r/m16/32/64, 1
// opc[4] = r/m16/32/64, cl
// opc[5] = r/m16/32/64, imm8
fn ins_shllike(ins: &Instruction, opc: &[u8; 6], ovr: u8, bits: u8) -> Vec<u8> {
    let mut api = GenAPI::new().modrm(true, Some(ovr), None).rex(true);
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();
    let (opcd, _) = match src {
        Operand::Reg(Register::CL) => match dst.size() {
            Size::Byte => (opc[1], None),
            Size::Word | Size::Dword | Size::Qword => (opc[4], None),
            _ => panic!("CL failure"),
        },
        Operand::Imm(imm) => {
            if imm == &Number::uint64(1) {
                match dst.size() {
                    Size::Byte => (opc[0], None),
                    _ => (opc[3], None),
                }
            } else {
                api = api.imm_atindex(1, 1);
                match dst.size() {
                    Size::Byte => (opc[2], Some(imm.split_into_bytes())),
                    _ => (opc[5], Some(imm.split_into_bytes())),
                }
            }
        }
        _ => panic!("Other {:?}", src),
    };
    api = api.opcode(&[opcd]);
    api.assemble(ins, bits)
}

fn ins_inclike(ins: &Instruction, opc: &[u8; 2], ovr: u8, bits: u8) -> Vec<u8> {
    let opc = match ins.dst().unwrap().size() {
        Size::Byte => opc[0],
        _ => opc[1],
    };
    gen_ins(ins, &[opc], (true, Some(ovr), None), None, bits, false)
}

fn ins_lea(ins: &Instruction, bits: u8) -> (Vec<u8>, Option<Relocation>) {
    let mut base = gen_base(ins, &[0x8D], bits, false);
    let modrm = if let Operand::Reg(r) = ins.dst().unwrap() {
        0b100 + (r.to_byte() << 3)
    } else {
        0
    };
    base.push(modrm);
    base.push(0x25);
    let symbol = match ins.src().unwrap() {
        Operand::SymbolRef(s) => s,
        _ => invalid(1),
    };
    let blen = base.len();
    base.extend([0x00; 4]);
    (
        base,
        Some(Relocation {
            rtype: RType::S32,
            symbol: Cow::Owned(symbol),
            offset: blen as u64,
            addend: 0,
            size: 4,
            catg: RCategory::Lea,
        }),
    )
}

// opc = opcode ONLY for rel32
// why? because i'm too lazy to implement other rel's
//
// opc[0] = rel32
// opc[1] = r/m
fn ins_jmplike(
    ins: &Instruction,
    opc: [Vec<u8>; 2],
    addt: u8,
    bits: u8,
) -> (Vec<u8>, Option<Relocation>) {
    match ins.dst().unwrap() {
        Operand::SymbolRef(s) => {
            let rel = Relocation {
                rtype: RType::PCRel32,
                symbol: Cow::Owned(s),
                addend: -4,
                offset: opc[0].len() as u64,
                size: 4,
                catg: RCategory::Jump,
            };
            let mut opc = opc[0].clone();
            opc.extend([0; 4]);
            (opc, Some(rel))
        }
        Operand::Reg(_) | Operand::Mem(_) => (
            GenAPI::new()
                .opcode(&opc[1])
                .modrm(true, Some(addt), None)
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        _ => invalid(0),
    }
}

fn ins_divmul(ins: &Instruction, ovr: u8, bits: u8) -> Vec<u8> {
    let opc = match ins.dst().unwrap().size() {
        Size::Byte => [0xF6],
        _ => [0xF7],
    };
    GenAPI::new()
        .opcode(&opc)
        .modrm(true, Some(ovr), None)
        .assemble(ins, bits)
}

// ==============================
// Utils

pub fn bs_imm(ins: &Instruction, opc: &[u8], imm: &[u8], bits: u8, rev: bool) -> Vec<u8> {
    let mut b = gen_base(ins, opc, bits, rev);
    b.extend(imm);
    b
}

pub fn extend_imm(imm: &mut Vec<u8>, size: u8) {
    let size = size as usize;
    while imm.len() < size {
        imm.push(0)
    }
}

pub fn gen_ins_wpref(
    ins: &Instruction,
    opc: &[u8],
    modrm: (bool, Option<u8>, Option<u8>),
    imm: Option<Vec<u8>>,
    pref: u8,
    bits: u8,
    rev: bool,
) -> Vec<u8> {
    let mut base = vec![pref];
    let gbase = gen_base(ins, opc, bits, rev);
    if gbase[0] == 0x66 {
        base = vec![0x66, pref];
        base.extend(&gbase[1..]);
    } else {
        base.extend(gbase);
    }
    if modrm.0 {
        base.push(modrm::gen_modrm(ins, modrm.1, modrm.2, rev));

        if let Some(dst) = ins.dst() {
            if let Some(sib) = sib::gen_sib(dst) {
                base.push(sib);
            }
        }
        if let Some(src) = ins.src() {
            if let Some(sib) = sib::gen_sib(src) {
                base.push(sib);
            }
        }
    }

    if let Some(dst) = ins.dst() {
        if let Some(disp) = disp::gen_disp(dst) {
            base.extend(disp);
        }
    }
    if let Some(src) = ins.src() {
        if let Some(disp) = disp::gen_disp(src) {
            base.extend(disp);
        }
    }
    if let Some(imm) = imm {
        base.extend(imm);
    }
    base
}
#[allow(clippy::too_many_arguments)]
pub fn vex_gen_ins_norm(
    ins: &Instruction,
    opc: &[u8],
    modrm: (bool, Option<u8>),
    imm: Option<Vec<u8>>,
    modrm_reg_is_dst: bool,
    pp: u8,
    map_select: u8,
    vex_we: bool,
    dst: Option<&Operand>,
    src: Option<&Operand>,
    ssrc: Option<&Operand>,
) -> Vec<u8> {
    let mut base = vex::gen_vex_norm(
        ins,
        pp,
        map_select,
        modrm_reg_is_dst,
        vex_we,
        dst,
        src,
        ssrc,
    )
    .unwrap_or_default();
    base.extend(opc);
    if modrm.0 {
        base.push(vex::vex_modrm_norm(
            ins,
            modrm.1,
            None,
            modrm_reg_is_dst,
            dst,
            ssrc,
        ));
        if let Some(dst) = ins.dst() {
            if let Some(sib) = sib::gen_sib(dst) {
                base.push(sib);
            }
        }
        if let Some(src) = ins.src() {
            if let Some(sib) = sib::gen_sib(src) {
                base.push(sib);
            }
        }
        if let Some(ssrc) = ins.src2() {
            if let Some(sib) = sib::gen_sib(ssrc) {
                base.push(sib);
            }
        }
    }
    if let Some(dst) = ins.dst() {
        if let Some(disp) = disp::gen_disp(dst) {
            base.extend(disp);
        }
    }
    if let Some(src) = ins.src() {
        if let Some(disp) = disp::gen_disp(src) {
            base.extend(disp);
        }
    }
    if let Some(ssrc) = ins.src2() {
        if let Some(disp) = disp::gen_disp(ssrc) {
            base.extend(disp);
        }
    }
    if let Some(imm) = imm {
        base.extend(imm);
    }

    base
}

#[allow(clippy::too_many_arguments)]
pub fn vex_gen_ins(
    ins: &Instruction,
    opc: &[u8],
    modrm: (bool, Option<u8>),
    imm: Option<Vec<u8>>,
    modrm_reg_is_dst: bool,
    pp: u8,
    map_select: u8,
    vex_we: bool,
) -> Vec<u8> {
    let mut base = vex::gen_vex(ins, pp, map_select, modrm_reg_is_dst, vex_we).unwrap_or_default();
    base.extend(opc);
    if modrm.0 {
        base.push(if ins.src2().is_some() {
            if let Operand::Imm(_) = ins.src2().unwrap() {
                modrm::gen_modrm(ins, modrm.1, None, modrm_reg_is_dst)
            } else {
                vex::vex_modrm(ins, modrm.1, None, modrm_reg_is_dst)
            }
        } else {
            modrm::gen_modrm(ins, modrm.1, None, modrm_reg_is_dst)
        });
        if let Some(dst) = ins.dst() {
            if let Some(sib) = sib::gen_sib(dst) {
                base.push(sib);
            }
        }
        if let Some(src) = ins.src() {
            if let Some(sib) = sib::gen_sib(src) {
                base.push(sib);
            }
        }
        if let Some(ssrc) = ins.src2() {
            if let Some(sib) = sib::gen_sib(ssrc) {
                base.push(sib);
            }
        }
    }
    if let Some(dst) = ins.dst() {
        if let Some(disp) = disp::gen_disp(dst) {
            base.extend(disp);
        }
    }
    if let Some(src) = ins.src() {
        if let Some(disp) = disp::gen_disp(src) {
            base.extend(disp);
        }
    }
    if let Some(ssrc) = ins.src2() {
        if let Some(disp) = disp::gen_disp(ssrc) {
            base.extend(disp);
        }
    }
    if let Some(imm) = imm {
        base.extend(imm);
    }

    base
}

pub fn gen_ins(
    ins: &Instruction,
    opc: &[u8],
    modrm: (bool, Option<u8>, Option<u8>),
    imm: Option<Vec<u8>>,
    bits: u8,
    rev: bool,
) -> Vec<u8> {
    let mut base = gen_base(ins, opc, bits, rev);
    if modrm.0 {
        base.push(modrm::gen_modrm(ins, modrm.1, modrm.2, rev));

        if let Some(dst) = ins.dst() {
            if let Some(sib) = sib::gen_sib(dst) {
                base.push(sib);
            }
        }
        if let Some(src) = ins.src() {
            if let Some(sib) = sib::gen_sib(src) {
                base.push(sib);
            }
        }
    }

    if let Some(dst) = ins.dst() {
        if let Some(disp) = disp::gen_disp(dst) {
            base.extend(disp);
        }
    }
    if let Some(src) = ins.src() {
        if let Some(disp) = disp::gen_disp(src) {
            base.extend(disp);
        }
    }
    if let Some(imm) = imm {
        base.extend(imm);
    }
    base
}

pub fn gen_base(ins: &Instruction, opc: &[u8], bits: u8, rev: bool) -> Vec<u8> {
    // how does this even work? (probably doesn't)
    let (rex_bool, rex) = if bits == 64 {
        if let Some(rex) = rex::gen_rex(ins, rev) {
            (rex & 0x08 == 8, Some(rex))
        } else {
            (ins.size() == Size::Qword || ins.size() == Size::Any, None)
        }
    } else {
        (false, None)
    };

    // for instructions that have opcode starting with 0x66 (SSE)
    let mut opcode_start = 0;

    let mut used_66 = ins.which_variant() == IVariant::MMX;

    let mut size_ovr = if let Some(dst) = ins.dst() {
        if opc[0] == 0x66 {
            opcode_start = 1;
            vec![0x66]
        } else if let Some(s) = gen_size_ovr(ins, dst, bits, rex_bool) {
            if !used_66 {
                used_66 = s == 0x66;
                vec![s]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if let Some(src) = ins.src() {
        if let Some(s) = gen_size_ovr(ins, src, bits, rex_bool) {
            if !used_66 && !s == 0x66 {
                size_ovr.push(s);
            }
        }
    }
    let mut base = size_ovr;

    if let Some(s) = gen_segm_pref(ins) {
        base.push(s);
    }

    if let Some(rex) = rex {
        base.push(rex);
    }

    base.extend(&opc[opcode_start..]);
    base
}

fn gen_size_ovr(ins: &Instruction, op: &Operand, bits: u8, rexw: bool) -> Option<u8> {
    let (size, is_mem) = match op {
        Operand::Reg(r) => (r.size(), false),
        Operand::CtrReg(r) => (r.size(), false),
        Operand::Mem(m) => (m.size().unwrap_or_default(), false),
        Operand::Segment(s) => (s.address.size().unwrap_or_default(), true),
        _ => return None,
    };
    if size == Size::Byte || size == Size::Xword {
        return None;
    }
    match bits {
        32 => match (size, is_mem) {
            (Size::Word, false) => Some(0x66),
            (Size::Word, true) => Some(0x67),
            (Size::Dword, _) => None,
            _ => inv_osop(&format!("{:?}", op)),
        },
        16 => match (size, is_mem) {
            (Size::Word, _) => None,
            (Size::Dword, true) => Some(0x67),
            (Size::Dword, false) => Some(0x66),
            _ => inv_osop(&format!("{:?}", op)),
        },
        64 => match (size, is_mem) {
            (Size::Qword, false) => {
                if ins.mnem.defaults_to_64bit() || rexw || ins.uses_cr() || ins.uses_dr() {
                    None
                } else {
                    Some(0x66)
                }
            }
            (Size::Dword, false) | (Size::Qword, true) => None,
            (Size::Word, false) => Some(0x66),
            (Size::Word, true) => Some(0x67),
            (Size::Dword, true) => Some(0x67),
            _ => inv_osop(&format!("{:?}", op)),
        },
        _ => None,
    }
}
fn gen_segm_pref(ins: &Instruction) -> Option<u8> {
    if let Some(d) = ins.dst() {
        if let Some(s) = gen_segm_pref_op(d) {
            return Some(s);
        }
    }
    if let Some(d) = ins.src() {
        if let Some(s) = gen_segm_pref_op(d) {
            return Some(s);
        }
    }
    None
}

fn gen_segm_pref_op(op: &Operand) -> Option<u8> {
    if let Operand::Segment(s) = op {
        match s.segment {
            Register::CS => Some(0x2E),
            Register::SS => Some(0x36),
            Register::DS => Some(0x3E),
            Register::ES => Some(0x26),
            Register::FS => Some(0x64),
            Register::GS => Some(0x65),
            _ => None,
        }
    } else {
        None
    }
}

fn inv_osop(s: &str) -> ! {
    panic!("comp.rs:gen_size_ovr+1 {}", s)
}

fn invalid(ctx: i32) -> ! {
    panic!("Unexpected thing that should not happen - code {ctx}")
}
