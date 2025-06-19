// rasmx86_64 - src/core/comp.rs
// -----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    core::api::*,
    shr::{
        ast::{IVariant, Instruction, Label, Operand},
        ins::Mnemonic as Ins,
        num::Number,
        reg::{Purpose as RPurpose, Register},
        reloc::RelType,
        reloc::Relocation,
        segment::Segment,
        size::Size,
        symbol::{Symbol, SymbolType, Visibility},
    },
};

use OpOrd::*;

#[inline]
pub fn make_globals(symbols: &mut [Symbol], globals: &[String]) {
    for s in symbols {
        for g in globals {
            if s.name == g {
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
            name: extern_,
            offset: 0,
            size: 0,
            sindex: 0,
            stype: SymbolType::NoType,
            visibility: Visibility::Global,
            is_extern: true,
        });
    }
    symbols
}

pub fn compile_label(lbl: &Label, offset: usize) -> (Vec<u8>, Vec<Relocation>) {
    let mut bytes = Vec::new();
    let mut reallocs = Vec::new();
    let lbl_bits = lbl.bits;
    let lbl_align = lbl.align;
    for ins in &lbl.inst {
        let res = compile_instruction(ins, lbl_bits);
        // we do not want situation, where label is entry and we place padding before it -
        // preventing UB
        if offset != 0 && lbl_align != 0 {
            let align = lbl_align as usize;
            let mut padding = align - (offset % align);
            while padding > 0 {
                bytes.push(0x0);
                padding -= 1;
            }
        }

        if let Some(mut rl) = res.1 {
            rl.offset += bytes.len() as u32;
            reallocs.push(rl);
        }
        bytes.extend(res.0);
    }
    (bytes, reallocs)
}

