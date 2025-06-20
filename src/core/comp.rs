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
        size::Size,
        symbol::{Symbol, SymbolType, Visibility},
    },
    cli::CLI,
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
    let fn_ptr = if CLI.debug {
        GenAPI::debug_assemble
    } else {
        GenAPI::assemble
    };
    let mut bytes = Vec::new();
    let mut reallocs = Vec::new();
    let lbl_bits = lbl.bits;
    let lbl_align = lbl.align;
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
    for ins in &lbl.inst {
        let res = fn_ptr(&get_genapi(ins, lbl_bits), ins, lbl_bits);
        for mut rl in res.1.into_iter().flatten() {
            rl.offset += bytes.len() as u32;
            reallocs.push(rl);
        }
        bytes.extend(res.0);
    }
    (bytes, reallocs)
}

pub fn get_genapi(ins: &'_ Instruction, bits: u8) -> GenAPI {
    match ins.mnem {
        Ins::IN => ins_in(ins, bits),
        Ins::OUT => ins_out(ins, bits),

        Ins::BYTE | Ins::BYTELE | Ins::BYTEBE => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 1)
            .fixed_size(Size::Byte),
        Ins::WORD => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 2)
            .fixed_size(Size::Byte)
            .imm_is_be(false),
        Ins::WORDLE | Ins::WORDBE => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 2)
            .fixed_size(Size::Byte)
            .imm_is_be(ins.mnem != Ins::WORDLE),
        Ins::DWORD => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 4)
            .fixed_size(Size::Byte)
            .imm_is_be(false),
        Ins::DWORDLE | Ins::DWORDBE => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 4)
            .fixed_size(Size::Byte)
            .imm_is_be(ins.mnem != Ins::DWORDLE),
        Ins::QWORD => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 8)
            .fixed_size(Size::Byte)
            .imm_is_be(false),
        Ins::QWORDBE | Ins::QWORDLE => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 8)
            .fixed_size(Size::Byte)
            .imm_is_be(ins.mnem != Ins::QWORDLE),
        Ins::ASCII | Ins::STRING => GenAPI::new().opcode(&[]).imm_atindex(0, 0),

        //Ins::EMPTY => ins_empty(ins),
        Ins::__LAST => GenAPI::new(),
        Ins::CPUID => GenAPI::new().opcode(&[0x0F, 0xA2]),
        Ins::RET => GenAPI::new().opcode(&[0xC3]),
        Ins::SYSCALL => GenAPI::new().opcode(&[0x0F, 0x05]),
        Ins::PUSH => ins_push(ins, bits),
        Ins::POP => ins_pop(ins, bits),
        Ins::MOV => ins_mov(ins, bits),
        Ins::ADD => add_like_ins(
            ins,
            &[0x04, 0x05, 0x80, 0x81, 0x83, 0x00, 0x01, 0x02, 0x03],
            0,
            bits,
        ),
        Ins::OR => add_like_ins(
            ins,
            &[0x0C, 0x0D, 0x80, 0x81, 0x83, 0x08, 0x09, 0x0A, 0x0B],
            1,
            bits,
        ),
        Ins::AND => add_like_ins(
            ins,
            &[0x24, 0x25, 0x80, 0x81, 0x83, 0x20, 0x21, 0x22, 0x23],
            4,
            bits,
        ),
        Ins::SUB => add_like_ins(
            ins,
            &[0x2C, 0x2D, 0x80, 0x81, 0x83, 0x28, 0x29, 0x2A, 0x2B],
            5,
            bits,
        ),
        Ins::XOR => add_like_ins(
            ins,
            &[0x34, 0x35, 0x80, 0x81, 0x83, 0x30, 0x31, 0x32, 0x33],
            6,
            bits,
        ),
        Ins::SAL | Ins::SHL => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 4, bits),
        Ins::SHR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 5, bits),
        Ins::SAR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 7, bits),
        Ins::TEST => ins_test(ins, bits),
        Ins::INC => ins_inclike(ins, &[0xFE, 0xFF], 0, bits),
        Ins::DEC => ins_inclike(ins, &[0xFE, 0xFF], 1, bits),
        Ins::NOT => ins_inclike(ins, &[0xF6, 0xF7], 2, bits),
        Ins::NEG => ins_inclike(ins, &[0xF6, 0xF7], 3, bits),
        Ins::CMP => ins_cmp(ins, bits),
        Ins::IMUL => ins_imul(ins, bits),
        Ins::DIV => ins_divmul(ins, 6, bits),
        Ins::IDIV => ins_divmul(ins, 7, bits),
        Ins::MUL => ins_divmul(ins, 4, bits),
        Ins::JMP => ins_jmplike(ins, [&[0xE9], &[0xFF], &[0xEB]], 4, bits),
        Ins::CALL => ins_jmplike(ins, [&[0xE8], &[0xFF], &[0xE8]], 2, bits),

        // jcc
        Ins::JA => ins_jmplike(ins, [&[0x0F, 0x87], &[], &[0x77]], 0, bits),
        Ins::JB => ins_jmplike(ins, [&[0x0F, 0x82], &[], &[0x72]], 0, bits),
        Ins::JC => ins_jmplike(ins, [&[0x0F, 0x82], &[], &[0x72]], 0, bits),
        Ins::JO => ins_jmplike(ins, [&[0x0F, 0x80], &[], &[0x70]], 0, bits),
        Ins::JP => ins_jmplike(ins, [&[0x0F, 0x8A], &[], &[0x7A]], 0, bits),
        Ins::JS => ins_jmplike(ins, [&[0x0F, 0x88], &[], &[0x78]], 0, bits),
        Ins::JL => ins_jmplike(ins, [&[0x0F, 0x8C], &[], &[0x7C]], 0, bits),
        Ins::JG => ins_jmplike(ins, [&[0x0F, 0x8F], &[], &[0x7C]], 0, bits),
        Ins::JE | Ins::JZ => ins_jmplike(ins, [&[0x0F, 0x84], &[], &[0x74]], 0, bits),
        Ins::JAE => ins_jmplike(ins, [&[0x0F, 0x83], &[], &[0x73]], 0, bits),
        Ins::JBE => ins_jmplike(ins, [&[0x0F, 0x86], &[], &[0x76]], 0, bits),
        Ins::JNA => ins_jmplike(ins, [&[0x0F, 0x86], &[], &[0x76]], 0, bits),
        Ins::JNB => ins_jmplike(ins, [&[0x0F, 0x83], &[], &[0x73]], 0, bits),
        Ins::JNC => ins_jmplike(ins, [&[0x0F, 0x83], &[], &[0x73]], 0, bits),
        Ins::JNG => ins_jmplike(ins, [&[0x0F, 0x8E], &[], &[0x7E]], 0, bits),
        Ins::JNL => ins_jmplike(ins, [&[0x0F, 0x8D], &[], &[0x7D]], 0, bits),
        Ins::JNO => ins_jmplike(ins, [&[0x0F, 0x81], &[], &[0x71]], 0, bits),
        Ins::JNP => ins_jmplike(ins, [&[0x0F, 0x8B], &[], &[0x7B]], 0, bits),
        Ins::JNS => ins_jmplike(ins, [&[0x0F, 0x89], &[], &[0x79]], 0, bits),
        Ins::JPE => ins_jmplike(ins, [&[0x0F, 0x8A], &[], &[0x7A]], 0, bits),
        Ins::JPO => ins_jmplike(ins, [&[0x0F, 0x8B], &[], &[0x7B]], 0, bits),
        Ins::JNE | Ins::JNZ => ins_jmplike(ins, [&[0xFF, 0x85], &[], &[0x75]], 0, bits),
        Ins::JLE => ins_jmplike(ins, [&[0x0F, 0x8E], &[], &[0x7E]], 0, bits),
        Ins::JGE => ins_jmplike(ins, [&[0x0F, 0x8D], &[], &[0x7D]], 0, bits),
        Ins::JNAE => ins_jmplike(ins, [&[0x0F, 0x82], &[], &[0x72]], 0, bits),
        Ins::JNBE => ins_jmplike(ins, [&[0x0F, 0x87], &[], &[0x77]], 0, bits),
        Ins::JNGE => ins_jmplike(ins, [&[0x0F, 0x8C], &[], &[0x7C]], 0, bits),
        Ins::JNLE => ins_jmplike(ins, [&[0x0F, 0x8F], &[], &[0x7F]], 0, bits),

        Ins::JCXZ => ins_jmplike(ins, [&[], &[], &[0xE3]], 0, bits),
        Ins::JECXZ => ins_jmplike(ins, [&[], &[], &[0xE3]], 0, bits),
        Ins::JRCXZ => ins_jmplike(ins, [&[], &[], &[0xE3]], 0, bits),

        Ins::LEA => ins_lea(ins, bits),

        Ins::NOP => GenAPI::new().opcode(&[0x90]),

        Ins::PUSHF | Ins::PUSHFD | Ins::PUSHFQ => GenAPI::new().opcode(&[0x9C]),
        Ins::POPF | Ins::POPFD | Ins::POPFQ => GenAPI::new().opcode(&[0x9D]),

        Ins::CLFLUSH => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(7), None)
            .rex(true),

        Ins::PAUSE => GenAPI::new().opcode(&[0xF3, 0x90]),
        Ins::MWAIT => GenAPI::new().opcode(&[0x0F, 0x01, 0xC9]),

        Ins::CMOVA => ins_cmovcc(ins, &[0x0F, 0x47], bits),
        Ins::CMOVAE => ins_cmovcc(ins, &[0x0F, 0x43], bits),
        Ins::CMOVB => ins_cmovcc(ins, &[0x0F, 0x42], bits),
        Ins::CMOVBE => ins_cmovcc(ins, &[0x0F, 0x46], bits),
        Ins::CMOVC => ins_cmovcc(ins, &[0x0F, 0x42], bits),
        Ins::CMOVE => ins_cmovcc(ins, &[0x0F, 0x44], bits),
        Ins::CMOVG => ins_cmovcc(ins, &[0x0F, 0x4F], bits),
        Ins::CMOVGE => ins_cmovcc(ins, &[0x0F, 0x4D], bits),
        Ins::CMOVL => ins_cmovcc(ins, &[0x0F, 0x4C], bits),
        Ins::CMOVLE => ins_cmovcc(ins, &[0x0F, 0x4E], bits),
        Ins::CMOVNA => ins_cmovcc(ins, &[0x0F, 0x46], bits),
        Ins::CMOVNB => ins_cmovcc(ins, &[0x0F, 0x43], bits),
        Ins::CMOVNBE => ins_cmovcc(ins, &[0x0F, 0x47], bits),
        Ins::CMOVNC => ins_cmovcc(ins, &[0x0F, 0x43], bits),
        Ins::CMOVNE => ins_cmovcc(ins, &[0x0F, 0x45], bits),
        Ins::CMOVNG => ins_cmovcc(ins, &[0x0F, 0x4E], bits),
        Ins::CMOVNGE => ins_cmovcc(ins, &[0x0F, 0x4C], bits),
        Ins::CMOVNL => ins_cmovcc(ins, &[0x0F, 0x4D], bits),
        Ins::CMOVNLE => ins_cmovcc(ins, &[0x0F, 0x4F], bits),
        Ins::CMOVNAE => ins_cmovcc(ins, &[0x0F, 0x42], bits),
        Ins::CMOVNO => ins_cmovcc(ins, &[0x0F, 0x41], bits),
        Ins::CMOVNP => ins_cmovcc(ins, &[0x0F, 0x4B], bits),
        Ins::CMOVNS => ins_cmovcc(ins, &[0x0F, 0x49], bits),
        Ins::CMOVNZ => ins_cmovcc(ins, &[0x0F, 0x45], bits),
        Ins::CMOVO => ins_cmovcc(ins, &[0x0F, 0x40], bits),
        Ins::CMOVP => ins_cmovcc(ins, &[0x0F, 0x4A], bits),
        Ins::CMOVPO => ins_cmovcc(ins, &[0x0F, 0x4B], bits),
        Ins::CMOVS => ins_cmovcc(ins, &[0x0F, 0x48], bits),
        Ins::CMOVZ => ins_cmovcc(ins, &[0x0F, 0x44], bits),
        Ins::CMOVPE => ins_cmovcc(ins, &[0x0F, 0x4A], bits),

        // SSE
        Ins::MOVSS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true).prefix(0xF3);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVHLPS => GenAPI::new()
            .modrm(true, None, None)
            .rex(true)
            .opcode(&[0x0F, 0x12])
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MOVLHPS => GenAPI::new()
            .modrm(true, None, None)
            .rex(true)
            .opcode(&[0x0F, 0x16])
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MOVAPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x29]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVUPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVLPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x13]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x12]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVHPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x17]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x16]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }

        Ins::ADDPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x58])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ADDSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x58])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::SUBPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x5C])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::SUBSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5C])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MULPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x59])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MULSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x59])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::DIVPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x5E])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::DIVSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5E])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MINPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x5D])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MINSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5D])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MAXPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x5F])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MAXSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5F])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::RSQRTPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x52])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::RSQRTSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x52])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::SHUFPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0xC6])
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::SQRTPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x51])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::SQRTSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x51])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::CMPPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0xC2])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Ins::CMPSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0xC2])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Ins::RCPPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x53])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::RCPSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x53])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::UCOMISS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x2E])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::COMISS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x2F])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ORPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x56])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ANDPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x54])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ANDNPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x55])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::XORPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x57])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::UNPCKLPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x14])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::UNPCKHPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x15])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

        // SSE2
        Ins::MOVNTI => GenAPI::new()
            .opcode(&[0x0F, 0xC3])
            .modrm(true, None, None)
            .rex(true),

        Ins::MFENCE => GenAPI::new().opcode(&[0xF0, 0xAE, 0xF0]),
        Ins::LFENCE => GenAPI::new().opcode(&[0xF0, 0xAE, 0xE8]),

        Ins::MOVNTPD => GenAPI::new()
            .opcode(&[0x0F, 0x2B])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true),
        Ins::MOVNTDQ => GenAPI::new()
            .opcode(&[0x0F, 0xE7])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true),
        Ins::MOVAPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x29]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVUPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVLPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x13]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x12]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVHPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x17]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x16]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVSD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0xF2).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVDQA => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x7F]).ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x6F]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Ins::MOVDQ2Q => GenAPI::new()
            .opcode(&[0x0F, 0xD6])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true),
        Ins::MOVQ2DQ => GenAPI::new()
            .opcode(&[0x0F, 0xD6])
            .prefix(0xF3)
            .modrm(true, None, None)
            .rex(true),

        Ins::MOVMSKPD => GenAPI::new()
            .opcode(&[0x0F, 0x50])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

        Ins::ADDPD => GenAPI::new()
            .opcode(&[0x0F, 0x58])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ADDSD => GenAPI::new()
            .opcode(&[0x0F, 0x58])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::SUBPD => GenAPI::new()
            .opcode(&[0x0F, 0x5C])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::SUBSD => GenAPI::new()
            .opcode(&[0x0F, 0x5C])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MULPD => GenAPI::new()
            .opcode(&[0x0F, 0x59])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MULSD => GenAPI::new()
            .opcode(&[0x0F, 0x59])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::DIVPD => GenAPI::new()
            .opcode(&[0x0F, 0x5E])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::DIVSD => GenAPI::new()
            .opcode(&[0x0F, 0x5E])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MINPD => GenAPI::new()
            .opcode(&[0x0F, 0x5D])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MINSD => GenAPI::new()
            .opcode(&[0x0F, 0x5D])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MAXPD => GenAPI::new()
            .opcode(&[0x0F, 0x5F])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MAXSD => GenAPI::new()
            .opcode(&[0x0F, 0x5F])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::SQRTPD => GenAPI::new()
            .opcode(&[0x0F, 0x51])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::SQRTSD => GenAPI::new()
            .opcode(&[0x0F, 0x51])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::CMPPD => GenAPI::new()
            .opcode(&[0x0F, 0xC2])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::CMPSD => GenAPI::new()
            .opcode(&[0x0F, 0xC2])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::COMISD => GenAPI::new()
            .opcode(&[0x0F, 0x2F])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::UCOMISD => GenAPI::new()
            .opcode(&[0x0F, 0x2E])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ORPD => GenAPI::new()
            .opcode(&[0x0F, 0x56])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ANDPD => GenAPI::new()
            .opcode(&[0x0F, 0x54])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ANDNPD => GenAPI::new()
            .opcode(&[0x0F, 0x55])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::XORPD => GenAPI::new()
            .opcode(&[0x0F, 0x57])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PSHUFLW => GenAPI::new()
            .opcode(&[0x0F, 0x70])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PSHUFHW => GenAPI::new()
            .opcode(&[0x0F, 0x70])
            .prefix(0xF3)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PSHUFD => GenAPI::new()
            .opcode(&[0x0F, 0x70])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),

        Ins::PSLLDQ => GenAPI::new()
            .opcode(&[0x0F, 0x73])
            .prefix(0x66)
            .modrm(true, Some(7), None)
            .rex(true)
            .imm_atindex(1, 1),
        Ins::PSRLDQ => GenAPI::new()
            .opcode(&[0x0F, 0x73])
            .prefix(0x66)
            .modrm(true, Some(3), None)
            .rex(true)
            .imm_atindex(1, 1),
        Ins::PUNPCKHQDQ => GenAPI::new()
            .opcode(&[0x0F, 0x6D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .prefix(0x66)
            .rex(true),
        Ins::PUNPCKLQDQ => GenAPI::new()
            .opcode(&[0x0F, 0x6C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .prefix(0x66)
            .rex(true),
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
            api
        }
        Ins::PADDB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PADDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFD])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PADDD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFE])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PADDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD4])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PADDUSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PADDUSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDD])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PADDSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PADDSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xED])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PSUBUSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD8])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PSUBUSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PSUBB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF8])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PSUBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PSUBD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFA])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PSUBQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::MASKMOVDQU => GenAPI::new()
            .opcode(&[0x0F, 0xF7])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true),

        Ins::PSUBSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE8])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PSUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PMULLW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD5])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PMULHW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE5])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PMULUDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF4])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PMADDWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF5])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PCMPEQB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x74])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PCMPEQW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x75])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PCMPEQD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x76])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PCMPGTB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x64])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PCMPGTW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x65])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PCMPGTD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x66])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PACKUSWB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x67])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PACKSSWB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x63])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PACKSSDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x6B])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PUNPCKLBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x60])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PUNPCKLWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x61])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PUNPCKLDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x62])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PUNPCKHBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x68])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PUNPCKHWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x69])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PUNPCKHDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x6A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
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
            api
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
            api
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
            api
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
            api
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
            api
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
            api
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
            api
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
            api
        }

        Ins::POR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PAND => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PANDN => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDF])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PXOR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEF])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::EMMS => GenAPI::new().opcode(&[0x0F, 0x77]),

        // sse3
        Ins::ADDSUBPD => GenAPI::new()
            .opcode(&[0x0F, 0xD0])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::ADDSUBPS => GenAPI::new()
            .opcode(&[0x0F, 0xD0])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),

        Ins::HADDPD => GenAPI::new()
            .opcode(&[0x0F, 0x7C])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::HADDPS => GenAPI::new()
            .opcode(&[0x0F, 0x7C])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::HSUBPD => GenAPI::new()
            .opcode(&[0x0F, 0x7D])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::HSUBPS => GenAPI::new()
            .opcode(&[0x0F, 0x7D])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),

        Ins::MOVSLDUP => GenAPI::new()
            .opcode(&[0x0F, 0x12])
            .modrm(true, None, None)
            .prefix(0xF3)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MOVSHDUP => GenAPI::new()
            .opcode(&[0x0F, 0x16])
            .modrm(true, None, None)
            .prefix(0xF3)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MOVDDUP => GenAPI::new()
            .opcode(&[0x0F, 0x12])
            .modrm(true, None, None)
            .prefix(0xF2)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

        Ins::LDDQU => GenAPI::new()
            .opcode(&[0x0F, 0xF0])
            .modrm(true, None, None)
            .prefix(0xF2)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

        Ins::MONITOR => GenAPI::new().opcode(&[0x0F, 0x01, 0xC8]),

        // ssse3
        Ins::PABSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1C])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PABSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1D])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PABSD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1E])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PSIGNB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x08])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PSIGNW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x09])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PSIGND => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x0A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Ins::PSHUFB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x00])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PHADDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x01])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PHADDD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x02])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PHADDSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x03])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PHSUBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x05])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PHSUBD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x06])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PHSUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x07])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
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
            api
        }
        Ins::PMULHRSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x0B])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PMADDUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x04])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        // sse4
        Ins::DPPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x40])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::DPPD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x41])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PTEST => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x17])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PEXTRW => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x15])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1),
        Ins::PEXTRB => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x14])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1),
        Ins::PEXTRD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x16])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1),
        Ins::PEXTRQ => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x16])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1),
        Ins::PINSRB => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x20])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PINSRD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x22])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PINSRQ => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x22])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PMAXSB => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3C])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PMAXSD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3D])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PMAXUW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3E])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PMINSB => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x38])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PMINSD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x39])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PMINUW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3A])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PMULDQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x28])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PMULLD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x40])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::BLENDPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0C])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::BLENDPD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0D])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PBLENDW => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0E])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PCMPEQQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x29])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ROUNDPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x08])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ROUNDPD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x09])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ROUNDSS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0A])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ROUNDSD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0B])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MPSADBW => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x42])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PCMPGTQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x37])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::BLENDVPS => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x14])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::BLENDVPD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x15])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PBLENDVB => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x10])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::INSERTPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x21])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PACKUSDW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x2B])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::MOVNTDQA => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x2A])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PCMPESTRM => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x60])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PCMPESTRI => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x61])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PCMPISTRM => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x62])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PCMPISTRI => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x63])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Ins::EXTRACTPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x17])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::PHMINPOSUW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x41])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::CRC32 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF0])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::POPCNT => GenAPI::new()
            .opcode(&[0x0F, 0xB8])
            .prefix(0xF3)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

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
            api
        }
        Ins::VMOVSLDUP => GenAPI::new()
            .opcode(&[0x12])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VLDDQU => GenAPI::new()
            .opcode(&[0xF0])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VMOVDDUP => GenAPI::new()
            .opcode(&[0x12])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VMOVSHDUP => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VMOVMSKPD => GenAPI::new()
            .opcode(&[0x50])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VMOVAPS => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x29]);
            } else {
                api = api.opcode(&[0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
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
            api
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
            api
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
            api
        }
        Ins::VADDPS => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VADDSUBPS => GenAPI::new()
            .opcode(&[0xD0])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VADDSUBPD => GenAPI::new()
            .opcode(&[0xD0])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VHADDPS => GenAPI::new()
            .opcode(&[0x7C])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VHADDPD => GenAPI::new()
            .opcode(&[0x7C])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VHSUBPS => GenAPI::new()
            .opcode(&[0x7D])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VHSUBPD => GenAPI::new()
            .opcode(&[0x7D])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VADDPD => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VADDSS => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VADDSD => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VSUBPS => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VSUBPD => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VSUBSS => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VSUBSD => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Ins::VMULPS => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMULPD => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMULSS => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMULSD => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VDIVPS => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VDIVPD => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VDIVSS => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VDIVSD => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Ins::VRCPPS => GenAPI::new()
            .opcode(&[0x53])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VRCPSS => GenAPI::new()
            .opcode(&[0x53])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),

        Ins::VSQRTPS => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VSQRTPD => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VSQRTSS => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VSQRTSD => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VRSQRTPS => GenAPI::new()
            .opcode(&[0x52])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VRSQRTSS => GenAPI::new()
            .opcode(&[0x52])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VPMULDQ => GenAPI::new()
            .opcode(&[0x28])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMULLD => GenAPI::new()
            .opcode(&[0x40])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMINSB => GenAPI::new()
            .opcode(&[0x38])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMINSD => GenAPI::new()
            .opcode(&[0x39])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMINUB => GenAPI::new()
            .opcode(&[0xDA])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMINUW => GenAPI::new()
            .opcode(&[0x3A])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMAXSB => GenAPI::new()
            .opcode(&[0x3C])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMAXSD => GenAPI::new()
            .opcode(&[0x3D])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMAXUB => GenAPI::new()
            .opcode(&[0xDE])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMAXUW => GenAPI::new()
            .opcode(&[0x3E])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Ins::VMINPS => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMINPD => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMINSS => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMINSD => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMAXPS => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMAXPD => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMAXSS => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMAXSD => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Ins::VORPS => GenAPI::new()
            .opcode(&[0x56])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VORPD => GenAPI::new()
            .opcode(&[0x56])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VANDPS => GenAPI::new()
            .opcode(&[0x54])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VANDPD => GenAPI::new()
            .opcode(&[0x54])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VANDNPD => GenAPI::new()
            .opcode(&[0x55])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VXORPD => GenAPI::new()
            .opcode(&[0x57])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Ins::VBLENDVPS => GenAPI::new()
            .opcode(&[0x4A])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VPBLENDVB => GenAPI::new()
            .opcode(&[0x4C])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VBLENDVPD => GenAPI::new()
            .opcode(&[0x4B])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),

        Ins::VPHMINPOSUW => GenAPI::new()
            .opcode(&[0x41])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VEXTRACTPS => GenAPI::new()
            .opcode(&[0x17])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),

        Ins::VMOVNTDQA => GenAPI::new()
            .opcode(&[0x2A])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPACKUSDW => GenAPI::new()
            .opcode(&[0x2B])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPCMPESTRM => GenAPI::new()
            .opcode(&[0x60])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Ins::VPCMPESTRI => GenAPI::new()
            .opcode(&[0x61])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Ins::VPCMPISTRM => GenAPI::new()
            .opcode(&[0x62])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Ins::VPCMPISTRI => GenAPI::new()
            .opcode(&[0x63])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Ins::VINSERTPS => GenAPI::new()
            .opcode(&[0x21])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VBLENDPS => GenAPI::new()
            .opcode(&[0x0C])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VBLENDPD => GenAPI::new()
            .opcode(&[0x0D])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VPCMPGTQ => GenAPI::new()
            .opcode(&[0x37])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPCMPEQQ => GenAPI::new()
            .opcode(&[0x29])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMPSADBW => GenAPI::new()
            .opcode(&[0x42])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VROUNDSS => GenAPI::new()
            .opcode(&[0x0A])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VROUNDSD => GenAPI::new()
            .opcode(&[0x0B])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VROUNDPS => GenAPI::new()
            .opcode(&[0x08])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Ins::VROUNDPD => GenAPI::new()
            .opcode(&[0x09])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Ins::VPBLENDW => GenAPI::new()
            .opcode(&[0x0E])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VCMPPD => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VANDNPS => GenAPI::new()
            .opcode(&[0x55])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VXORPS => GenAPI::new()
            .opcode(&[0x57])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPTEST => GenAPI::new()
            .opcode(&[0x17])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VDPPS => GenAPI::new()
            .opcode(&[0x40])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VDPPD => GenAPI::new()
            .opcode(&[0x41])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VCMPPS => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VCMPSS => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VCMPSD => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VUCOMISS => GenAPI::new()
            .opcode(&[0x2E])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VUCOMISD => GenAPI::new()
            .opcode(&[0x2E])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCOMISS => GenAPI::new()
            .opcode(&[0x2F])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCOMISD => GenAPI::new()
            .opcode(&[0x2F])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VUNPCKLPS => GenAPI::new()
            .opcode(&[0x14])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VUNPCKHPS => GenAPI::new()
            .opcode(&[0x15])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VSHUFPS => GenAPI::new()
            .opcode(&[0xC6])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VMOVSS => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0xF3).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x11]);
            } else {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            api
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
            api
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
            api
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
            api
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
            api
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
            api
        }
        Ins::VMOVLHPS => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMOVHLPS => GenAPI::new()
            .opcode(&[0x12])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPEXTRB => GenAPI::new()
            .opcode(&[0x14])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Ins::VPEXTRW => GenAPI::new()
            .opcode(&[0xC5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Ins::VPEXTRD => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Ins::VPEXTRQ => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(true))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Ins::VPINSRB => GenAPI::new()
            .opcode(&[0x20])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VPINSRD => GenAPI::new()
            .opcode(&[0x22])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Ins::VPINSRQ => GenAPI::new()
            .opcode(&[0x22])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(true))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),

        // MMX derived part 1
        Ins::VPOR => GenAPI::new()
            .opcode(&[0xEB])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
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
            api
        }
        Ins::VPAND => GenAPI::new()
            .opcode(&[0xDB])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPXOR => GenAPI::new()
            .opcode(&[0xEF])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPADDB => GenAPI::new()
            .opcode(&[0xFC])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPADDW => GenAPI::new()
            .opcode(&[0xFD])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPADDD => GenAPI::new()
            .opcode(&[0xFE])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPADDQ => GenAPI::new()
            .opcode(&[0xD4])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSUBB => GenAPI::new()
            .opcode(&[0xF8])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSUBW => GenAPI::new()
            .opcode(&[0xF9])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSUBD => GenAPI::new()
            .opcode(&[0xFA])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSUBQ => GenAPI::new()
            .opcode(&[0xFB])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPANDN => GenAPI::new()
            .opcode(&[0xDF])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
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
            api
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
            api
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
            api
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
            api
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
            api
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
            api
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
            api
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
            api
        }
        Ins::VPSUBSB => GenAPI::new()
            .opcode(&[0xE8])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSUBSW => GenAPI::new()
            .opcode(&[0xE9])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPADDSB => GenAPI::new()
            .opcode(&[0xEC])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPADDSW => GenAPI::new()
            .opcode(&[0xED])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMULHW => GenAPI::new()
            .opcode(&[0xE5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMULLW => GenAPI::new()
            .opcode(&[0xD5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        // part 2
        Ins::VPADDUSB => GenAPI::new()
            .opcode(&[0xDC])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPADDUSW => GenAPI::new()
            .opcode(&[0xDD])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSUBUSB => GenAPI::new()
            .opcode(&[0xD8])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSUBUSW => GenAPI::new()
            .opcode(&[0xD9])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMADDWD => GenAPI::new()
            .opcode(&[0xF5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPCMPEQB => GenAPI::new()
            .opcode(&[0x74])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPCMPEQW => GenAPI::new()
            .opcode(&[0x75])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPCMPEQD => GenAPI::new()
            .opcode(&[0x76])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPCMPGTB => GenAPI::new()
            .opcode(&[0x64])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPCMPGTW => GenAPI::new()
            .opcode(&[0x65])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPCMPGTD => GenAPI::new()
            .opcode(&[0x66])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPACKUSWB => GenAPI::new()
            .opcode(&[0x67])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPACKSSWB => GenAPI::new()
            .opcode(&[0x63])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPACKSSDW => GenAPI::new()
            .opcode(&[0x6B])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPUNPCKLBW => GenAPI::new()
            .opcode(&[0x60])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPUNPCKLWD => GenAPI::new()
            .opcode(&[0x61])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPUNPCKLDQ => GenAPI::new()
            .opcode(&[0x62])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPUNPCKHBW => GenAPI::new()
            .opcode(&[0x68])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPUNPCKHWD => GenAPI::new()
            .opcode(&[0x69])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPUNPCKHDQ => GenAPI::new()
            .opcode(&[0x6A])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        // part2a
        Ins::PAVGB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE0])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PAVGW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE3])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::VPAVGB => GenAPI::new()
            .opcode(&[0xE0])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPAVGW => GenAPI::new()
            .opcode(&[0xE3])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPHADDW => GenAPI::new()
            .opcode(&[0x01])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPHADDD => GenAPI::new()
            .opcode(&[0x02])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPHSUBW => GenAPI::new()
            .opcode(&[0x05])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPHSUBD => GenAPI::new()
            .opcode(&[0x06])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VZEROUPPER => GenAPI::new().opcode(&[0xC5, 0xF8, 0x77]),
        Ins::VZEROALL => GenAPI::new().opcode(&[0xC5, 0xFC, 0x77]),
        Ins::VPALIGNR => GenAPI::new()
            .opcode(&[0x0F])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .imm_atindex(3, 1)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VINSERTF128 => GenAPI::new()
            .opcode(&[0x18])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .modrm(true, None, None)
            .imm_atindex(3, 1)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VEXTRACTF128 => GenAPI::new()
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
            .ord(&[MODRM_RM, MODRM_REG]),
        Ins::VBROADCASTSS => GenAPI::new()
            .opcode(&[0x18])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VBROADCASTSD => GenAPI::new()
            .opcode(&[0x19])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VBROADCASTF128 => GenAPI::new()
            .opcode(&[0x1A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None),
        Ins::STMXCSR => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(3), None)
            .rex(true),
        Ins::LDMXCSR => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(2), None)
            .rex(true),
        Ins::VSTMXCSR => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false))
            .modrm(true, Some(3), None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VLDMXCSR => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false))
            .modrm(true, Some(2), None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VMOVMSKPS => GenAPI::new()
            .opcode(&[0x50])
            .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPERMILPS => {
            if let Some(Operand::Imm(_)) = ins.src2() {
                GenAPI::new()
                    .modrm(true, None, None)
                    .opcode(&[0x04])
                    .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                    .ord(&[MODRM_REG, MODRM_RM])
                    .imm_atindex(2, 1)
            } else {
                GenAPI::new()
                    .modrm(true, None, None)
                    .opcode(&[0x0C])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            }
        }
        Ins::VPERMILPD => {
            if let Some(Operand::Imm(_)) = ins.src2() {
                GenAPI::new()
                    .modrm(true, None, None)
                    .opcode(&[0x05])
                    .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                    .ord(&[MODRM_REG, MODRM_RM])
                    .imm_atindex(2, 1)
            } else {
                GenAPI::new()
                    .modrm(true, None, None)
                    .opcode(&[0x0D])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            }
        }
        Ins::PCLMULQDQ => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x44])
            .prefix(0x66)
            .rex(true)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VPCLMULQDQ => GenAPI::new()
            .opcode(&[0x44])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPERM2F128 => GenAPI::new()
            .opcode(&[0x06])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPERM2I128 => GenAPI::new()
            .opcode(&[0x46])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        // part2c
        Ins::VPINSRW => GenAPI::new()
            .opcode(&[0xC4])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMAXSW => GenAPI::new()
            .opcode(&[0xEE])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMINSW => GenAPI::new()
            .opcode(&[0xEA])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSRLDQ => GenAPI::new()
            .opcode(&[0x73])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .imm_atindex(2, 1)
            .modrm(true, Some(3), None)
            .ord(&[VEX_VVVV, MODRM_RM]),
        Ins::VPSIGNB => GenAPI::new()
            .opcode(&[0x08])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSIGNW => GenAPI::new()
            .opcode(&[0x09])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPSIGND => GenAPI::new()
            .opcode(&[0x0A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMULUDQ => GenAPI::new()
            .opcode(&[0xF4])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMULHUW => GenAPI::new()
            .opcode(&[0xE4])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VPMULHRSW => GenAPI::new()
            .opcode(&[0x0B])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        // part2c-ext
        Ins::PMAXSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEE])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
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
            api
        }
        Ins::PMINSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEA])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Ins::PMAXUD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3F])
            .prefix(0x66)
            .rex(true)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VPMAXUD => GenAPI::new()
            .opcode(&[0x3F])
            .modrm(true, None, None)
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::PMULHUW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE4])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66);
            }
            api
        }
        // fma-part1
        Ins::VFMADD132PS => GenAPI::new()
            .opcode(&[0x98])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD213PS => GenAPI::new()
            .opcode(&[0xA8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD231PS => GenAPI::new()
            .opcode(&[0xB8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD132PD => GenAPI::new()
            .opcode(&[0x98])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD213PD => GenAPI::new()
            .opcode(&[0xA8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD231PD => GenAPI::new()
            .opcode(&[0xB8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD132SD => GenAPI::new()
            .opcode(&[0x99])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD213SD => GenAPI::new()
            .opcode(&[0xA9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD231SD => GenAPI::new()
            .opcode(&[0xB9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD132SS => GenAPI::new()
            .opcode(&[0x99])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD213SS => GenAPI::new()
            .opcode(&[0xA9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADD231SS => GenAPI::new()
            .opcode(&[0xB9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFMSUB132PS => GenAPI::new()
            .opcode(&[0x9A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB213PS => GenAPI::new()
            .opcode(&[0xAA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB231PS => GenAPI::new()
            .opcode(&[0xBA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFMSUB132PD => GenAPI::new()
            .opcode(&[0x9A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB213PD => GenAPI::new()
            .opcode(&[0xAA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB231PD => GenAPI::new()
            .opcode(&[0xBA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB132SD => GenAPI::new()
            .opcode(&[0x9B])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB213SD => GenAPI::new()
            .opcode(&[0xAB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB231SD => GenAPI::new()
            .opcode(&[0xBB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB132SS => GenAPI::new()
            .opcode(&[0x9B])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB213SS => GenAPI::new()
            .opcode(&[0xAB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUB231SS => GenAPI::new()
            .opcode(&[0xBB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        // fma-part2
        Ins::VFNMADD132PS => GenAPI::new()
            .opcode(&[0x9C])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMADD213PS => GenAPI::new()
            .opcode(&[0xAC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMADD231PS => GenAPI::new()
            .opcode(&[0xBC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFNMADD132PD => GenAPI::new()
            .opcode(&[0x9C])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMADD213PD => GenAPI::new()
            .opcode(&[0xAC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMADD231PD => GenAPI::new()
            .opcode(&[0xBC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFNMADD132SS => GenAPI::new()
            .opcode(&[0x9D])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMADD213SS => GenAPI::new()
            .opcode(&[0xAD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMADD231SS => GenAPI::new()
            .opcode(&[0xBD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFNMADD132SD => GenAPI::new()
            .opcode(&[0x9D])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMADD213SD => GenAPI::new()
            .opcode(&[0xAD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMADD231SD => GenAPI::new()
            .opcode(&[0xBD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFNMSUB132PS => GenAPI::new()
            .opcode(&[0x9E])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMSUB213PS => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMSUB231PS => GenAPI::new()
            .opcode(&[0xBE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFNMSUB132PD => GenAPI::new()
            .opcode(&[0x9E])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMSUB213PD => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMSUB231PD => GenAPI::new()
            .opcode(&[0xBE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFNMSUB132SS => GenAPI::new()
            .opcode(&[0x9F])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMSUB213SS => GenAPI::new()
            .opcode(&[0xAF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMSUB231SS => GenAPI::new()
            .opcode(&[0xBF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFNMSUB132SD => GenAPI::new()
            .opcode(&[0x9F])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMSUB213SD => GenAPI::new()
            .opcode(&[0xAF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFNMSUB231SD => GenAPI::new()
            .opcode(&[0xBF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        // fma-part3
        Ins::VFMADDSUB132PS => GenAPI::new()
            .opcode(&[0x96])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADDSUB213PS => GenAPI::new()
            .opcode(&[0xA6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADDSUB231PS => GenAPI::new()
            .opcode(&[0xB6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADDSUB132PD => GenAPI::new()
            .opcode(&[0x96])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADDSUB213PD => GenAPI::new()
            .opcode(&[0xA6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMADDSUB231PD => GenAPI::new()
            .opcode(&[0xB6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Ins::VFMSUBADD132PS => GenAPI::new()
            .opcode(&[0x97])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUBADD213PS => GenAPI::new()
            .opcode(&[0xA7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUBADD231PS => GenAPI::new()
            .opcode(&[0xB7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUBADD132PD => GenAPI::new()
            .opcode(&[0x97])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUBADD213PD => GenAPI::new()
            .opcode(&[0xA7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VFMSUBADD231PD => GenAPI::new()
            .opcode(&[0xB7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        // aes
        Ins::AESDEC => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDE])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::AESENC => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDC])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::AESIMC => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDB])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::AESDECLAST => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDF])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::AESENCLAST => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),

        Ins::VAESDEC => GenAPI::new()
            .opcode(&[0xDE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VAESENC => GenAPI::new()
            .opcode(&[0xDC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VAESIMC => GenAPI::new()
            .opcode(&[0xDB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM]),
        Ins::VAESENCLAST => GenAPI::new()
            .opcode(&[0xDD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VAESDECLAST => GenAPI::new()
            .opcode(&[0xDF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Ins::VAESKEYGENASSIST => GenAPI::new()
            .opcode(&[0xDF])
            .imm_atindex(2, 1)
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM]),
        Ins::AESKEYGENASSIST => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0xDF])
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .prefix(0x66)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        // cvt-part1
        Ins::CVTPD2PI => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x2D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTSS2SD => GenAPI::new()
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTPD2PS => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTPS2PD => GenAPI::new()
            .opcode(&[0x0F, 0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTPI2PD => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTPD2DQ => GenAPI::new()
            .opcode(&[0x0F, 0xE6])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTSD2SS => GenAPI::new()
            .opcode(&[0x0F, 0x5A])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTPS2DQ => GenAPI::new()
            .opcode(&[0x0F, 0x5B])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTDQ2PS => GenAPI::new()
            .opcode(&[0x0F, 0x5B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTDQ2PD => GenAPI::new()
            .opcode(&[0x0F, 0xE6])
            .modrm(true, None, None)
            .prefix(0xF3)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTSD2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2D])
            .modrm(true, None, None)
            .prefix(0xF2)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTSI2SD => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .modrm(true, None, None)
            .prefix(0xF2)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),

        Ins::CVTTPS2DQ => GenAPI::new()
            .opcode(&[0x0F, 0x5B])
            .prefix(0xF3)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTTSD2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None, None)
            .prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTTPD2PI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None, None)
            .prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTSI2SS => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .prefix(0xF3)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::CVTPS2PI => GenAPI::new()
            .opcode(&[0x0F, 0x2D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTTPS2PI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTPI2PS => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTTPD2DQ => GenAPI::new()
            .opcode(&[0x0F, 0xE6])
            .modrm(true, None, None)
            .prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::CVTTSS2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .prefix(0xF3)
            .rex(true),
        Ins::CVTSS2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2D])
            .prefix(0xF3)
            .rex(true)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        // cvt-part2
        Ins::VCVTPD2DQ => GenAPI::new()
            .opcode(&[0xE6])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTPD2PS => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTPS2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTPS2PD => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTSD2SI => GenAPI::new()
            .opcode(&[0x2D])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF2)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTSD2SS => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VCVTSI2SD => GenAPI::new()
            .opcode(&[0x2A])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF2)
                    .vex_we(ins.src2().unwrap().size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None, None),
        Ins::VCVTSI2SS => GenAPI::new()
            .opcode(&[0x2A])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF3)
                    .vex_we(ins.src2().unwrap().size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VCVTSS2SD => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::VCVTSS2SI => GenAPI::new()
            .opcode(&[0x2D])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF3)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Ins::VCVTDQ2PD => GenAPI::new()
            .opcode(&[0xE6])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTDQ2PS => GenAPI::new()
            .opcode(&[0x5B])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTTPD2DQ => GenAPI::new()
            .opcode(&[0xE6])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTTPS2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTTSD2SI => GenAPI::new()
            .opcode(&[0x2C])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF2)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::VCVTTSS2SI => GenAPI::new()
            .opcode(&[0x2C])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF3)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        // norm-part1a
        Ins::BT => ins_bt(ins, &[0x0F, 0xA3], &[0x0F, 0xBA], bits, 4),
        Ins::BTS => ins_bt(ins, &[0x0F, 0xAB], &[0x0F, 0xBA], bits, 5),
        Ins::BTC => ins_bt(ins, &[0x0F, 0xBB], &[0x0F, 0xBA], bits, 7),
        Ins::BTR => ins_bt(ins, &[0x0F, 0xB3], &[0x0F, 0xBA], bits, 6),
        Ins::CLC => GenAPI::new().opcode(&[0xF8]),
        Ins::CMC => GenAPI::new().opcode(&[0xF5]),
        Ins::CWD => GenAPI::new().opcode(&[0x99]).fixed_size(Size::Word),
        Ins::CDQ => GenAPI::new().opcode(&[0x99]).fixed_size(Size::Dword),
        Ins::CQO => GenAPI::new().opcode(&[0x48, 0x99]),
        Ins::DAA => GenAPI::new().opcode(&[0x27]),
        Ins::DAS => GenAPI::new().opcode(&[0x2F]),
        Ins::CLD => GenAPI::new().opcode(&[0xFC]),
        Ins::CBW => GenAPI::new().opcode(&[0x98]).fixed_size(Size::Word),
        Ins::CLI => GenAPI::new().opcode(&[0xFA]),
        Ins::AAA => GenAPI::new().opcode(&[0x37]),
        Ins::AAS => GenAPI::new().opcode(&[0x3F]),
        Ins::AAD => GenAPI::new().opcode(&[
            0xD5,
            if let Some(Operand::Imm(n)) = ins.dst() {
                n.split_into_bytes()[0]
            } else {
                0x0A
            },
        ]),
        Ins::AAM => GenAPI::new().opcode(&[
            0xD4,
            if let Some(Operand::Imm(n)) = ins.dst() {
                n.split_into_bytes()[0]
            } else {
                0x0A
            },
        ]),
        Ins::ADC => add_like_ins(
            ins,
            &[0x14, 0x15, 0x80, 0x81, 0x83, 0x10, 0x11, 0x12, 0x13],
            2,
            bits,
        ),
        Ins::BSF => GenAPI::new()
            .opcode(&[0x0F, 0xBC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::BSR => GenAPI::new()
            .opcode(&[0x0F, 0xBD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        // part b
        Ins::ADCX => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF6])
            .modrm(true, None, None)
            .prefix(0x66)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Ins::ADOX => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF6])
            .modrm(true, None, None)
            .prefix(0xF3)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::ANDN => GenAPI::new()
            .opcode(&[0xF2])
            .modrm(true, None, None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0x00)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::CWDE => GenAPI::new().opcode(&[0x98]).fixed_size(Size::Dword),
        Ins::CDQE => GenAPI::new().opcode(&[0x48, 0x98]),
        Ins::CLAC => GenAPI::new().opcode(&[0x0F, 0x01, 0xCA]),
        Ins::CLTS => GenAPI::new().opcode(&[0x0F, 0x06]),
        Ins::CLUI => GenAPI::new().opcode(&[0xF3, 0x0F, 0x01, 0xEE]),
        Ins::CLWB => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .prefix(0x66)
            .modrm(true, Some(6), None),
        Ins::ARPL => GenAPI::new().opcode(&[0x63]).modrm(true, None, None),

        Ins::BLSR => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(1), None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[VEX_VVVV, MODRM_RM]),
        Ins::BLSI => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(3), None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[VEX_VVVV, MODRM_RM]),
        Ins::BZHI => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None, None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::BEXTR => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::BLSMSK => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(2), None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[VEX_VVVV, MODRM_RM]),
        Ins::BSWAP => GenAPI::new()
            .opcode(&[0x0F, 0xC8 + ins.reg_byte(0).unwrap_or(0)])
            .modrm(false, None, None)
            .rex(true),
        // part c
        Ins::CMPSTRB => GenAPI::new().opcode(&[0xA6]).fixed_size(Size::Byte),
        Ins::CMPSTRW => GenAPI::new().opcode(&[0xA7]).fixed_size(Size::Word),
        Ins::CMPSTRD => GenAPI::new().opcode(&[0xA7]).fixed_size(Size::Dword),
        Ins::CMPSTRQ => GenAPI::new().opcode(&[0x48, 0xA7]).fixed_size(Size::Qword),
        Ins::ENDBR64 => GenAPI::new().opcode(&[0xF3, 0x0F, 0x1E, 0xFA]),
        Ins::ENDBR32 => GenAPI::new().opcode(&[0xF3, 0x0F, 0x1E, 0xFB]),
        Ins::CMPXCHG => GenAPI::new()
            .opcode(&[0x0F, (0xB1 - ((ins.size() == Size::Byte) as u8))])
            .modrm(true, None, None)
            .rex(true),
        Ins::CLDEMOTE => GenAPI::new()
            .opcode(&[0x0F, 0x1C])
            .modrm(true, Some(0), None),
        Ins::CLRSSBSY => {
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .prefix(0xF3)
                .modrm(true, Some(6), None)
        }
        Ins::CMPXCHG8B => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(1), None),
        Ins::CMPXCHG16B => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(1), None)
            .rex(true),
        // part 3
        Ins::ENTER => {
            let imm = if let Some(Operand::Imm(n)) = ins.dst() {
                n.get_as_u64()
            } else {
                0
            };
            let imm = imm.to_le_bytes();
            GenAPI::new()
                .opcode(&[0xC8, imm[0], imm[1]])
                .imm_atindex(1, 1)
        }
        Ins::HLT => GenAPI::new().opcode(&[0xF4]),
        Ins::HRESET => GenAPI::new()
            .opcode(&[0xF3, 0x0F, 0x3A, 0xF0, 0xC0])
            .modrm(false, None, None)
            .imm_atindex(0, 1),
        Ins::INSB => GenAPI::new().opcode(&[0x6C]).fixed_size(Size::Byte),
        Ins::INSW => GenAPI::new().opcode(&[0x6D]).fixed_size(Size::Word),
        Ins::INSD => GenAPI::new().opcode(&[0x6D]).fixed_size(Size::Dword),
        Ins::INT => GenAPI::new().opcode(&[
            0xCC,
            if let Some(Operand::Imm(imm)) = ins.dst() {
                imm.split_into_bytes()[0]
            } else {
                0x00
            },
        ]),
        Ins::INTO => GenAPI::new().opcode(&[0xCE]),
        Ins::INT3 => GenAPI::new().opcode(&[0xCC]),
        Ins::INT1 => GenAPI::new().opcode(&[0xF1]),
        Ins::INVD => GenAPI::new().opcode(&[0x0F, 0x08]),
        Ins::INVLPG => GenAPI::new().opcode(&[0x0F, 0x01, 0b11_111_000]),
        Ins::INVPCID => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x82])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::IRET | Ins::IRETD => GenAPI::new().opcode(&[0xCF]),
        Ins::IRETQ => GenAPI::new().opcode(&[0x48, 0xCF]),
        Ins::LAHF => GenAPI::new().opcode(&[0x9F]),
        Ins::LAR => GenAPI::new()
            .opcode(&[0x0F, 0x02])
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Ins::LEAVE => GenAPI::new().opcode(&[0xC9]),
        Ins::LLDT => GenAPI::new()
            .opcode(&[0x0F, 0x00])
            .modrm(true, Some(2), None)
            .can_h66(false),
        Ins::LMSW => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(6), None)
            .can_h66(false),
        Ins::LODSB => GenAPI::new().fixed_size(Size::Byte).opcode(&[0xAC]),
        Ins::LODSW => GenAPI::new().fixed_size(Size::Word).opcode(&[0xAD]),
        Ins::LODSD => GenAPI::new().fixed_size(Size::Dword).opcode(&[0xAD]),
        Ins::LODSQ => GenAPI::new().fixed_size(Size::Qword).opcode(&[0x48, 0xAD]),

        // part 3
        Ins::LOOP => ins_shrtjmp(ins, vec![0xE2]),
        Ins::LOOPE => ins_shrtjmp(ins, vec![0xE1]),
        Ins::LOOPNE => ins_shrtjmp(ins, vec![0xE0]),
        Ins::LSL => GenAPI::new()
            .opcode(&[0x0F, 0x03])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::LTR => GenAPI::new()
            .opcode(&[0x0F, 0x00])
            .modrm(true, Some(3), None)
            .can_h66(false),
        Ins::LZCNT => GenAPI::new()
            .opcode(&[0x0F, 0xBD])
            .prefix(0xF3)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::MOVBE => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Reg(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x38, 0xF0]);
                api = api.ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x38, 0xF1]);
            }
            api
        }
        Ins::MOVZX => GenAPI::new()
            .opcode(&[
                0x0F,
                (0xB6 + ((ins.src().unwrap().size() == Size::Word) as u8)),
            ])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Ins::MOVDIRI => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF9])
            .modrm(true, None, None)
            .rex(true),
        Ins::MOVSTRB => GenAPI::new().opcode(&[0xA4]).fixed_size(Size::Byte),
        Ins::MOVSTRW => GenAPI::new().opcode(&[0xA5]).fixed_size(Size::Word),
        Ins::MOVSTRD => GenAPI::new().opcode(&[0xA5]).fixed_size(Size::Dword),
        Ins::MOVSTRQ => GenAPI::new().opcode(&[0x48, 0xA5]).fixed_size(Size::Qword),
        Ins::MULX => GenAPI::new()
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
            )
            .opcode(&[0xF6])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::OUTSB => GenAPI::new().opcode(&[0x6E]).fixed_size(Size::Byte),
        Ins::OUTSW => GenAPI::new().opcode(&[0x6F]).fixed_size(Size::Word),
        Ins::OUTSD => GenAPI::new().opcode(&[0x6F]).fixed_size(Size::Dword),
        Ins::PEXT => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None, None)
            .vex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::PDEP => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None, None)
            .vex(
                VexDetails::new()
                    .pp(0xF2)
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Ins::PREFETCHW => GenAPI::new()
            .opcode(&[0x0F, 0x0D])
            .modrm(true, Some(1), None),
        Ins::PREFETCH0 => GenAPI::new()
            .opcode(&[0x0F, 0x18])
            .modrm(true, Some(1), None),
        Ins::PREFETCH1 => GenAPI::new()
            .opcode(&[0x0F, 0x18])
            .modrm(true, Some(2), None),
        Ins::PREFETCH2 => GenAPI::new()
            .opcode(&[0x0F, 0x18])
            .modrm(true, Some(3), None),
        Ins::PREFETCHA => GenAPI::new()
            .opcode(&[0x0F, 0x18])
            .modrm(true, Some(0), None),

        Ins::ROL => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 0, bits),
        Ins::ROR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 1, bits),
        Ins::RCL => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 2, bits),
        Ins::RCR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 3, bits),
        // part 4
        Ins::RDMSR => GenAPI::new().opcode(&[0x0F, 0x32]),
        Ins::RDPID => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .prefix(0xF3)
            .modrm(true, Some(7), None),
        Ins::RDPKRU => GenAPI::new().opcode(&[0x0F, 0x01, 0xEE]),
        Ins::RDPMC => GenAPI::new().opcode(&[0x0F, 0x33]),
        Ins::RDRAND => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(6), None)
            .rex(true),
        Ins::RDSEED => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(7), None)
            .rex(true),
        Ins::RDSSPD | Ins::RDSSPQ => GenAPI::new()
            .opcode(&[0x0F, 0x1E])
            .modrm(true, Some(1), None)
            .prefix(0xF3)
            .rex(true),
        Ins::RDTSC => GenAPI::new().opcode(&[0x0F, 0x31]),
        Ins::RDTSCP => GenAPI::new().opcode(&[0x0F, 0x01, 0xF9]),
        Ins::RORX => GenAPI::new()
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
            .imm_atindex(2, 1),
        Ins::RSM => GenAPI::new().opcode(&[0x0F, 0xAA]),
        Ins::RSTORSSP => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(5), None)
            .prefix(0xF3)
            .rex(true),
        Ins::SAHF => GenAPI::new().opcode(&[0x9E]),
        Ins::SHLX => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0x66)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::SHRX => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::SARX => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0xF3)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::SBB => add_like_ins(
            ins,
            &[0x1C, 0x1D, 0x80, 0x81, 0x83, 0x18, 0x19, 0x1A, 0x1B],
            3,
            bits,
        ),
        Ins::SCASB => GenAPI::new().fixed_size(Size::Byte).opcode(&[0xAE]),
        Ins::SCASW => GenAPI::new().fixed_size(Size::Word).opcode(&[0xAF]),
        Ins::SCASD => GenAPI::new().fixed_size(Size::Dword).opcode(&[0xAF]),
        Ins::SCASQ => GenAPI::new().fixed_size(Size::Qword).opcode(&[0x48, 0xAF]),
        Ins::SENDUIPI => GenAPI::new()
            .prefix(0xF3)
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(6), None)
            .can_h66(false)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::SERIALIZE => GenAPI::new().opcode(&[0x0F, 0x01, 0xE8]),
        // for some reason NASM generates this as no opcode at all?
        Ins::SETSSBY => GenAPI::new(),

        // setcc
        Ins::SETO => GenAPI::new()
            .opcode(&[0x0F, 0x90])
            .modrm(true, None, None)
            .rex(true),
        Ins::SETNO => GenAPI::new()
            .opcode(&[0x0F, 0x91])
            .modrm(true, None, None)
            .rex(true),
        Ins::SETB | Ins::SETC | Ins::SETNAE => GenAPI::new()
            .opcode(&[0x0F, 0x92])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETAE | Ins::SETNB | Ins::SETNC => GenAPI::new()
            .opcode(&[0x0F, 0x93])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETE | Ins::SETZ => GenAPI::new()
            .opcode(&[0x0F, 0x94])
            .modrm(true, None, None)
            .rex(true),
        Ins::SETNE | Ins::SETNZ => GenAPI::new()
            .opcode(&[0x0F, 0x95])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETBE | Ins::SETNA => GenAPI::new()
            .opcode(&[0x0F, 0x96])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETA | Ins::SETNBE => GenAPI::new()
            .opcode(&[0x0F, 0x97])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETS => GenAPI::new()
            .opcode(&[0x0F, 0x98])
            .modrm(true, None, None)
            .rex(true),
        Ins::SETNS => GenAPI::new()
            .opcode(&[0x0F, 0x99])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETP | Ins::SETPE => GenAPI::new()
            .opcode(&[0x0F, 0x9A])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETNP | Ins::SETPO => GenAPI::new()
            .opcode(&[0x0F, 0x9B])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETL | Ins::SETNGE => GenAPI::new()
            .opcode(&[0x0F, 0x9C])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETGE | Ins::SETNL => GenAPI::new()
            .opcode(&[0x0F, 0x9D])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETLE | Ins::SETNG => GenAPI::new()
            .opcode(&[0x0F, 0x9E])
            .modrm(true, None, None)
            .rex(true),

        Ins::SETG | Ins::SETNLE => GenAPI::new()
            .opcode(&[0x0F, 0x9F])
            .modrm(true, None, None)
            .rex(true),

        // norm-part5
        Ins::SFENCE => GenAPI::new().opcode(&[0x0F, 0xAE, 0xF8]),
        Ins::STAC => GenAPI::new().opcode(&[0x0F, 0x01, 0xCB]),
        Ins::STC => GenAPI::new().opcode(&[0xF9]),
        Ins::STD => GenAPI::new().opcode(&[0xFD]),
        Ins::STI => GenAPI::new().opcode(&[0xFB]),
        Ins::STUI => GenAPI::new().opcode(&[0xF3, 0x0F, 0x01, 0xEF]),
        Ins::STOSB => GenAPI::new().opcode(&[0xAA]),
        Ins::STOSW => GenAPI::new().opcode(&[0xAB]).fixed_size(Size::Word),
        Ins::STOSD => GenAPI::new().opcode(&[0xAB]).fixed_size(Size::Dword),
        Ins::STOSQ => GenAPI::new().opcode(&[0x48, 0xAB]),
        Ins::SYSENTER => GenAPI::new().opcode(&[0x0F, 0x34]),
        Ins::SYSEXIT => GenAPI::new().opcode(&[0x0F, 0x35]),
        Ins::SYSRET => GenAPI::new().opcode(&[0x0F, 0x07]),
        Ins::TESTUI => GenAPI::new().opcode(&[0xF3, 0x0F, 0x01, 0xED]),
        Ins::UD2 => GenAPI::new().opcode(&[0x0F, 0x0B]),
        Ins::UD0 => GenAPI::new()
            .opcode(&[0x0F, 0xFF])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::UD1 => GenAPI::new()
            .opcode(&[0x0F, 0xB9])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::TPAUSE => GenAPI::new()
            .modrm(true, Some(6), None)
            .prefix(0x66)
            .opcode(&[0x0F, 0xAE]),
        Ins::UMWAIT => GenAPI::new()
            .modrm(true, Some(6), None)
            .prefix(0xF2)
            .opcode(&[0x0F, 0xAE]),
        Ins::UMONITOR => GenAPI::new()
            .modrm(true, Some(6), None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0xAE]),
        Ins::SMSW => GenAPI::new()
            .modrm(true, Some(4), None)
            .opcode(&[0x0F, 0x01])
            .rex(true),
        Ins::STR => GenAPI::new()
            .modrm(true, Some(1), None)
            .opcode(&[0x0F, 0x00]),
        Ins::VERR => GenAPI::new()
            .modrm(true, Some(4), None)
            .opcode(&[0x0F, 0x00])
            .can_h66(false),
        Ins::VERW => GenAPI::new()
            .modrm(true, Some(5), None)
            .opcode(&[0x0F, 0x00])
            .can_h66(false),
        Ins::SHLD => ins_shlx(ins, &[0x0F, 0xA4], &[0x0F, 0xA5]),
        Ins::SHRD => ins_shlx(ins, &[0x0F, 0xAC], &[0x0F, 0xAD]),
        Ins::UIRET => GenAPI::new().opcode(&[0xF3, 0x0F, 0x01, 0xEC]),
        Ins::WAIT | Ins::FWAIT => GenAPI::new().opcode(&[0x9B]),
        Ins::WBINVD => GenAPI::new().opcode(&[0x0F, 0x09]),
        Ins::WRMSR => GenAPI::new().opcode(&[0x0F, 0x30]),
        Ins::WRPKRU => GenAPI::new().opcode(&[0x0F, 0x01, 0xEF]),

        // norm-part6
        Ins::XABORT => GenAPI::new().imm_atindex(0, 1).opcode(&[0xC6, 0xF8]),
        Ins::XACQUIRE => GenAPI::new().opcode(&[0xF2]),
        Ins::XRELEASE => GenAPI::new().opcode(&[0xF3]),
        Ins::XADD => GenAPI::new()
            .opcode(&[0x0F, (0xC0 + ((ins.size() != Size::Byte) as u8))])
            .modrm(true, None, None)
            .rex(true),
        Ins::XBEGIN => ins_xbegin(ins),
        Ins::XCHG => ins_xchg(ins),
        Ins::XEND => GenAPI::new().opcode(&[0x0F, 0x01, 0xD5]),
        Ins::XGETBV => GenAPI::new().opcode(&[0x0F, 0x01, 0xD0]),
        Ins::XLAT | Ins::XLATB => GenAPI::new().opcode(&[0xD7]),
        Ins::XLATB64 => GenAPI::new().opcode(&[0x48, 0xD7]),
        Ins::XRESLDTRK => GenAPI::new().opcode(&[0xF2, 0x0F, 0x01, 0xE9]),

        Ins::XRSTOR | Ins::XRSTOR64 => {
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(5), None)
        }
        Ins::XRSTORS | Ins::XRSTORS64 => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(3), None)
            .rex(true),
        Ins::XSAVE | Ins::XSAVE64 => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(4), None)
            .rex(true),
        Ins::XSAVEC | Ins::XSAVEC64 => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(4), None)
            .rex(true),
        Ins::XSAVEOPT | Ins::XSAVEOPT64 => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(6), None)
            .rex(true),
        Ins::XSAVES | Ins::XSAVES64 => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(5), None)
            .rex(true),
        Ins::XSETBV => GenAPI::new().opcode(&[0x0F, 0x01, 0xD1]),
        Ins::XSUSLDTRK => GenAPI::new().opcode(&[0xF2, 0x0F, 0x01, 0xE8]),
        Ins::XTEST => GenAPI::new().opcode(&[0x0F, 0x01, 0xD6]),
        // sha.asm
        Ins::SHA1MSG1 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xC9])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true),
        Ins::SHA1NEXTE => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xC8])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::SHA1MSG2 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCA])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Ins::SHA1RNDS4 => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0xCC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true)
            .imm_atindex(2, 1),
        Ins::SHA256RNDS2 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCB])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true),
        Ins::SHA256MSG2 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true),
        Ins::SHA256MSG1 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true),

        // fxd
        Ins::WRGSBASE => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(3), None)
            .prefix(0xF3)
            .rex(true),
        Ins::WRFSBASE => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(2), None)
            .prefix(0xF3)
            .rex(true),
        Ins::LIDT => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(3), None)
            .rex(true)
            .can_h66(false),
        Ins::LGDT => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(2), None)
            .rex(true)
            .can_h66(false),
        Ins::LOCK => GenAPI::new().opcode(&[0xF0]),
        Ins::REPNE | Ins::REPNZ => GenAPI::new().opcode(&[0xF2]),
        Ins::REP | Ins::REPE | Ins::REPZ => GenAPI::new().opcode(&[0xF3]),
        _ => panic!("haha"),
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

fn ins_xbegin(_: &Instruction) -> GenAPI {
    GenAPI::new().opcode(&[0xC7, 0xF8]).imm_atindex(0, 4)
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

/*
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
*/

fn ins_bt(ins: &Instruction, opc_noimm: &[u8], opc_imm: &[u8], _: u8, modrm: u8) -> GenAPI {
    let mut api = GenAPI::new().rex(true);
    if let Some(Operand::Imm(_)) = ins.src() {
        api = api
            .opcode(opc_imm)
            .modrm(true, Some(modrm), None)
            .imm_atindex(1, 1);
    } else {
        api = api.opcode(opc_noimm).modrm(true, None, None)
    };
    api
}

fn ins_cmovcc(_: &Instruction, opc: &[u8], _: u8) -> GenAPI {
    GenAPI::new()
        .opcode(opc)
        .modrm(true, None, None)
        .rex(true)
        .ord(&[MODRM_REG, MODRM_RM])
}

fn ins_pop(ins: &Instruction, _: u8) -> GenAPI {
    match ins.dst().unwrap() {
        Operand::Reg(r) => GenAPI::new().opcode(&[0x58 + r.to_byte()]).rex(true),
        Operand::SegReg(r) => match r {
            Register::DS => GenAPI::new().opcode(&[0x1F]),
            Register::ES => GenAPI::new().opcode(&[0x07]),
            Register::SS => GenAPI::new().opcode(&[0x17]),
            Register::FS => GenAPI::new().opcode(&[0x0F, 0xA1]),
            Register::GS => GenAPI::new().opcode(&[0x0F, 0xA9]),
            Register::CS => GenAPI::new().opcode(&[0x90]),
            _ => invalid(34),
        },
        Operand::Mem(_) => GenAPI::new()
            .opcode(&[0x8F])
            .rex(true)
            .modrm(true, None, Some(0)),
        _ => invalid(33),
    }
}

fn ins_push(ins: &Instruction, _: u8) -> GenAPI {
    match ins.dst().unwrap() {
        Operand::Reg(r) => GenAPI::new().opcode(&[0x50 + r.to_byte()]).rex(true),
        Operand::SegReg(r) => match r {
            Register::CS => GenAPI::new().opcode(&[0x0E]),
            Register::SS => GenAPI::new().opcode(&[0x16]),
            Register::DS => GenAPI::new().opcode(&[0x1E]),
            Register::ES => GenAPI::new().opcode(&[0x06]),
            Register::FS => GenAPI::new().opcode(&[0x0F, 0xA0]),
            Register::GS => GenAPI::new().opcode(&[0x0F, 0xA8]),
            _ => invalid(32),
        },
        Operand::Imm(nb) => match nb.size() {
            Size::Byte => GenAPI::new().opcode(&[0x6A]).imm_atindex(0, 1),
            Size::Word | Size::Dword => GenAPI::new().opcode(&[0x68]).imm_atindex(0, 4),
            _ => invalid(31),
        },
        Operand::Mem(_) => GenAPI::new()
            .opcode(&[0xFF])
            .modrm(true, Some(6), None)
            .rex(true),
        _ => invalid(30),
    }
}

fn ins_mov(ins: &Instruction, _: u8) -> GenAPI {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();
    if let Operand::Reg(r) = dst {
        match src {
            Operand::SegReg(_) => GenAPI::new()
                .opcode(&[0x8C])
                .modrm(true, None, None)
                .rex(true),
            Operand::CtrReg(_) => GenAPI::new()
                .opcode(&[0x0F, 0x20])
                .modrm(true, None, None)
                .rex(true),
            Operand::DbgReg(_) => GenAPI::new()
                .opcode(&[0x0F, 0x21])
                .modrm(true, None, None)
                .rex(true),
            Operand::Imm(_) => {
                let size = dst.size();
                let opc = match size {
                    Size::Byte => 0xB0 + r.to_byte(),
                    Size::Word | Size::Dword | Size::Qword => 0xB8 + r.to_byte(),
                    _ => invalid(29),
                };
                let size = if size == Size::Qword { 4 } else { size.into() };
                GenAPI::new().opcode(&[opc]).imm_atindex(1, size as u16)
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
            }
            Operand::Mem(_) => {
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
            }
            _ => invalid(26),
        }
    } else if let Operand::CtrReg(_) = dst {
        GenAPI::new()
            .opcode(&[0x0F, 0x22])
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
            .rex(true)
    } else if let Operand::DbgReg(_) = dst {
        GenAPI::new()
            .opcode(&[0x0F, 0x23])
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
            .rex(true)
    } else if let Operand::SegReg(_) = dst {
        match src {
            Operand::Reg(_) | Operand::Mem(_) => GenAPI::new()
                .opcode(&[0x8E])
                .modrm(true, None, None)
                .rex(true),
            _ => invalid(25),
        }
    } else if let Operand::Mem(_) | Operand::SymbolRef(_) = dst {
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
            }
            Operand::Imm(_) | Operand::SymbolRef(_) => {
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
fn add_like_ins(ins: &Instruction, opc: &[u8; 9], ovrreg: u8, _: u8) -> GenAPI {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let imm = srci.split_into_bytes();
            if let Size::Dword | Size::Word = srci.size() {
                if let Register::RAX | Register::EAX = dstr {
                    return GenAPI::new().opcode(&[opc[1]]).imm_atindex(1, 4).rex(true);
                } else if let Register::AX = dstr {
                    return GenAPI::new().opcode(&[opc[1]]).imm_atindex(1, 2).rex(true);
                }
            }
            if let Register::AL = dstr {
                return GenAPI::new().opcode(&[opc[0]]).imm_atindex(1, 1).rex(true);
            } else if let Register::AX = dstr {
                return GenAPI::new().opcode(&[opc[1]]).imm_atindex(1, 1).rex(true);
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
        }
        (Operand::Mem(dstm), Operand::Imm(srci)) => {
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
        }
        (Operand::Reg(r), Operand::Mem(_) | Operand::Reg(_)) => {
            let opc = match r.size() {
                Size::Byte => opc[7],
                Size::Word | Size::Dword | Size::Qword => opc[6],
                _ => invalid(17),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
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
        }
        _ => invalid(14),
    }
}

fn ins_cmp(ins: &Instruction, _: u8) -> GenAPI {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            let imm = srci.split_into_bytes();
            if let Size::Dword | Size::Word = srci.size() {
                if let Register::RAX | Register::EAX = dstr {
                    return GenAPI::new().opcode(&[0x3D]).imm_atindex(1, 4).rex(true);
                } else if let Register::AX = dstr {
                    return GenAPI::new().opcode(&[0x3D]).imm_atindex(1, 2).rex(true);
                }
            }
            if let Register::AL = dstr {
                return GenAPI::new().opcode(&[0x3C]).imm_atindex(1, 1).rex(true);
            } else if let Register::AX = dstr {
                return GenAPI::new().opcode(&[0x3D]).imm_atindex(1, 2).rex(true);
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
        }
        (Operand::Reg(r), Operand::Mem(_) | Operand::Reg(_)) => {
            let opc = match r.size() {
                Size::Byte => 0x3A,
                Size::Word | Size::Dword | Size::Qword => 0x3B,
                _ => invalid(10),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
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
        }
        _ => invalid(7),
    }
}

fn ins_test(ins: &Instruction, _: u8) -> GenAPI {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, src) {
        (Operand::Reg(dstr), Operand::Imm(srci)) => {
            if let Size::Dword | Size::Word = srci.size() {
                if let Register::RAX | Register::EAX = dstr {
                    return GenAPI::new().opcode(&[0xA9]).imm_atindex(1, 4).rex(true);
                } else if let Register::AX = dstr {
                    return GenAPI::new().opcode(&[0xA9]).imm_atindex(1, 2).rex(true);
                }
            }
            if let Register::AL = dstr {
                return GenAPI::new().opcode(&[0xA8]).imm_atindex(1, 1).rex(true);
            } else if let Register::AX = dstr {
                return GenAPI::new().opcode(&[0xA9]).imm_atindex(1, 2).rex(true);
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
        }
        (Operand::Reg(_) | Operand::Mem(_), Operand::Reg(_)) => {
            let opc = match dst.size() {
                Size::Byte => 0x84,
                Size::Word | Size::Dword | Size::Qword => 0x85,
                _ => invalid(3),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None, None)
                .rex(true)
        }
        _ => invalid(2),
    }
}

fn ins_imul(ins: &Instruction, _: u8) -> GenAPI {
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
            }
            _ => GenAPI::new()
                .opcode(&[0x0F, 0xAF])
                .modrm(true, None, None)
                .rex(true),
        },
    }
}

// opc[0] = r/m8, 1
// opc[1] = r/m8, cl
// opc[2] = r/m8, imm8
// opc[3] = r/m16/32/64, 1
// opc[4] = r/m16/32/64, cl
// opc[5] = r/m16/32/64, imm8
fn ins_shllike(ins: &Instruction, opc: &[u8; 6], ovr: u8, _: u8) -> GenAPI {
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
    api
}

fn ins_inclike(ins: &Instruction, opc: &[u8; 2], ovr: u8, _: u8) -> GenAPI {
    let opcd = match ins.dst().unwrap().size() {
        Size::Byte => opc[0],
        _ => opc[1],
    };
    GenAPI::new()
        .opcode(&[opcd])
        .modrm(true, Some(ovr), None)
        .rex(true)
}

fn ins_lea(_: &Instruction, _: u8) -> GenAPI {
    GenAPI::new()
        .opcode(&[0x8D])
        .modrm(true, None, None)
        .ord(&[MODRM_REG, MODRM_RM])
}

// opc[0] = rel16/32
// opc[1] = r/m
// opc[2] = rel8
fn ins_jmplike<'a>(ins: &'a Instruction, opc: [&'a [u8]; 3], addt: u8, _: u8) -> GenAPI {
    match ins.dst().unwrap() {
        Operand::Imm(i) => {
            let opc = match i.size() {
                Size::Byte => opc[2],
                _ => opc[0],
            };
            GenAPI::new().opcode(opc).imm_atindex(0, 0)
        }
        Operand::SymbolRef(s) => {
            let sz = s.reltype().unwrap_or(RelType::REL32).size();
            let base = match sz {
                1 => opc[2].to_vec(),
                _ => opc[0].to_vec(),
            };
            GenAPI::new().opcode(&base).imm_atindex(0, 0)
        }
        Operand::Reg(_) | Operand::Mem(_) => GenAPI::new()
            .opcode(opc[1])
            .modrm(true, Some(addt), None)
            .rex(true),
        _ => invalid(0),
    }
}

fn ins_divmul(ins: &Instruction, ovr: u8, _: u8) -> GenAPI {
    let opc = match ins.dst().unwrap().size() {
        Size::Byte => [0xF6],
        _ => [0xF7],
    };
    GenAPI::new().opcode(&opc).modrm(true, Some(ovr), None)
}

/*
fn ins_empty(ins: &Instruction) -> Vec<u8> {
    if let Some(Operand::Imm(n)) = ins.get_opr(0) {
        vec![0x00; n.get_as_u32() as usize]
    } else {
        vec![]
    }
}
*/

fn ins_in(ins: &Instruction, _: u8) -> GenAPI {
    if let Operand::Reg(_) = ins.src().unwrap() {
        let sz = ins.dst().unwrap().size();
        if sz == Size::Byte {
            GenAPI::new().opcode(&[0xEC]).fixed_size(Size::Byte)
        } else {
            GenAPI::new().opcode(&[0xED]).fixed_size(sz)
        }
    } else {
        if ins.size() == Size::Byte {
            GenAPI::new().opcode(&[0xE4]).imm_atindex(1, 1)
        } else {
            GenAPI::new().opcode(&[0xE5]).imm_atindex(1, 1)
        }
    }
}

fn ins_out(ins: &Instruction, _: u8) -> GenAPI {
    let sz = ins.src().unwrap().size();
    if let Operand::Reg(_) = ins.dst().unwrap() {
        if sz == Size::Byte {
            GenAPI::new()
                .opcode(&[0xEE])
                .fixed_size(Size::Byte)
                .can_h66(false)
        } else {
            GenAPI::new().opcode(&[0xEF]).fixed_size(sz)
        }
    } else {
        if sz == Size::Byte {
            GenAPI::new().opcode(&[0xE6]).imm_atindex(0, 1)
        } else {
            GenAPI::new()
                .opcode(&[0xE7])
                .imm_atindex(0, 1)
                .fixed_size(sz)
        }
    }
}

fn ins_shrtjmp(_: &Instruction, opc: Vec<u8>) -> GenAPI {
    GenAPI::new().opcode(&opc).imm_atindex(0, 1)
}

// ==============================
// Utils

fn invalid(ctx: i32) -> ! {
    panic!("Unexpected thing that should not happen - code {ctx}")
}