pub fn compile_instruction(ins: &'_ Instruction, bits: u8) -> (Vec<u8>, Option<Relocation<'_>>) {
    match ins.mnem {
        Ins::IN => (ins_in(ins, bits), None),
        Ins::OUT => (ins_out(ins, bits), None),

        Ins::BYTE | Ins::BYTELE | Ins::BYTEBE => (
            GenAPI::new()
                .opcode(&[])
                .imm_atindex(0, 1)
                .fixed_size(Size::Byte)
                .assemble(ins, bits),
            None,
        ),
        Ins::WORD => (
            GenAPI::new()
                .opcode(&[])
                .imm_atindex(0, 2)
                .fixed_size(Size::Byte)
                .imm_is_be(false)
                .assemble(ins, bits),
            None,
        ),
        Ins::WORDLE | Ins::WORDBE => (
            GenAPI::new()
                .opcode(&[])
                .imm_atindex(0, 2)
                .fixed_size(Size::Byte)
                .imm_is_be(ins.mnem != Ins::WORDLE)
                .assemble(ins, bits),
            None,
        ),
        Ins::DWORD => (
            GenAPI::new()
                .opcode(&[])
                .imm_atindex(0, 4)
                .fixed_size(Size::Byte)
                .imm_is_be(false)
                .assemble(ins, bits),
            None,
        ),
        Ins::DWORDLE | Ins::DWORDBE => (
            GenAPI::new()
                .opcode(&[])
                .imm_atindex(0, 4)
                .fixed_size(Size::Byte)
                .imm_is_be(ins.mnem != Ins::DWORDLE)
                .assemble(ins, bits),
            None,
        ),
        Ins::QWORD => (
            GenAPI::new()
                .opcode(&[])
                .imm_atindex(0, 8)
                .fixed_size(Size::Byte)
                .imm_is_be(false)
                .assemble(ins, bits),
            None,
        ),
        Ins::QWORDBE | Ins::QWORDLE => (
            GenAPI::new()
                .opcode(&[])
                .imm_atindex(0, 8)
                .fixed_size(Size::Byte)
                .imm_is_be(ins.mnem != Ins::QWORDLE)
                .assemble(ins, bits),
            None,
        ),
        Ins::STRZ | Ins::ASCIIZ => (ins_str(ins), None),
        Ins::EMPTY => (ins_empty(ins), None),

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
            GenAPI::new()
                .opcode(&[0xEB])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VMOVD | Ins::VMOVQ => {
            let mut api = GenAPI::new().modrm(true, None, None).vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vex_we(ins.mnem == Ins::VMOVQ),
            );
            if let Some(Operand::Reg(r)) = ins.dst() {
                if r.size() != Size::Xword {
                    api = api.opcode(&[0x7E]).ord(&[MODRM_RM, MODRM_REG]);
                } else {
                    api = api.opcode(&[0x6E]);
                }
            } else if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x7E]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x6E]);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPAND => (
            GenAPI::new()
                .opcode(&[0xDB])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPXOR => (
            GenAPI::new()
                .opcode(&[0xEF])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPADDB => (
            GenAPI::new()
                .opcode(&[0xFC])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPADDW => (
            GenAPI::new()
                .opcode(&[0xFD])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPADDD => (
            GenAPI::new()
                .opcode(&[0xFE])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPADDQ => (
            GenAPI::new()
                .opcode(&[0xD4])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSUBB => (
            GenAPI::new()
                .opcode(&[0xF8])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSUBW => (
            GenAPI::new()
                .opcode(&[0xF9])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSUBD => (
            GenAPI::new()
                .opcode(&[0xFA])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSUBQ => (
            GenAPI::new()
                .opcode(&[0xFB])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPANDN => (
            GenAPI::new()
                .opcode(&[0xDF])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSLLW => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.src2() {
                api = api
                    .opcode(&[0x71])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(6), None);
            } else {
                api = api
                    .opcode(&[0xF1])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None, None);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPSLLD => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.src2() {
                api = api
                    .opcode(&[0x72])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(6), None);
            } else {
                api = api
                    .opcode(&[0xF2])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None, None);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPSLLQ => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.src2() {
                api = api
                    .opcode(&[0x73])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(6), None);
            } else {
                api = api
                    .opcode(&[0xF3])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None, None);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPSRLW => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.src2() {
                api = api
                    .opcode(&[0x71])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(2), None);
            } else {
                api = api
                    .opcode(&[0xD1])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None, None);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPSRLD => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.src2() {
                api = api
                    .opcode(&[0x72])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(2), None);
            } else {
                api = api
                    .opcode(&[0xD2])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None, None);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPSRLQ => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.src2() {
                api = api
                    .opcode(&[0x73])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(2), None);
            } else {
                api = api
                    .opcode(&[0xD3])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None, None);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPSRAW => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.src2() {
                api = api
                    .opcode(&[0x71])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(4), None);
            } else {
                api = api
                    .opcode(&[0xE1])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None, None);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPSRAD => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.src2() {
                api = api
                    .opcode(&[0x72])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(4), None);
            } else {
                api = api
                    .opcode(&[0xE2])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None, None);
            }
            (api.assemble(ins, bits), None)
        }
        Ins::VPSUBSB => (
            GenAPI::new()
                .opcode(&[0xE8])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSUBSW => (
            GenAPI::new()
                .opcode(&[0xE9])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPADDSB => (
            GenAPI::new()
                .opcode(&[0xEC])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPADDSW => (
            GenAPI::new()
                .opcode(&[0xED])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMULHW => (
            GenAPI::new()
                .opcode(&[0xE5])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMULLW => (
            GenAPI::new()
                .opcode(&[0xD5])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        // part 2
        Ins::VPADDUSB => (
            GenAPI::new()
                .opcode(&[0xDC])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPADDUSW => (
            GenAPI::new()
                .opcode(&[0xDD])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSUBUSB => (
            GenAPI::new()
                .opcode(&[0xD8])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPSUBUSW => (
            GenAPI::new()
                .opcode(&[0xD9])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPMADDWD => (
            GenAPI::new()
                .opcode(&[0xF5])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPEQB => (
            GenAPI::new()
                .opcode(&[0x74])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPEQW => (
            GenAPI::new()
                .opcode(&[0x75])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPEQD => (
            GenAPI::new()
                .opcode(&[0x76])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPGTB => (
            GenAPI::new()
                .opcode(&[0x64])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPGTW => (
            GenAPI::new()
                .opcode(&[0x65])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPCMPGTD => (
            GenAPI::new()
                .opcode(&[0x66])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPACKUSWB => (
            GenAPI::new()
                .opcode(&[0x67])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPACKSSWB => (
            GenAPI::new()
                .opcode(&[0x63])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPACKSSDW => (
            GenAPI::new()
                .opcode(&[0x6B])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPUNPCKLBW => (
            GenAPI::new()
                .opcode(&[0x60])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPUNPCKLWD => (
            GenAPI::new()
                .opcode(&[0x61])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPUNPCKLDQ => (
            GenAPI::new()
                .opcode(&[0x62])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPUNPCKHBW => (
            GenAPI::new()
                .opcode(&[0x68])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPUNPCKHWD => (
            GenAPI::new()
                .opcode(&[0x69])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPUNPCKHDQ => (
            GenAPI::new()
                .opcode(&[0x6A])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
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
            GenAPI::new()
                .opcode(&[0xE0])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPAVGW => (
            GenAPI::new()
                .opcode(&[0xE3])
                .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPHADDW => (
            GenAPI::new()
                .opcode(&[0x01])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPHADDD => (
            GenAPI::new()
                .opcode(&[0x02])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPHSUBW => (
            GenAPI::new()
                .opcode(&[0x05])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VPHSUBD => (
            GenAPI::new()
                .opcode(&[0x06])
                .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
            None,
        ),
        Ins::VZEROUPPER => (vec![0xC5, 0xF8, 0x77], None),
        Ins::VZEROALL => (vec![0xC5, 0xFC, 0x77], None),
        Ins::VPALIGNR => (
            GenAPI::new()
                .opcode(&[0x0F])
                .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
                .modrm(true, None, None)
                .imm_atindex(3, 1)
                .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                .assemble(ins, bits),
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
            GenAPI::new()
                .opcode(&[0x19])
                .vex(
                    VexDetails::new()
                        .map_select(0x3A)
                        .pp(0x66)
                        .vex_we(false)
                        .vlength(Some(true)),
                )
                .modrm(true, None, None)
                .imm_atindex(2, 1)
                .ord(&[MODRM_RM, MODRM_REG])
                .assemble(ins, bits),
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
        Ins::CVTSD2SI => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2D])
                .modrm(true, None, None)
                .prefix(0xF2)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),
        Ins::CVTSI2SD => (
            GenAPI::new()
                .opcode(&[0x0F, 0x2A])
                .modrm(true, None, None)
                .prefix(0xF2)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
            None,
        ),

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
            GenAPI::new()
                .opcode(&[0x0F, 0x2C])
                .modrm(true, None, None)
                .prefix(0x66)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
                .assemble(ins, bits),
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

        // fxd
        Ins::WRGSBASE => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(3), None)
                .prefix(0xF3)
                .rex(true)
                .assemble(ins, bits)
            , None),
        Ins::WRFSBASE => (
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(2), None)
                .prefix(0xF3)
                .rex(true)
                .assemble(ins, bits)
            , None),
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
    let (symbol, reltype, addend) = if let Some(Operand::SymbolRef(str)) = ins.dst() {
        (str, RelType::REL32, 0)
    } else if let Some(Operand::SymbolRefExt(s)) = ins.dst() {
        (&s.symbol, s.reltype, s.addend)
    } else {
        invalid(3923)
    };

    (
        vec![0xC7, 0xF8],
        Some(Relocation {
            reltype,
            symbol,
            addend,
            offset: 2,
            shidx: 0,
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

fn ins_enter(ins: &Instruction, bits: u8) -> Vec<u8> {
    let immw = if let Some(Operand::Imm(w)) = ins.dst() {
        let mut vec = w.split_into_bytes();
        if vec.len() == 1 {
            vec.push(0);
        }
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
    GenAPI::new()
        .opcode(&[0xC8, immw[0], immw[1], immb[0]])
        .assemble(ins, bits)
}

fn ins_bt(ins: &Instruction, opc_noimm: &[u8], opc_imm: &[u8], bits: u8, modrm: u8) -> Vec<u8> {
    let mut api = GenAPI::new().rex(true);
    if let Some(Operand::Imm(_)) = ins.src() {
        api = api
            .opcode(opc_imm)
            .modrm(true, Some(modrm), None)
            .imm_atindex(1, 1);
    } else {
        api = api.opcode(opc_noimm).modrm(true, None, None)
    };
    api.assemble(ins, bits)
}

fn ins_cmovcc(ins: &Instruction, opc: &[u8], bits: u8) -> Vec<u8> {
    GenAPI::new()
        .opcode(opc)
        .modrm(true, None, None)
        .rex(true)
        .ord(&[MODRM_REG, MODRM_RM])
        .assemble(ins, bits)
}

fn ins_pop(ins: &Instruction, bits: u8) -> Vec<u8> {
    match ins.dst().unwrap() {
        Operand::Reg(r) => GenAPI::new()
            .opcode(&[0x58 + r.to_byte()])
            .rex(true)
            .assemble(ins, bits),
        Operand::SegReg(r) => match r {
            Register::DS => vec![0x1F],
            Register::ES => vec![0x07],
            Register::SS => vec![0x17],
            Register::FS => vec![0x0F, 0xA1],
            Register::GS => vec![0x0F, 0xA9],
            Register::CS => vec![0x90],
            _ => invalid(34),
        },
        Operand::Mem(_) | Operand::Segment(_) => GenAPI::new()
            .opcode(&[0x8F])
            .rex(true)
            .modrm(true, None, Some(0))
            .assemble(ins, bits),
        _ => invalid(33),
    }
}

fn ins_push(ins: &Instruction, bits: u8) -> Vec<u8> {
    match ins.dst().unwrap() {
        Operand::Reg(r) => GenAPI::new()
            .opcode(&[0x50 + r.to_byte()])
            .rex(true)
            .assemble(ins, bits),
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
            Size::Byte => GenAPI::new()
                .opcode(&[0x6A])
                .imm_atindex(0, 1)
                .assemble(ins, bits),
            Size::Word | Size::Dword => GenAPI::new()
                .opcode(&[0x68])
                .imm_atindex(0, 4)
                .assemble(ins, bits),
            _ => invalid(31),
        },
        Operand::Mem(_) | Operand::Segment(_) => GenAPI::new()
            .opcode(&[0xFF])
            .modrm(true, Some(6), None)
            .rex(true)
            .assemble(ins, bits),
        _ => invalid(30),
    }
}

fn ins_mov(ins: &Instruction, bits: u8) -> Vec<u8> {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();
    if let Operand::Reg(r) = dst {
        match src {
            Operand::SegReg(_) => GenAPI::new()
                .opcode(&[0x8C])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            Operand::CtrReg(_) => GenAPI::new()
                .opcode(&[0x0F, 0x20])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            Operand::DbgReg(_) => GenAPI::new()
                .opcode(&[0x0F, 0x21])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
            Operand::Imm(_) => {
                let size = dst.size();
                let opc = match size {
                    Size::Byte => 0xB0 + r.to_byte(),
                    Size::Word | Size::Dword | Size::Qword => 0xB8 + r.to_byte(),
                    _ => invalid(29),
                };
                let size = if size == Size::Qword { 4 } else { size.into() };
                GenAPI::new()
                    .opcode(&[opc])
                    .imm_atindex(1, size as u16)
                    .assemble(ins, bits)
            }
            Operand::Reg(_) => {
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
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, None, None)
                    .rex(true)
                    .assemble(ins, bits)
            }
            Operand::Mem(_) | Operand::Segment(_) => {
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
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, None, None)
                    .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                    .rex(true)
                    .assemble(ins, bits)
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
            let imm = srci.split_into_bytes();
            if let Size::Dword | Size::Word = srci.size() {
                if let Register::RAX | Register::EAX = dstr {
                    return GenAPI::new()
                        .opcode(&[opc[1]])
                        .imm_atindex(1, 4)
                        .rex(true)
                        .assemble(ins, bits);
                } else if let Register::AX = dstr {
                    return GenAPI::new()
                        .opcode(&[opc[1]])
                        .imm_atindex(1, 2)
                        .rex(true)
                        .assemble(ins, bits);
                }
            }
            if let Register::AL = dstr {
                return GenAPI::new()
                    .opcode(&[opc[0]])
                    .imm_atindex(1, 1)
                    .rex(true)
                    .assemble(ins, bits);
            } else if let Register::AX = dstr {
                return GenAPI::new()
                    .opcode(&[opc[1]])
                    .imm_atindex(1, 1)
                    .rex(true)
                    .assemble(ins, bits);
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
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(ovrreg), None)
                .rex(true)
                .imm_atindex(1, 1)
                .assemble(ins, bits)
        }
        (
            Operand::Mem(dstm)
            | Operand::Segment(Segment {
                segment: _,
                address: dstm,
            }),
            Operand::Imm(srci),
        ) => {
            let imm = srci.split_into_bytes();
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
            let size = if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                2
            } else if let (Size::Byte, Size::Dword) = (srci.size(), dstm.size().unwrap_or_default())
            {
                4
            } else if srci.size() != Size::Byte {
                4
            } else {
                1
            };

            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(ovrreg), None)
                .rex(true)
                .imm_atindex(1, size)
                .assemble(ins, bits)
        }
        (Operand::Reg(r), Operand::Segment(_) | Operand::Mem(_) | Operand::Reg(_)) => {
            let opc = match r.size() {
                Size::Byte => opc[7],
                Size::Word | Size::Dword | Size::Qword => opc[6],
                _ => invalid(17),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits)
        }
        (Operand::Segment(m), Operand::Reg(_)) => {
            let opc = match m.address.size().unwrap_or_default() {
                Size::Byte => opc[7],
                Size::Word | Size::Dword | Size::Qword => opc[6],
                _ => invalid(16),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits)
        }
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size().unwrap_or_default() {
                Size::Byte => opc[7],
                Size::Word | Size::Dword | Size::Qword => opc[6],
                _ => invalid(15),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits)
        }
        _ => invalid(14),
    }
}

fn ins_cmp(ins: &Instruction, bits: u8) -> Vec<u8> {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let imm = srci.split_into_bytes();
            if let Size::Dword | Size::Word = srci.size() {
                if let Register::RAX | Register::EAX = dstr {
                    return GenAPI::new()
                        .opcode(&[0x3D])
                        .imm_atindex(1, 4)
                        .rex(true)
                        .assemble(ins, bits);
                } else if let Register::AX = dstr {
                    return GenAPI::new()
                        .opcode(&[0x3D])
                        .imm_atindex(1, 2)
                        .rex(true)
                        .assemble(ins, bits);
                }
            }
            if let Register::AL = dstr {
                return GenAPI::new()
                    .opcode(&[0x3C])
                    .imm_atindex(1, 1)
                    .rex(true)
                    .assemble(ins, bits);
            } else if let Register::AX = dstr {
                return GenAPI::new()
                    .opcode(&[0x3D])
                    .imm_atindex(1, 2)
                    .rex(true)
                    .assemble(ins, bits);
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
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(7), None)
                .rex(true)
                .imm_atindex(1, 1)
                .assemble(ins, bits)
        }
        (Operand::Segment(dstm), Operand::Imm(srci)) => {
            let imm = srci.split_into_bytes();
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
            let size = if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dstm.address.size().unwrap_or_default())
            {
                2
            } else if let (Size::Byte, Size::Dword | Size::Qword) =
                (srci.size(), dstm.address.size().unwrap_or_default())
            {
                4
            } else if srci.size() != Size::Byte {
                4
            } else {
                1
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(7), None)
                .rex(true)
                .imm_atindex(1, size)
                .assemble(ins, bits)
        }
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let imm = srci.split_into_bytes();
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
            let size = if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                2
            } else if let (Size::Byte, Size::Dword | Size::Qword) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                4
            } else if srci.size() != Size::Byte {
                4
            } else {
                1
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(7), None)
                .rex(true)
                .imm_atindex(1, size)
                .assemble(ins, bits)
        }
        (Operand::Reg(r), Operand::Segment(_) | Operand::Mem(_) | Operand::Reg(_)) => {
            let opc = match r.size() {
                Size::Byte => 0x3A,
                Size::Word | Size::Dword | Size::Qword => 0x3B,
                _ => invalid(10),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits)
        }
        (Operand::Mem(m), Operand::Reg(_)) => {
            let opc = match m.size().unwrap_or_default() {
                Size::Byte => 0x38,
                Size::Word | Size::Dword | Size::Qword => 0x39,
                _ => invalid(9),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits)
        }
        (Operand::Segment(m), Operand::Reg(_)) => {
            let opc = match m.address.size().unwrap_or_default() {
                Size::Byte => 0x38,
                Size::Word | Size::Dword | Size::Qword => 0x39,
                _ => invalid(8),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits)
        }
        _ => invalid(7),
    }
}

fn ins_test(ins: &Instruction, bits: u8) -> Vec<u8> {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            if let Size::Dword | Size::Word = srci.size() {
                if let Register::RAX | Register::EAX = dstr {
                    return GenAPI::new()
                        .opcode(&[0xA9])
                        .imm_atindex(1, 4)
                        .rex(true)
                        .assemble(ins, bits);
                } else if let Register::AX = dstr {
                    return GenAPI::new()
                        .opcode(&[0xA9])
                        .imm_atindex(1, 2)
                        .rex(true)
                        .assemble(ins, bits);
                }
            }
            if let Register::AL = dstr {
                return GenAPI::new()
                    .opcode(&[0xA8])
                    .imm_atindex(1, 1)
                    .rex(true)
                    .assemble(ins, bits);
            } else if let Register::AX = dstr {
                return GenAPI::new()
                    .opcode(&[0xA9])
                    .imm_atindex(1, 2)
                    .rex(true)
                    .assemble(ins, bits);
            }

            let opc = match dstr.size() {
                Size::Byte => 0xF6,
                Size::Dword | Size::Qword | Size::Word => 0xF7,
                _ => invalid(6),
            };
            let size = match ins.size() {
                Size::Byte => 1,
                Size::Word => 2,
                Size::Dword | Size::Qword => 4,
                _ => 1,
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(0), None)
                .imm_atindex(1, size)
                .rex(true)
                .assemble(ins, bits)
        }
        (Operand::Segment(dsts), Operand::Imm(srci)) => {
            let opc = match dsts.address.size().unwrap_or_default() {
                Size::Byte => 0xF6,
                Size::Qword | Size::Word | Size::Dword => 0xF7,
                _ => invalid(5),
            };
            let size = if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dsts.address.size().unwrap_or_default())
            {
                2
            } else if let (Size::Byte, Size::Dword | Size::Qword) =
                (srci.size(), dsts.address.size().unwrap_or_default())
            {
                4
            } else if srci.size() != Size::Byte {
                4
            } else {
                1
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(0), None)
                .rex(true)
                .imm_atindex(1, size)
                .assemble(ins, bits)
        }
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
            let opc = match dstm.size().unwrap_or_default() {
                Size::Byte => 0xF6,
                Size::Qword | Size::Word | Size::Dword => 0xF7,
                _ => invalid(4),
            };
            let size = if let (Size::Word | Size::Byte, Size::Word) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                2
            } else if let (Size::Byte, Size::Dword | Size::Qword) =
                (srci.size(), dstm.size().unwrap_or_default())
            {
                4
            } else if srci.size() != Size::Byte {
                4
            } else {
                1
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(0), None)
                .rex(true)
                .imm_atindex(1, size)
                .assemble(ins, bits)
        }
        (Operand::Reg(_) | Operand::Mem(_) | Operand::Segment(_), Operand::Reg(_)) => {
            let opc = match dst.size() {
                Size::Byte => 0x84,
                Size::Word | Size::Dword | Size::Qword => 0x85,
                _ => invalid(3),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits)
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
                .assemble(ins, bits)
        }
        Some(_) => match ins.get_opr(2) {
            Some(Operand::Imm(imm)) => {
                let (opc, size) = match imm.size() {
                    Size::Byte => (0x6B, 1),
                    Size::Word => (0x69, 2),
                    _ => (0x69, 4),
                };
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, None, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .rex(true)
                    .imm_atindex(2, size)
                    .assemble(ins, bits)
            }
            _ => GenAPI::new()
                .opcode(&[0x0F, 0xAF])
                .modrm(true, None, None)
                .rex(true)
                .assemble(ins, bits),
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
    let opcd = match ins.dst().unwrap().size() {
        Size::Byte => opc[0],
        _ => opc[1],
    };
    GenAPI::new()
        .opcode(&[opcd])
        .modrm(true, Some(ovr), None)
        .rex(true)
        .assemble(ins, bits)
}

fn ins_lea(ins: &Instruction, bits: u8) -> (Vec<u8>, Option<Relocation>) {
    if ins.src().unwrap().is_mem() {
        return (GenAPI::new()
            .opcode(&[0x8D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .assemble(ins, bits), None)
    }
    let mut base = GenAPI::new()
        .opcode(&[0x8D])
        .modrm(
            true,
            Some(if let Operand::Reg(r) = ins.dst().unwrap() {
                r.to_byte()
            } else {
                0
            }),
            Some(0b100),
        )
        .modrm_mod(0b00)
        .rex(true)
        .assemble(ins, bits);
    base.push(0x25);
    let (symbol, reltype, addend) = match ins.src().unwrap() {
        Operand::SymbolRef(s) => (s, RelType::ABS32, 0),
        Operand::SymbolRefExt(s) => (&s.symbol, s.reltype, s.addend),
        _ => invalid(1),
    };
    let blen = base.len();
    base.extend([0x00; 4]);
    (
        base,
        Some(Relocation {
            reltype,
            symbol,
            offset: blen as u32,
            addend,
            shidx: 0,
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
        Operand::SymbolRefExt(s) => {
            let rel = Relocation {
                reltype: s.reltype,
                symbol: &s.symbol,
                offset: opc[0].len() as u32,
                addend: s.addend,
                shidx: 0,
            };
            let mut opc = opc[0].clone();
            opc.extend([0; 4]);
            (opc, Some(rel))
        }
        Operand::SymbolRef(s) => {
            let rel = Relocation {
                reltype: RelType::REL32,
                symbol: s,
                addend: -4,
                offset: opc[0].len() as u32,
                shidx: 0,
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

fn ins_empty(ins: &Instruction) -> Vec<u8> {
    if let Some(Operand::Imm(n)) = ins.get_opr(0) {
        vec![0x00; n.get_as_u32() as usize]
    } else {
        vec![]
    }
}

fn ins_str(ins: &Instruction) -> Vec<u8> {
    let mut bts = Vec::new();
    if let Some(Operand::String(s)) = ins.dst() {
        bts.extend(s.as_bytes())
    };
    bts.push(0x00);

    bts
}

fn ins_in(ins: &Instruction, bits: u8) -> Vec<u8> {
    if let Operand::Reg(_) = ins.src().unwrap() {
        let sz = ins.dst().unwrap().size();
        if sz == Size::Byte {
            GenAPI::new()
                .opcode(&[0xEC])
                .fixed_size(Size::Byte)
                .assemble(ins, bits)
        } else {
            GenAPI::new()
                .opcode(&[0xED])
                .fixed_size(sz)
                .assemble(ins, bits)
        }
    } else {
        if ins.size() == Size::Byte {
            GenAPI::new()
                .opcode(&[0xE4])
                .imm_atindex(1, 1)
                .assemble(ins, bits)
        } else {
            GenAPI::new()
                .opcode(&[0xE5])
                .imm_atindex(1, 1)
                .assemble(ins, bits)
        }
    }
}

fn ins_out(ins: &Instruction, bits: u8) -> Vec<u8> {
    let sz = ins.src().unwrap().size();
    if let Operand::Reg(_) = ins.dst().unwrap() {
        if sz == Size::Byte {
            GenAPI::new()
                .opcode(&[0xEE])
                .fixed_size(Size::Byte)
                .can_h66(false)
                .assemble(ins, bits)
        } else {
            GenAPI::new()
                .opcode(&[0xEF])
                .fixed_size(sz)
                .assemble(ins, bits)
        }
    } else {
        if sz == Size::Byte {
            GenAPI::new()
                .opcode(&[0xE6])
                .imm_atindex(0, 1)
                .assemble(ins, bits)
        } else {
            GenAPI::new()
                .opcode(&[0xE7])
                .imm_atindex(0, 1)
                .fixed_size(sz)
                .assemble(ins, bits)
        }
    }
}

fn ins_shrtjmp(ins: &Instruction, opc: Vec<u8>) -> (Vec<u8>, Option<Relocation<'_>>) {
    let mut b = [0; 2];
    b[0] = opc[0];
    let (symbol, reltype, addend) = if let Operand::SymbolRefExt(s) = ins.dst().unwrap() {
        (&s.symbol, s.reltype, s.addend)
    } else if let Operand::SymbolRef(r) = ins.dst().unwrap() {
        (r, RelType::REL8, 0)
    } else {
        panic!("Unhandled exception");
    };
    (b.to_vec(), Some(Relocation {
        symbol,
        offset: 1,
        addend: addend + 1,
        shidx: 0,
        reltype,
    }))
}

// ==============================
// Utils

fn invalid(ctx: i32) -> ! {
    panic!("Unexpected thing that should not happen - code {ctx}")
}
