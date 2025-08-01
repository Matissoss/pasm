// pasm - src/core/comp.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    consts::*,
    core::api::*,
    core::apx::*,
    core::evex::*,
    shr::{
        ast::{IVariant, Instruction, Operand},
        ins::Mnemonic,
        num::Number,
        reg::{Purpose as RPurpose, Register},
        reloc::RelType,
        size::Size,
    },
};

use OpOrd::*;

pub fn get_genapi(ins: &'_ Instruction, bits: u8) -> GenAPI {
    match ins.mnemonic {
        Mnemonic::IN => ins_in(ins, bits),
        Mnemonic::OUT => ins_out(ins, bits),

        Mnemonic::BYTELE | Mnemonic::BYTEBE => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 1)
            .fixed_size(Size::Byte),
        Mnemonic::WORDLE | Mnemonic::WORDBE => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 2)
            .fixed_size(Size::Byte)
            .imm_is_be(ins.mnemonic != Mnemonic::WORDLE),
        Mnemonic::DWORDLE | Mnemonic::DWORDBE => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 4)
            .fixed_size(Size::Byte)
            .imm_is_be(ins.mnemonic != Mnemonic::DWORDLE),
        Mnemonic::QWORDBE | Mnemonic::QWORDLE => GenAPI::new()
            .opcode(&[])
            .imm_atindex(0, 8)
            .fixed_size(Size::Byte)
            .imm_is_be(ins.mnemonic != Mnemonic::QWORDLE),
        Mnemonic::ASCII | Mnemonic::STRING => GenAPI::new().opcode(&[]).imm_atindex(0, 0),

        Mnemonic::EMPTY => GenAPI::new()
            .opcode(&[])
            .fixed_size(Size::Byte)
            .imm_atindex(0, 0),
        Mnemonic::__LAST => GenAPI::new(),
        Mnemonic::CPUID => GenAPI::new().opcode(&[0x0F, 0xA2]),
        Mnemonic::RET => GenAPI::new().opcode(&[0xC3]),
        Mnemonic::SYSCALL => GenAPI::new().opcode(&[0x0F, 0x05]),
        Mnemonic::PUSH => ins_push(ins, bits),
        Mnemonic::POP => ins_pop(ins, bits),
        Mnemonic::MOV => ins_mov(ins, bits),
        Mnemonic::ADD => add_like_ins(
            ins,
            &[0x04, 0x05, 0x80, 0x81, 0x83, 0x00, 0x01, 0x02, 0x03],
            0,
            bits,
        ),
        Mnemonic::OR => add_like_ins(
            ins,
            &[0x0C, 0x0D, 0x80, 0x81, 0x83, 0x08, 0x09, 0x0A, 0x0B],
            1,
            bits,
        ),
        Mnemonic::AND => add_like_ins(
            ins,
            &[0x24, 0x25, 0x80, 0x81, 0x83, 0x20, 0x21, 0x22, 0x23],
            4,
            bits,
        ),
        Mnemonic::SUB => add_like_ins(
            ins,
            &[0x2C, 0x2D, 0x80, 0x81, 0x83, 0x28, 0x29, 0x2A, 0x2B],
            5,
            bits,
        ),
        Mnemonic::XOR => add_like_ins(
            ins,
            &[0x34, 0x35, 0x80, 0x81, 0x83, 0x30, 0x31, 0x32, 0x33],
            6,
            bits,
        ),
        Mnemonic::SAL | Mnemonic::SHL => {
            ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 4, bits)
        }
        Mnemonic::SHR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 5, bits),
        Mnemonic::SAR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 7, bits),
        Mnemonic::TEST => ins_test(ins, bits),
        Mnemonic::INC => ins_inclike(ins, &[0xFE, 0xFF], 0, bits),
        Mnemonic::DEC => ins_inclike(ins, &[0xFE, 0xFF], 1, bits),
        Mnemonic::NOT => ins_inclike(ins, &[0xF6, 0xF7], 2, bits),
        Mnemonic::NEG => ins_inclike(ins, &[0xF6, 0xF7], 3, bits),
        Mnemonic::CMP => ins_cmp(ins, bits),
        Mnemonic::IMUL => ins_imul(ins, bits),
        Mnemonic::DIV => ins_divmul(ins, 6, bits),
        Mnemonic::IDIV => ins_divmul(ins, 7, bits),
        Mnemonic::MUL => ins_divmul(ins, 4, bits),
        Mnemonic::JMP => ins_jmplike(ins, [&[0xE9], &[0xFF], &[0xEB]], 4, bits),
        Mnemonic::CALL => ins_jmplike(ins, [&[0xE8], &[0xFF], &[0xE8]], 2, bits),

        // jcc
        Mnemonic::JA => ins_jmplike(ins, [&[0x0F, 0x87], &[], &[0x77]], 0, bits),
        Mnemonic::JB => ins_jmplike(ins, [&[0x0F, 0x82], &[], &[0x72]], 0, bits),
        Mnemonic::JC => ins_jmplike(ins, [&[0x0F, 0x82], &[], &[0x72]], 0, bits),
        Mnemonic::JO => ins_jmplike(ins, [&[0x0F, 0x80], &[], &[0x70]], 0, bits),
        Mnemonic::JP => ins_jmplike(ins, [&[0x0F, 0x8A], &[], &[0x7A]], 0, bits),
        Mnemonic::JS => ins_jmplike(ins, [&[0x0F, 0x88], &[], &[0x78]], 0, bits),
        Mnemonic::JL => ins_jmplike(ins, [&[0x0F, 0x8C], &[], &[0x7C]], 0, bits),
        Mnemonic::JG => ins_jmplike(ins, [&[0x0F, 0x8F], &[], &[0x7C]], 0, bits),
        Mnemonic::JE | Mnemonic::JZ => ins_jmplike(ins, [&[0x0F, 0x84], &[], &[0x74]], 0, bits),
        Mnemonic::JAE => ins_jmplike(ins, [&[0x0F, 0x83], &[], &[0x73]], 0, bits),
        Mnemonic::JBE => ins_jmplike(ins, [&[0x0F, 0x86], &[], &[0x76]], 0, bits),
        Mnemonic::JNA => ins_jmplike(ins, [&[0x0F, 0x86], &[], &[0x76]], 0, bits),
        Mnemonic::JNB => ins_jmplike(ins, [&[0x0F, 0x83], &[], &[0x73]], 0, bits),
        Mnemonic::JNC => ins_jmplike(ins, [&[0x0F, 0x83], &[], &[0x73]], 0, bits),
        Mnemonic::JNG => ins_jmplike(ins, [&[0x0F, 0x8E], &[], &[0x7E]], 0, bits),
        Mnemonic::JNL => ins_jmplike(ins, [&[0x0F, 0x8D], &[], &[0x7D]], 0, bits),
        Mnemonic::JNO => ins_jmplike(ins, [&[0x0F, 0x81], &[], &[0x71]], 0, bits),
        Mnemonic::JNP => ins_jmplike(ins, [&[0x0F, 0x8B], &[], &[0x7B]], 0, bits),
        Mnemonic::JNS => ins_jmplike(ins, [&[0x0F, 0x89], &[], &[0x79]], 0, bits),
        Mnemonic::JPE => ins_jmplike(ins, [&[0x0F, 0x8A], &[], &[0x7A]], 0, bits),
        Mnemonic::JPO => ins_jmplike(ins, [&[0x0F, 0x8B], &[], &[0x7B]], 0, bits),
        Mnemonic::JNE | Mnemonic::JNZ => ins_jmplike(ins, [&[0x0F, 0x85], &[], &[0x75]], 0, bits),
        Mnemonic::JLE => ins_jmplike(ins, [&[0x0F, 0x8E], &[], &[0x7E]], 0, bits),
        Mnemonic::JGE => ins_jmplike(ins, [&[0x0F, 0x8D], &[], &[0x7D]], 0, bits),
        Mnemonic::JNAE => ins_jmplike(ins, [&[0x0F, 0x82], &[], &[0x72]], 0, bits),
        Mnemonic::JNBE => ins_jmplike(ins, [&[0x0F, 0x87], &[], &[0x77]], 0, bits),
        Mnemonic::JNGE => ins_jmplike(ins, [&[0x0F, 0x8C], &[], &[0x7C]], 0, bits),
        Mnemonic::JNLE => ins_jmplike(ins, [&[0x0F, 0x8F], &[], &[0x7F]], 0, bits),

        Mnemonic::JCXZ => ins_jmplike(ins, [&[], &[], &[0xE3]], 0, bits),
        Mnemonic::JECXZ => ins_jmplike(ins, [&[], &[], &[0xE3]], 0, bits),
        Mnemonic::JRCXZ => ins_jmplike(ins, [&[], &[], &[0xE3]], 0, bits),

        Mnemonic::LEA => ins_lea(ins, bits),

        Mnemonic::NOP => GenAPI::new().opcode(&[0x90]),

        Mnemonic::PUSHF | Mnemonic::PUSHFD | Mnemonic::PUSHFQ => GenAPI::new().opcode(&[0x9C]),
        Mnemonic::POPF | Mnemonic::POPFD | Mnemonic::POPFQ => GenAPI::new().opcode(&[0x9D]),

        Mnemonic::CLFLUSH => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(7))
            .rex(),

        Mnemonic::PAUSE => GenAPI::new().opcode(&[0xF3, 0x90]),
        Mnemonic::MWAIT => GenAPI::new().opcode(&[0x0F, 0x01, 0xC9]),

        Mnemonic::CMOVA => ins_cmovcc(ins, &[0x0F, 0x47], bits),
        Mnemonic::CMOVAE => ins_cmovcc(ins, &[0x0F, 0x43], bits),
        Mnemonic::CMOVB => ins_cmovcc(ins, &[0x0F, 0x42], bits),
        Mnemonic::CMOVBE => ins_cmovcc(ins, &[0x0F, 0x46], bits),
        Mnemonic::CMOVC => ins_cmovcc(ins, &[0x0F, 0x42], bits),
        Mnemonic::CMOVNC => ins_cmovcc(ins, &[0x0F, 0x43], bits),
        Mnemonic::CMOVE => ins_cmovcc(ins, &[0x0F, 0x44], bits),
        Mnemonic::CMOVG => ins_cmovcc(ins, &[0x0F, 0x4F], bits),
        Mnemonic::CMOVGE => ins_cmovcc(ins, &[0x0F, 0x4D], bits),
        Mnemonic::CMOVL => ins_cmovcc(ins, &[0x0F, 0x4C], bits),
        Mnemonic::CMOVLE => ins_cmovcc(ins, &[0x0F, 0x4E], bits),
        Mnemonic::CMOVNA => ins_cmovcc(ins, &[0x0F, 0x46], bits),
        Mnemonic::CMOVNB => ins_cmovcc(ins, &[0x0F, 0x43], bits),
        Mnemonic::CMOVNBE => ins_cmovcc(ins, &[0x0F, 0x47], bits),
        Mnemonic::CMOVNE => ins_cmovcc(ins, &[0x0F, 0x45], bits),
        Mnemonic::CMOVNG => ins_cmovcc(ins, &[0x0F, 0x4E], bits),
        Mnemonic::CMOVNGE => ins_cmovcc(ins, &[0x0F, 0x4C], bits),
        Mnemonic::CMOVNL => ins_cmovcc(ins, &[0x0F, 0x4D], bits),
        Mnemonic::CMOVNLE => ins_cmovcc(ins, &[0x0F, 0x4F], bits),
        Mnemonic::CMOVNAE => ins_cmovcc(ins, &[0x0F, 0x42], bits),
        Mnemonic::CMOVNO => ins_cmovcc(ins, &[0x0F, 0x41], bits),
        Mnemonic::CMOVNP => ins_cmovcc(ins, &[0x0F, 0x4B], bits),
        Mnemonic::CMOVNS => ins_cmovcc(ins, &[0x0F, 0x49], bits),
        Mnemonic::CMOVNZ => ins_cmovcc(ins, &[0x0F, 0x45], bits),
        Mnemonic::CMOVO => ins_cmovcc(ins, &[0x0F, 0x40], bits),
        Mnemonic::CMOVP => ins_cmovcc(ins, &[0x0F, 0x4A], bits),
        Mnemonic::CMOVPO => ins_cmovcc(ins, &[0x0F, 0x4B], bits),
        Mnemonic::CMOVS => ins_cmovcc(ins, &[0x0F, 0x48], bits),
        Mnemonic::CMOVZ => ins_cmovcc(ins, &[0x0F, 0x44], bits),
        Mnemonic::CMOVPE => ins_cmovcc(ins, &[0x0F, 0x4A], bits),

        // SSE
        Mnemonic::MOVSS => {
            let mut api = GenAPI::new().modrm(true, None).rex().opcode_prefix(0xF3);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVHLPS => GenAPI::new()
            .modrm(true, None)
            .rex()
            .opcode(&[0x0F, 0x12])
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVLHPS => GenAPI::new()
            .modrm(true, None)
            .rex()
            .opcode(&[0x0F, 0x16])
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVAPS => {
            let mut api = GenAPI::new().modrm(true, None).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x29]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVUPS => {
            let mut api = GenAPI::new().modrm(true, None).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVLPS => {
            let mut api = GenAPI::new().modrm(true, None).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x13]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x12]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVHPS => {
            let mut api = GenAPI::new().modrm(true, None).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x17]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x16]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }

        Mnemonic::ADDPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x58])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ADDSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x58])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SUBPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x5C])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SUBSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x5C])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MULPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x59])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MULSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x59])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DIVPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x5E])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DIVSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x5E])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MINPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x5D])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MINSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x5D])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MAXPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x5F])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MAXSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x5F])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::RSQRTPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x52])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::RSQRTSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x52])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SHUFPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0xC6])
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SQRTPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x51])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SQRTSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x51])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CMPPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0xC2])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::CMPSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0xC2])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::RCPPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x53])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::RCPSS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x53])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::UCOMISS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x2E])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::COMISS => GenAPI::new()
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x2F])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ORPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x56])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ANDPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x54])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ANDNPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x55])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::XORPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x57])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::UNPCKLPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x14])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::UNPCKHPS => GenAPI::new()
            .modrm(true, None)
            .opcode(&[0x0F, 0x15])
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),

        // SSE2
        Mnemonic::MOVNTI => GenAPI::new().opcode(&[0x0F, 0xC3]).modrm(true, None).rex(),

        Mnemonic::MFENCE => GenAPI::new().opcode(&[0xF0, 0xAE, 0xF0]),
        Mnemonic::LFENCE => GenAPI::new().opcode(&[0xF0, 0xAE, 0xE8]),

        Mnemonic::MOVNTPD => GenAPI::new()
            .opcode(&[0x0F, 0x2B])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex(),
        Mnemonic::MOVNTDQ => GenAPI::new()
            .opcode(&[0x0F, 0xE7])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex(),
        Mnemonic::MOVAPD => {
            let mut api = GenAPI::new().modrm(true, None).opcode_prefix(0x66).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x29]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVUPD => {
            let mut api = GenAPI::new().modrm(true, None).opcode_prefix(0x66).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVLPD => {
            let mut api = GenAPI::new().modrm(true, None).opcode_prefix(0x66).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x13]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x12]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVHPD => {
            let mut api = GenAPI::new().modrm(true, None).opcode_prefix(0x66).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x17]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x16]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVSD => {
            let mut api = GenAPI::new().modrm(true, None).opcode_prefix(0xF2).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVDQA => {
            let mut api = GenAPI::new().modrm(true, None).opcode_prefix(0x66).rex();
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x7F]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x6F]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVDQ2Q => GenAPI::new()
            .opcode(&[0x0F, 0xD6])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex(),
        Mnemonic::MOVQ2DQ => GenAPI::new()
            .opcode(&[0x0F, 0xD6])
            .opcode_prefix(0xF3)
            .modrm(true, None)
            .rex(),

        Mnemonic::MOVMSKPD => GenAPI::new()
            .opcode(&[0x0F, 0x50])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::ADDPD => GenAPI::new()
            .opcode(&[0x0F, 0x58])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ADDSD => GenAPI::new()
            .opcode(&[0x0F, 0x58])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SUBPD => GenAPI::new()
            .opcode(&[0x0F, 0x5C])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SUBSD => GenAPI::new()
            .opcode(&[0x0F, 0x5C])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MULPD => GenAPI::new()
            .opcode(&[0x0F, 0x59])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MULSD => GenAPI::new()
            .opcode(&[0x0F, 0x59])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DIVPD => GenAPI::new()
            .opcode(&[0x0F, 0x5E])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DIVSD => GenAPI::new()
            .opcode(&[0x0F, 0x5E])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MINPD => GenAPI::new()
            .opcode(&[0x0F, 0x5D])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MINSD => GenAPI::new()
            .opcode(&[0x0F, 0x5D])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MAXPD => GenAPI::new()
            .opcode(&[0x0F, 0x5F])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MAXSD => GenAPI::new()
            .opcode(&[0x0F, 0x5F])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SQRTPD => GenAPI::new()
            .opcode(&[0x0F, 0x51])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SQRTSD => GenAPI::new()
            .opcode(&[0x0F, 0x51])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CMPPD => GenAPI::new()
            .opcode(&[0x0F, 0xC2])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CMPSD => GenAPI::new()
            .opcode(&[0x0F, 0xC2])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::COMISD => GenAPI::new()
            .opcode(&[0x0F, 0x2F])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::UCOMISD => GenAPI::new()
            .opcode(&[0x0F, 0x2E])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ORPD => GenAPI::new()
            .opcode(&[0x0F, 0x56])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ANDPD => GenAPI::new()
            .opcode(&[0x0F, 0x54])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ANDNPD => GenAPI::new()
            .opcode(&[0x0F, 0x55])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::XORPD => GenAPI::new()
            .opcode(&[0x0F, 0x57])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PSHUFLW => GenAPI::new()
            .opcode(&[0x0F, 0x70])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PSHUFHW => GenAPI::new()
            .opcode(&[0x0F, 0x70])
            .opcode_prefix(0xF3)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PSHUFD => GenAPI::new()
            .opcode(&[0x0F, 0x70])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::PSLLDQ => GenAPI::new()
            .opcode(&[0x0F, 0x73])
            .opcode_prefix(0x66)
            .modrm(true, Some(7))
            .rex()
            .imm_atindex(1, 1),
        Mnemonic::PSRLDQ => GenAPI::new()
            .opcode(&[0x0F, 0x73])
            .opcode_prefix(0x66)
            .modrm(true, Some(3))
            .rex()
            .imm_atindex(1, 1),
        Mnemonic::PUNPCKHQDQ => GenAPI::new()
            .opcode(&[0x0F, 0x6D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .opcode_prefix(0x66)
            .rex(),
        Mnemonic::PUNPCKLQDQ => GenAPI::new()
            .opcode(&[0x0F, 0x6C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .opcode_prefix(0x66)
            .rex(),
        // MMX/SSE2
        Mnemonic::MOVD | Mnemonic::MOVQ => {
            let mut api = GenAPI::new().modrm(true, None).rex();
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66);
            }
            if let Some(Operand::Register(r)) = ins.dst() {
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
        Mnemonic::PADDB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFC])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PADDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFD])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PADDD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFE])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PADDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD4])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PADDUSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDC])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PADDUSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDD])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PADDSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEC])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PADDSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xED])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSUBUSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD8])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSUBUSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD9])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PSUBB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF8])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSUBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF9])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSUBD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFA])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSUBQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFB])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::MASKMOVDQU => GenAPI::new()
            .opcode(&[0x0F, 0xF7])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex(),

        Mnemonic::PSUBSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE8])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE9])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PMULLW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD5])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PMULHW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE5])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PMULUDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF4])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PMADDWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF5])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PCMPEQB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x74])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PCMPEQW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x75])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PCMPEQD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x76])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PCMPGTB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x64])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PCMPGTW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x65])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PCMPGTD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x66])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PACKUSWB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x67])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PACKSSWB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x63])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PACKSSDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x6B])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PUNPCKLBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x60])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PUNPCKLWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x61])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PUNPCKLDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x62])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PUNPCKHBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x68])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PUNPCKHWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x69])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PUNPCKHDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x6A])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PSLLQ => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x73])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(6));
            } else {
                api = api
                    .opcode(&[0x0F, 0xF3])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSLLD => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x72])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(6));
            } else {
                api = api
                    .opcode(&[0x0F, 0xF2])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSLLW => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x71])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(6));
            } else {
                api = api
                    .opcode(&[0x0F, 0xF1])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSRLW => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x71])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(2));
            } else {
                api = api
                    .opcode(&[0x0F, 0xD1])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSRLD => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x72])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(2));
            } else {
                api = api
                    .opcode(&[0x0F, 0xD2])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSRLQ => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x73])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(2));
            } else {
                api = api
                    .opcode(&[0x0F, 0xD3])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSRAW => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x71])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(4));
            } else {
                api = api
                    .opcode(&[0x0F, 0xE1])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSRAD => {
            let mut api = GenAPI::new();
            if let Some(Operand::Imm(_)) = ins.src() {
                api = api
                    .opcode(&[0x0F, 0x72])
                    .imm_atindex(1, 1)
                    .modrm(true, Some(4));
            } else {
                api = api
                    .opcode(&[0x0F, 0xE2])
                    .ord(&[MODRM_REG, MODRM_RM])
                    .modrm(true, None);
            }
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::POR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEB])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PAND => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDB])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PANDN => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDF])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PXOR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEF])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::EMMS => GenAPI::new().opcode(&[0x0F, 0x77]),

        // sse3
        Mnemonic::ADDSUBPD => GenAPI::new()
            .opcode(&[0x0F, 0xD0])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::ADDSUBPS => GenAPI::new()
            .opcode(&[0x0F, 0xD0])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),

        Mnemonic::HADDPD => GenAPI::new()
            .opcode(&[0x0F, 0x7C])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::HADDPS => GenAPI::new()
            .opcode(&[0x0F, 0x7C])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::HSUBPD => GenAPI::new()
            .opcode(&[0x0F, 0x7D])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::HSUBPS => GenAPI::new()
            .opcode(&[0x0F, 0x7D])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),

        Mnemonic::MOVSLDUP => GenAPI::new()
            .opcode(&[0x0F, 0x12])
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVSHDUP => GenAPI::new()
            .opcode(&[0x0F, 0x16])
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVDDUP => GenAPI::new()
            .opcode(&[0x0F, 0x12])
            .modrm(true, None)
            .opcode_prefix(0xF2)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::LDDQU => GenAPI::new()
            .opcode(&[0x0F, 0xF0])
            .modrm(true, None)
            .opcode_prefix(0xF2)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::MONITOR => GenAPI::new().opcode(&[0x0F, 0x01, 0xC8]),

        // ssse3
        Mnemonic::PABSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1C])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PABSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1D])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PABSD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1E])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PSIGNB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x08])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSIGNW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x09])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PSIGND => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x0A])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }

        Mnemonic::PSHUFB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x00])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PHADDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x01])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PHADDD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x02])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PHADDSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x03])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PHSUBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x05])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PHSUBD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x06])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PHSUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x07])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PALIGNR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x3A, 0x0F])
                .modrm(true, None)
                .imm_atindex(2, 1)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PMULHRSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x0B])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PMADDUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x04])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        // sse4
        Mnemonic::DPPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x40])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DPPD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x41])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PTEST => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x17])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PEXTRW => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x15])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1),
        Mnemonic::PEXTRB => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x14])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1),
        Mnemonic::PEXTRD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x16])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1),
        Mnemonic::PEXTRQ => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x16])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1),
        Mnemonic::PINSRB => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x20])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PINSRD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x22])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PINSRQ => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x22])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMAXSB => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3C])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMAXSD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3D])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMAXUW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3E])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMINSB => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x38])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMINSD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x39])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMINUW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3A])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMULDQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x28])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMULLD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x40])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::BLENDPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0C])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .imm_atindex(2, 1)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::BLENDPD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0D])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .imm_atindex(2, 1)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PBLENDW => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0E])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .imm_atindex(2, 1)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPEQQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x29])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ROUNDPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x08])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .imm_atindex(2, 1)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ROUNDPD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x09])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .imm_atindex(2, 1)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ROUNDSS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0A])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .imm_atindex(2, 1)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ROUNDSD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0B])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .imm_atindex(2, 1)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MPSADBW => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x42])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .imm_atindex(2, 1)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPGTQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x37])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::BLENDVPS => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x14])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::BLENDVPD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x15])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PBLENDVB => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x10])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::INSERTPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x21])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PACKUSDW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x2B])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVNTDQA => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x2A])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPESTRM => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x60])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPESTRI => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x61])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPISTRM => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x62])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPISTRI => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x63])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::EXTRACTPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x17])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_RM, MODRM_REG]),
        Mnemonic::PHMINPOSUW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x41])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .rex()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CRC32 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF0])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::POPCNT => GenAPI::new()
            .opcode(&[0x0F, 0xB8])
            .opcode_prefix(0xF3)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),

        // AVX
        Mnemonic::VMOVDQA => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x7F]);
            } else {
                api = api.opcode(&[0x6F]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVSLDUP => GenAPI::new()
            .opcode(&[0x12])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VLDDQU => GenAPI::new()
            .opcode(&[0xF0])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VMOVDDUP => GenAPI::new()
            .opcode(&[0x12])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VMOVSHDUP => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VMOVMSKPD => GenAPI::new()
            .opcode(&[0x50])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VMOVAPS => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x29]);
            } else {
                api = api.opcode(&[0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVAPD => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x29]);
            } else {
                api = api.opcode(&[0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVUPS => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x11]);
            } else {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVUPD => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x11]);
            } else {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::VADDPS => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSUBPS => GenAPI::new()
            .opcode(&[0xD0])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSUBPD => GenAPI::new()
            .opcode(&[0xD0])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VHADDPS => GenAPI::new()
            .opcode(&[0x7C])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VHADDPD => GenAPI::new()
            .opcode(&[0x7C])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VHSUBPS => GenAPI::new()
            .opcode(&[0x7D])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VHSUBPD => GenAPI::new()
            .opcode(&[0x7D])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDPD => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSS => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSD => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSUBPS => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSUBPD => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSUBSS => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSUBSD => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VMULPS => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMULPD => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMULSS => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMULSD => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VDIVPS => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VDIVPD => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VDIVSS => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VDIVSD => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VRCPPS => GenAPI::new()
            .opcode(&[0x53])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VRCPSS => GenAPI::new()
            .opcode(&[0x53])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::VSQRTPS => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VSQRTPD => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VSQRTSS => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VSQRTSD => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VRSQRTPS => GenAPI::new()
            .opcode(&[0x52])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VRSQRTSS => GenAPI::new()
            .opcode(&[0x52])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMULDQ => GenAPI::new()
            .opcode(&[0x28])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULLD => GenAPI::new()
            .opcode(&[0x40])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINSB => GenAPI::new()
            .opcode(&[0x38])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINSD => GenAPI::new()
            .opcode(&[0x39])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINUB => GenAPI::new()
            .opcode(&[0xDA])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINUW => GenAPI::new()
            .opcode(&[0x3A])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXSB => GenAPI::new()
            .opcode(&[0x3C])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXSD => GenAPI::new()
            .opcode(&[0x3D])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXUB => GenAPI::new()
            .opcode(&[0xDE])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXUW => GenAPI::new()
            .opcode(&[0x3E])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VMINPS => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMINPD => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMINSS => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMINSD => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMAXPS => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMAXPD => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMAXSS => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMAXSD => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VORPS => GenAPI::new()
            .opcode(&[0x56])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VORPD => GenAPI::new()
            .opcode(&[0x56])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VANDPS => GenAPI::new()
            .opcode(&[0x54])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VANDPD => GenAPI::new()
            .opcode(&[0x54])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VANDNPD => GenAPI::new()
            .opcode(&[0x55])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VXORPD => GenAPI::new()
            .opcode(&[0x57])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VBLENDVPS => GenAPI::new()
            .opcode(&[0x4A])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VPBLENDVB => GenAPI::new()
            .opcode(&[0x4C])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VBLENDVPD => GenAPI::new()
            .opcode(&[0x4B])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),

        Mnemonic::VPHMINPOSUW => GenAPI::new()
            .opcode(&[0x41])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VEXTRACTPS => GenAPI::new()
            .opcode(&[0x17])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),

        Mnemonic::VMOVNTDQA => GenAPI::new()
            .opcode(&[0x2A])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPACKUSDW => GenAPI::new()
            .opcode(&[0x2B])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPESTRM => GenAPI::new()
            .opcode(&[0x60])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VPCMPESTRI => GenAPI::new()
            .opcode(&[0x61])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VPCMPISTRM => GenAPI::new()
            .opcode(&[0x62])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VPCMPISTRI => GenAPI::new()
            .opcode(&[0x63])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VINSERTPS => GenAPI::new()
            .opcode(&[0x21])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VBLENDPS => GenAPI::new()
            .opcode(&[0x0C])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VBLENDPD => GenAPI::new()
            .opcode(&[0x0D])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VPCMPGTQ => GenAPI::new()
            .opcode(&[0x37])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPEQQ => GenAPI::new()
            .opcode(&[0x29])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMPSADBW => GenAPI::new()
            .opcode(&[0x42])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VROUNDSS => GenAPI::new()
            .opcode(&[0x0A])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VROUNDSD => GenAPI::new()
            .opcode(&[0x0B])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VROUNDPS => GenAPI::new()
            .opcode(&[0x08])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VROUNDPD => GenAPI::new()
            .opcode(&[0x09])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VPBLENDW => GenAPI::new()
            .opcode(&[0x0E])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VCMPPD => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VANDNPS => GenAPI::new()
            .opcode(&[0x55])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VXORPS => GenAPI::new()
            .opcode(&[0x57])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPTEST => GenAPI::new()
            .opcode(&[0x17])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VDPPS => GenAPI::new()
            .opcode(&[0x40])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VDPPD => GenAPI::new()
            .opcode(&[0x41])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VCMPPS => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VCMPSS => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VCMPSD => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VUCOMISS => GenAPI::new()
            .opcode(&[0x2E])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VUCOMISD => GenAPI::new()
            .opcode(&[0x2E])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCOMISS => GenAPI::new()
            .opcode(&[0x2F])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCOMISD => GenAPI::new()
            .opcode(&[0x2F])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VUNPCKLPS => GenAPI::new()
            .opcode(&[0x14])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VUNPCKHPS => GenAPI::new()
            .opcode(&[0x15])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSHUFPS => GenAPI::new()
            .opcode(&[0xC6])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VMOVSS => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().pp(0xF3).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x11]);
            } else if ins.src().unwrap().is_mem() {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, MODRM_RM])
            } else {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVSD => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().pp(0xF2).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x11]);
            } else if ins.src().unwrap().is_mem() {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, MODRM_RM])
            } else {
                api = api.opcode(&[0x10]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVLPS => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x13]);
            } else if ins.src().unwrap().is_mem() {
                api = api.opcode(&[0x12]).ord(&[MODRM_REG, MODRM_RM])
            } else {
                api = api.opcode(&[0x12]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVLPD => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x13]);
            } else if ins.src().unwrap().is_mem() {
                api = api.opcode(&[0x12]).ord(&[MODRM_REG, MODRM_RM])
            } else {
                api = api.opcode(&[0x12]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVHPS => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x17]);
            } else if ins.src().unwrap().is_mem() {
                api = api.opcode(&[0x16]).ord(&[MODRM_REG, MODRM_RM])
            } else {
                api = api.opcode(&[0x16]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVHPD => {
            let mut api = GenAPI::new()
                .modrm(true, None)
                .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false));
            if ins.dst().unwrap().is_mem() {
                api = api.opcode(&[0x17]);
            } else if ins.src().unwrap().is_mem() {
                api = api.opcode(&[0x16]).ord(&[MODRM_REG, MODRM_RM])
            } else {
                api = api.opcode(&[0x16]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
            }
            api
        }
        Mnemonic::VMOVLHPS => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMOVHLPS => GenAPI::new()
            .opcode(&[0x12])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPEXTRB => GenAPI::new()
            .opcode(&[0x14])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Mnemonic::VPEXTRW => GenAPI::new()
            .opcode(&[0xC5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Mnemonic::VPEXTRD => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Mnemonic::VPEXTRQ => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(true))
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Mnemonic::VPINSRB => GenAPI::new()
            .opcode(&[0x20])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VPINSRD => GenAPI::new()
            .opcode(&[0x22])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VPINSRQ => GenAPI::new()
            .opcode(&[0x22])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(true))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),

        // MMX derived part 1
        Mnemonic::VPOR => GenAPI::new()
            .opcode(&[0xEB])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMOVD | Mnemonic::VMOVQ => {
            let mut api = GenAPI::new().modrm(true, None).vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vex_we(ins.mnemonic == Mnemonic::VMOVQ),
            );
            if let Some(Operand::Register(r)) = ins.dst() {
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
        Mnemonic::VPAND => GenAPI::new()
            .opcode(&[0xDB])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPXOR => GenAPI::new()
            .opcode(&[0xEF])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDB => GenAPI::new()
            .opcode(&[0xFC])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDW => GenAPI::new()
            .opcode(&[0xFD])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDD => GenAPI::new()
            .opcode(&[0xFE])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDQ => GenAPI::new()
            .opcode(&[0xD4])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBB => GenAPI::new()
            .opcode(&[0xF8])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBW => GenAPI::new()
            .opcode(&[0xF9])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBD => GenAPI::new()
            .opcode(&[0xFA])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBQ => GenAPI::new()
            .opcode(&[0xFB])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPANDN => GenAPI::new()
            .opcode(&[0xDF])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSLLW => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                api = api
                    .opcode(&[0x71])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(6));
            } else {
                api = api
                    .opcode(&[0xF1])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None);
            }
            api
        }
        Mnemonic::VPSLLD => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                api = api
                    .opcode(&[0x72])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(6));
            } else {
                api = api
                    .opcode(&[0xF2])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None);
            }
            api
        }
        Mnemonic::VPSLLQ => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                api = api
                    .opcode(&[0x73])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(6));
            } else {
                api = api
                    .opcode(&[0xF3])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None);
            }
            api
        }
        Mnemonic::VPSRLW => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                api = api
                    .opcode(&[0x71])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(2));
            } else {
                api = api
                    .opcode(&[0xD1])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None);
            }
            api
        }
        Mnemonic::VPSRLD => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                api = api
                    .opcode(&[0x72])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(2));
            } else {
                api = api
                    .opcode(&[0xD2])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None);
            }
            api
        }
        Mnemonic::VPSRLQ => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                api = api
                    .opcode(&[0x73])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(2));
            } else {
                api = api
                    .opcode(&[0xD3])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None);
            }
            api
        }
        Mnemonic::VPSRAW => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                api = api
                    .opcode(&[0x71])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(4));
            } else {
                api = api
                    .opcode(&[0xE1])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None);
            }
            api
        }
        Mnemonic::VPSRAD => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                api = api
                    .opcode(&[0x72])
                    .imm_atindex(2, 1)
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .modrm(true, Some(4));
            } else {
                api = api
                    .opcode(&[0xE2])
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .modrm(true, None);
            }
            api
        }
        Mnemonic::VPSUBSB => GenAPI::new()
            .opcode(&[0xE8])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBSW => GenAPI::new()
            .opcode(&[0xE9])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDSB => GenAPI::new()
            .opcode(&[0xEC])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDSW => GenAPI::new()
            .opcode(&[0xED])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULHW => GenAPI::new()
            .opcode(&[0xE5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULLW => GenAPI::new()
            .opcode(&[0xD5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        // part 2
        Mnemonic::VPADDUSB => GenAPI::new()
            .opcode(&[0xDC])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDUSW => GenAPI::new()
            .opcode(&[0xDD])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBUSB => GenAPI::new()
            .opcode(&[0xD8])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBUSW => GenAPI::new()
            .opcode(&[0xD9])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMADDWD => GenAPI::new()
            .opcode(&[0xF5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPEQB => GenAPI::new()
            .opcode(&[0x74])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPEQW => GenAPI::new()
            .opcode(&[0x75])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPEQD => GenAPI::new()
            .opcode(&[0x76])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPGTB => GenAPI::new()
            .opcode(&[0x64])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPGTW => GenAPI::new()
            .opcode(&[0x65])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPGTD => GenAPI::new()
            .opcode(&[0x66])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPACKUSWB => GenAPI::new()
            .opcode(&[0x67])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPACKSSWB => GenAPI::new()
            .opcode(&[0x63])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPACKSSDW => GenAPI::new()
            .opcode(&[0x6B])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKLBW => GenAPI::new()
            .opcode(&[0x60])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKLWD => GenAPI::new()
            .opcode(&[0x61])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKLDQ => GenAPI::new()
            .opcode(&[0x62])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKHBW => GenAPI::new()
            .opcode(&[0x68])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKHWD => GenAPI::new()
            .opcode(&[0x69])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKHDQ => GenAPI::new()
            .opcode(&[0x6A])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        // part2a
        Mnemonic::PAVGB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE0])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PAVGW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE3])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::VPAVGB => GenAPI::new()
            .opcode(&[0xE0])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPAVGW => GenAPI::new()
            .opcode(&[0xE3])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPHADDW => GenAPI::new()
            .opcode(&[0x01])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPHADDD => GenAPI::new()
            .opcode(&[0x02])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPHSUBW => GenAPI::new()
            .opcode(&[0x05])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPHSUBD => GenAPI::new()
            .opcode(&[0x06])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VZEROUPPER => GenAPI::new().opcode(&[0xC5, 0xF8, 0x77]),
        Mnemonic::VZEROALL => GenAPI::new().opcode(&[0xC5, 0xFC, 0x77]),
        Mnemonic::VPALIGNR => GenAPI::new()
            .opcode(&[0x0F])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None)
            .imm_atindex(3, 1)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VINSERTF128 => GenAPI::new()
            .opcode(&[0x18])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .modrm(true, None)
            .imm_atindex(3, 1)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VEXTRACTF128 => GenAPI::new()
            .opcode(&[0x19])
            .vex(
                VexDetails::new()
                    .map_select(0x3A)
                    .pp(0x66)
                    .vex_we(false)
                    .vlength(Some(true)),
            )
            .modrm(true, None)
            .imm_atindex(2, 1)
            .ord(&[MODRM_RM, MODRM_REG]),
        Mnemonic::VBROADCASTSS => GenAPI::new()
            .opcode(&[0x18])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VBROADCASTSD => GenAPI::new()
            .opcode(&[0x19])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x38)
                    .vex_we(ins.needs_evex()),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VBROADCASTF128 => GenAPI::new()
            .opcode(&[0x1A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::STMXCSR => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(3))
            .rex(),
        Mnemonic::LDMXCSR => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(2))
            .rex(),
        Mnemonic::VSTMXCSR => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, Some(3)),
        Mnemonic::VLDMXCSR => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, Some(2)),
        Mnemonic::VMOVMSKPS => GenAPI::new()
            .opcode(&[0x50])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPERMILPS => {
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                GenAPI::new()
                    .modrm(true, None)
                    .opcode(&[0x04])
                    .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                    .ord(&[MODRM_REG, MODRM_RM])
                    .imm_atindex(2, 1)
            } else {
                GenAPI::new()
                    .modrm(true, None)
                    .opcode(&[0x0C])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            }
        }
        Mnemonic::VPERMILPD => {
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                GenAPI::new()
                    .modrm(true, None)
                    .opcode(&[0x05])
                    .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
                    .ord(&[MODRM_REG, MODRM_RM])
                    .imm_atindex(2, 1)
            } else {
                GenAPI::new()
                    .modrm(true, None)
                    .opcode(&[0x0D])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            }
        }
        Mnemonic::PCLMULQDQ => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x44])
            .opcode_prefix(0x66)
            .rex()
            .modrm(true, None)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPCLMULQDQ => GenAPI::new()
            .opcode(&[0x44])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPERM2F128 => GenAPI::new()
            .opcode(&[0x06])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPERM2I128 => GenAPI::new()
            .opcode(&[0x46])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        // part2c
        Mnemonic::VPINSRW => GenAPI::new()
            .opcode(&[0xC4])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXSW => GenAPI::new()
            .opcode(&[0xEE])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINSW => GenAPI::new()
            .opcode(&[0xEA])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSRLDQ => GenAPI::new()
            .opcode(&[0x73])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .imm_atindex(2, 1)
            .modrm(true, Some(3))
            .ord(&[VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSIGNB => GenAPI::new()
            .opcode(&[0x08])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSIGNW => GenAPI::new()
            .opcode(&[0x09])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSIGND => GenAPI::new()
            .opcode(&[0x0A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULUDQ => GenAPI::new()
            .opcode(&[0xF4])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULHUW => GenAPI::new()
            .opcode(&[0xE4])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULHRSW => GenAPI::new()
            .opcode(&[0x0B])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        // part2c-ext
        Mnemonic::PMAXSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEE])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PINSRW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xC4])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .imm_atindex(2, 1);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PMINSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEA])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66).rex();
            }
            api
        }
        Mnemonic::PMAXUD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3F])
            .opcode_prefix(0x66)
            .rex()
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMAXUD => GenAPI::new()
            .opcode(&[0x3F])
            .modrm(true, None)
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::PMULHUW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE4])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.opcode_prefix(0x66);
            }
            api
        }
        // fma-part1
        Mnemonic::VFMADD132PS => GenAPI::new()
            .opcode(&[0x98])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD213PS => GenAPI::new()
            .opcode(&[0xA8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD231PS => GenAPI::new()
            .opcode(&[0xB8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD132PD => GenAPI::new()
            .opcode(&[0x98])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD213PD => GenAPI::new()
            .opcode(&[0xA8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD231PD => GenAPI::new()
            .opcode(&[0xB8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD132SD => GenAPI::new()
            .opcode(&[0x99])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD213SD => GenAPI::new()
            .opcode(&[0xA9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD231SD => GenAPI::new()
            .opcode(&[0xB9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD132SS => GenAPI::new()
            .opcode(&[0x99])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD213SS => GenAPI::new()
            .opcode(&[0xA9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD231SS => GenAPI::new()
            .opcode(&[0xB9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFMSUB132PS => GenAPI::new()
            .opcode(&[0x9A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB213PS => GenAPI::new()
            .opcode(&[0xAA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB231PS => GenAPI::new()
            .opcode(&[0xBA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFMSUB132PD => GenAPI::new()
            .opcode(&[0x9A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB213PD => GenAPI::new()
            .opcode(&[0xAA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB231PD => GenAPI::new()
            .opcode(&[0xBA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB132SD => GenAPI::new()
            .opcode(&[0x9B])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB213SD => GenAPI::new()
            .opcode(&[0xAB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB231SD => GenAPI::new()
            .opcode(&[0xBB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB132SS => GenAPI::new()
            .opcode(&[0x9B])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB213SS => GenAPI::new()
            .opcode(&[0xAB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB231SS => GenAPI::new()
            .opcode(&[0xBB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        // fma-part2
        Mnemonic::VFNMADD132PS => GenAPI::new()
            .opcode(&[0x9C])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD213PS => GenAPI::new()
            .opcode(&[0xAC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD231PS => GenAPI::new()
            .opcode(&[0xBC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMADD132PD => GenAPI::new()
            .opcode(&[0x9C])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD213PD => GenAPI::new()
            .opcode(&[0xAC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD231PD => GenAPI::new()
            .opcode(&[0xBC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMADD132SS => GenAPI::new()
            .opcode(&[0x9D])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD213SS => GenAPI::new()
            .opcode(&[0xAD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD231SS => GenAPI::new()
            .opcode(&[0xBD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMADD132SD => GenAPI::new()
            .opcode(&[0x9D])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD213SD => GenAPI::new()
            .opcode(&[0xAD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD231SD => GenAPI::new()
            .opcode(&[0xBD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMSUB132PS => GenAPI::new()
            .opcode(&[0x9E])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB213PS => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB231PS => GenAPI::new()
            .opcode(&[0xBE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMSUB132PD => GenAPI::new()
            .opcode(&[0x9E])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB213PD => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB231PD => GenAPI::new()
            .opcode(&[0xBE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMSUB132SS => GenAPI::new()
            .opcode(&[0x9F])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB213SS => GenAPI::new()
            .opcode(&[0xAF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB231SS => GenAPI::new()
            .opcode(&[0xBF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMSUB132SD => GenAPI::new()
            .opcode(&[0x9F])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB213SD => GenAPI::new()
            .opcode(&[0xAF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB231SD => GenAPI::new()
            .opcode(&[0xBF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        // fma-part3
        Mnemonic::VFMADDSUB132PS => GenAPI::new()
            .opcode(&[0x96])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB213PS => GenAPI::new()
            .opcode(&[0xA6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB231PS => GenAPI::new()
            .opcode(&[0xB6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB132PD => GenAPI::new()
            .opcode(&[0x96])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB213PD => GenAPI::new()
            .opcode(&[0xA6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB231PD => GenAPI::new()
            .opcode(&[0xB6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFMSUBADD132PS => GenAPI::new()
            .opcode(&[0x97])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD213PS => GenAPI::new()
            .opcode(&[0xA7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD231PS => GenAPI::new()
            .opcode(&[0xB7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD132PD => GenAPI::new()
            .opcode(&[0x97])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD213PD => GenAPI::new()
            .opcode(&[0xA7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD231PD => GenAPI::new()
            .opcode(&[0xB7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        // aes
        Mnemonic::AESDEC => GenAPI::new()
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDE])
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::AESENC => GenAPI::new()
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDC])
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::AESIMC => GenAPI::new()
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDB])
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::AESDECLAST => GenAPI::new()
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDF])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::AESENCLAST => GenAPI::new()
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDD])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),

        Mnemonic::VAESDEC => GenAPI::new()
            .opcode(&[0xDE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VAESENC => GenAPI::new()
            .opcode(&[0xDC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VAESIMC => GenAPI::new()
            .opcode(&[0xDB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM]),
        Mnemonic::VAESENCLAST => GenAPI::new()
            .opcode(&[0xDD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VAESDECLAST => GenAPI::new()
            .opcode(&[0xDF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VAESKEYGENASSIST => GenAPI::new()
            .opcode(&[0xDF])
            .imm_atindex(2, 1)
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .modrm(true, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM]),
        Mnemonic::AESKEYGENASSIST => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0xDF])
            .modrm(true, None)
            .imm_atindex(2, 1)
            .opcode_prefix(0x66)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        // cvt-part1
        Mnemonic::CVTPD2PI => GenAPI::new()
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0x2D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTSS2SD => GenAPI::new()
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTPD2PS => GenAPI::new()
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTPS2PD => GenAPI::new()
            .opcode(&[0x0F, 0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTPI2PD => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTPD2DQ => GenAPI::new()
            .opcode(&[0x0F, 0xE6])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTSD2SS => GenAPI::new()
            .opcode(&[0x0F, 0x5A])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTPS2DQ => GenAPI::new()
            .opcode(&[0x0F, 0x5B])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTDQ2PS => GenAPI::new()
            .opcode(&[0x0F, 0x5B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTDQ2PD => GenAPI::new()
            .opcode(&[0x0F, 0xE6])
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTSD2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2D])
            .modrm(true, None)
            .opcode_prefix(0xF2)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTSI2SD => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .modrm(true, None)
            .opcode_prefix(0xF2)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),

        Mnemonic::CVTTPS2DQ => GenAPI::new()
            .opcode(&[0x0F, 0x5B])
            .opcode_prefix(0xF3)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTTSD2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTTPD2PI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTSI2SS => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .opcode_prefix(0xF3)
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CVTPS2PI => GenAPI::new()
            .opcode(&[0x0F, 0x2D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTTPS2PI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTPI2PS => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTTPD2DQ => GenAPI::new()
            .opcode(&[0x0F, 0xE6])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::CVTTSS2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .opcode_prefix(0xF3)
            .rex(),
        Mnemonic::CVTSS2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2D])
            .opcode_prefix(0xF3)
            .rex()
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        // cvt-part2
        Mnemonic::VCVTPD2DQ => GenAPI::new()
            .opcode(&[0xE6])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTPD2PS => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTPS2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTPS2PD => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTSD2SI => GenAPI::new()
            .opcode(&[0x2D])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF2)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTSD2SS => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VCVTSI2SD => GenAPI::new()
            .opcode(&[0x2A])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF2)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VCVTSI2SS => GenAPI::new()
            .opcode(&[0x2A])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF3)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VCVTSS2SD => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VCVTSS2SI => GenAPI::new()
            .opcode(&[0x2D])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF3)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VCVTDQ2PD => GenAPI::new()
            .opcode(&[0xE6])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTDQ2PS => GenAPI::new()
            .opcode(&[0x5B])
            .vex(VexDetails::new().map_select(0x0F).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTTPD2DQ => GenAPI::new()
            .opcode(&[0xE6])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTTPS2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTTSD2SI => GenAPI::new()
            .opcode(&[0x2C])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF2)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTTSS2SI => GenAPI::new()
            .opcode(&[0x2C])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF3)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        // norm-part1a
        Mnemonic::BT => ins_bt(ins, &[0x0F, 0xA3], &[0x0F, 0xBA], bits, 4),
        Mnemonic::BTS => ins_bt(ins, &[0x0F, 0xAB], &[0x0F, 0xBA], bits, 5),
        Mnemonic::BTC => ins_bt(ins, &[0x0F, 0xBB], &[0x0F, 0xBA], bits, 7),
        Mnemonic::BTR => ins_bt(ins, &[0x0F, 0xB3], &[0x0F, 0xBA], bits, 6),
        Mnemonic::CLC => GenAPI::new().opcode(&[0xF8]),
        Mnemonic::CMC => GenAPI::new().opcode(&[0xF5]),
        Mnemonic::CWD => GenAPI::new().opcode(&[0x99]).fixed_size(Size::Word),
        Mnemonic::CDQ => GenAPI::new().opcode(&[0x99]).fixed_size(Size::Dword),
        Mnemonic::CQO => GenAPI::new().opcode(&[0x48, 0x99]),
        Mnemonic::DAA => GenAPI::new().opcode(&[0x27]),
        Mnemonic::DAS => GenAPI::new().opcode(&[0x2F]),
        Mnemonic::CLD => GenAPI::new().opcode(&[0xFC]),
        Mnemonic::CBW => GenAPI::new().opcode(&[0x98]).fixed_size(Size::Word),
        Mnemonic::CLI => GenAPI::new().opcode(&[0xFA]),
        Mnemonic::AAA => GenAPI::new().opcode(&[0x37]),
        Mnemonic::AAS => GenAPI::new().opcode(&[0x3F]),
        Mnemonic::AAD => GenAPI::new().opcode(&[
            0xD5,
            if let Some(Operand::Imm(n)) = ins.dst() {
                n.split_into_bytes()[0]
            } else {
                0x0A
            },
        ]),
        Mnemonic::AAM => GenAPI::new().opcode(&[
            0xD4,
            if let Some(Operand::Imm(n)) = ins.dst() {
                n.split_into_bytes()[0]
            } else {
                0x0A
            },
        ]),
        Mnemonic::ADC => add_like_ins(
            ins,
            &[0x14, 0x15, 0x80, 0x81, 0x83, 0x10, 0x11, 0x12, 0x13],
            2,
            bits,
        ),
        Mnemonic::BSF => GenAPI::new()
            .opcode(&[0x0F, 0xBC])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::BSR => GenAPI::new()
            .opcode(&[0x0F, 0xBD])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        // part b
        Mnemonic::ADCX => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF6])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ADOX => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF6])
            .modrm(true, None)
            .opcode_prefix(0xF3)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::ANDN => GenAPI::new()
            .opcode(&[0xF2])
            .modrm(true, None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0x00)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::CWDE => GenAPI::new().opcode(&[0x98]).fixed_size(Size::Dword),
        Mnemonic::CDQE => GenAPI::new().opcode(&[0x48, 0x98]),
        Mnemonic::CLAC => GenAPI::new().opcode(&[0x0F, 0x01, 0xCA]),
        Mnemonic::CLTS => GenAPI::new().opcode(&[0x0F, 0x06]),
        Mnemonic::CLUI => GenAPI::new()
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x01, 0xEE]),
        Mnemonic::CLWB => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .opcode_prefix(0x66)
            .modrm(true, Some(6)),
        Mnemonic::ARPL => GenAPI::new().opcode(&[0x63]).modrm(true, None),

        Mnemonic::BLSR => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(1))
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[VEX_VVVV, MODRM_RM]),
        Mnemonic::BLSI => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(3))
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[VEX_VVVV, MODRM_RM]),
        Mnemonic::BZHI => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::BEXTR => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::BLSMSK => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(2))
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[VEX_VVVV, MODRM_RM]),
        Mnemonic::BSWAP => GenAPI::new()
            .opcode(&[0x0F, 0xC8 + ins.reg_byte(0).unwrap_or(0)])
            .modrm(false, None)
            .rex(),
        // part c
        Mnemonic::CMPSTRB => GenAPI::new().opcode(&[0xA6]).fixed_size(Size::Byte),
        Mnemonic::CMPSTRW => GenAPI::new().opcode(&[0xA7]).fixed_size(Size::Word),
        Mnemonic::CMPSTRD => GenAPI::new().opcode(&[0xA7]).fixed_size(Size::Dword),
        Mnemonic::CMPSTRQ => GenAPI::new().opcode(&[0x48, 0xA7]).fixed_size(Size::Qword),
        Mnemonic::ENDBR64 => GenAPI::new()
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x1E, 0xFA]),
        Mnemonic::ENDBR32 => GenAPI::new()
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x1E, 0xFB]),
        Mnemonic::CMPXCHG => GenAPI::new()
            .opcode(&[0x0F, (0xB1 - ((ins.size() == Size::Byte) as u8))])
            .modrm(true, None)
            .rex(),
        Mnemonic::CLDEMOTE => GenAPI::new().opcode(&[0x0F, 0x1C]).modrm(true, Some(0)),
        Mnemonic::CMPXCHG8B => GenAPI::new().opcode(&[0x0F, 0xC7]).modrm(true, Some(1)),
        Mnemonic::CMPXCHG16B => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(1))
            .rex(),
        // part 3
        Mnemonic::ENTER => {
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
        Mnemonic::HLT => GenAPI::new().opcode(&[0xF4]),
        Mnemonic::HRESET => GenAPI::new()
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x3A, 0xF0, 0xC0])
            .modrm(false, None)
            .imm_atindex(0, 1),
        Mnemonic::INSB => GenAPI::new().opcode(&[0x6C]).fixed_size(Size::Byte),
        Mnemonic::INSW => GenAPI::new().opcode(&[0x6D]).fixed_size(Size::Word),
        Mnemonic::INSD => GenAPI::new().opcode(&[0x6D]).fixed_size(Size::Dword),
        Mnemonic::INT => GenAPI::new().opcode(&[
            0xCC,
            if let Some(Operand::Imm(imm)) = ins.dst() {
                imm.split_into_bytes()[0]
            } else {
                0x00
            },
        ]),
        Mnemonic::INTO => GenAPI::new().opcode(&[0xCE]),
        Mnemonic::INT3 => GenAPI::new().opcode(&[0xCC]),
        Mnemonic::INT1 => GenAPI::new().opcode(&[0xF1]),
        Mnemonic::INVD => GenAPI::new().opcode(&[0x0F, 0x08]),
        Mnemonic::INVLPG => GenAPI::new().opcode(&[0x0F, 0x01, 0b11_111_000]),
        Mnemonic::INVPCID => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x82])
            .opcode_prefix(0x66)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::IRET | Mnemonic::IRETD => GenAPI::new().opcode(&[0xCF]),
        Mnemonic::IRETQ => GenAPI::new().opcode(&[0x48, 0xCF]),
        Mnemonic::LAHF => GenAPI::new().opcode(&[0x9F]),
        Mnemonic::LAR => GenAPI::new()
            .opcode(&[0x0F, 0x02])
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::LEAVE => GenAPI::new().opcode(&[0xC9]),
        Mnemonic::LLDT => GenAPI::new()
            .opcode(&[0x0F, 0x00])
            .modrm(true, Some(2))
            .can_h66(false),
        Mnemonic::LMSW => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(6))
            .can_h66(false),
        Mnemonic::LODSB => GenAPI::new().fixed_size(Size::Byte).opcode(&[0xAC]),
        Mnemonic::LODSW => GenAPI::new().fixed_size(Size::Word).opcode(&[0xAD]),
        Mnemonic::LODSD => GenAPI::new().fixed_size(Size::Dword).opcode(&[0xAD]),
        Mnemonic::LODSQ => GenAPI::new().fixed_size(Size::Qword).opcode(&[0x48, 0xAD]),

        // part 3
        Mnemonic::LOOP => ins_shrtjmp(ins, vec![0xE2]),
        Mnemonic::LOOPE => ins_shrtjmp(ins, vec![0xE1]),
        Mnemonic::LOOPNE => ins_shrtjmp(ins, vec![0xE0]),
        Mnemonic::LSL => GenAPI::new()
            .opcode(&[0x0F, 0x03])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::LTR => GenAPI::new()
            .opcode(&[0x0F, 0x00])
            .modrm(true, Some(3))
            .can_h66(false),
        Mnemonic::LZCNT => GenAPI::new()
            .opcode(&[0x0F, 0xBD])
            .opcode_prefix(0xF3)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::MOVBE => {
            let mut api = GenAPI::new().modrm(true, None).rex();
            if let Some(Operand::Register(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x38, 0xF0]);
                api = api.ord(&[MODRM_REG, MODRM_RM]);
            } else {
                api = api.opcode(&[0x0F, 0x38, 0xF1]);
            }
            api
        }
        Mnemonic::MOVZX => GenAPI::new()
            .opcode(&[
                0x0F,
                (0xB6 + ((ins.src().unwrap().size() == Size::Word) as u8)),
            ])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::MOVDIRI => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF9])
            .modrm(true, None)
            .rex(),
        Mnemonic::MOVSTRB => GenAPI::new().opcode(&[0xA4]).fixed_size(Size::Byte),
        Mnemonic::MOVSTRW => GenAPI::new().opcode(&[0xA5]).fixed_size(Size::Word),
        Mnemonic::MOVSTRD => GenAPI::new().opcode(&[0xA5]).fixed_size(Size::Dword),
        Mnemonic::MOVSTRQ => GenAPI::new().opcode(&[0x48, 0xA5]).fixed_size(Size::Qword),
        Mnemonic::MULX => GenAPI::new()
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
            )
            .opcode(&[0xF6])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::OUTSB => GenAPI::new().opcode(&[0x6E]).fixed_size(Size::Byte),
        Mnemonic::OUTSW => GenAPI::new().opcode(&[0x6F]).fixed_size(Size::Word),
        Mnemonic::OUTSD => GenAPI::new().opcode(&[0x6F]).fixed_size(Size::Dword),
        Mnemonic::PEXT => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None)
            .vex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::PDEP => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None)
            .vex(
                VexDetails::new()
                    .pp(0xF2)
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::PREFETCHW => GenAPI::new().opcode(&[0x0F, 0x0D]).modrm(true, Some(1)),
        Mnemonic::PREFETCH0 => GenAPI::new().opcode(&[0x0F, 0x18]).modrm(true, Some(1)),
        Mnemonic::PREFETCH1 => GenAPI::new().opcode(&[0x0F, 0x18]).modrm(true, Some(2)),
        Mnemonic::PREFETCH2 => GenAPI::new().opcode(&[0x0F, 0x18]).modrm(true, Some(3)),
        Mnemonic::PREFETCHA => GenAPI::new().opcode(&[0x0F, 0x18]).modrm(true, Some(0)),

        Mnemonic::ROL => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 0, bits),
        Mnemonic::ROR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 1, bits),
        Mnemonic::RCL => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 2, bits),
        Mnemonic::RCR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 3, bits),
        // part 4
        Mnemonic::RDMSR => GenAPI::new().opcode(&[0x0F, 0x32]),
        Mnemonic::RDPID => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .opcode_prefix(0xF3)
            .modrm(true, Some(7)),
        Mnemonic::RDPKRU => GenAPI::new().opcode(&[0x0F, 0x01, 0xEE]),
        Mnemonic::RDPMC => GenAPI::new().opcode(&[0x0F, 0x33]),
        Mnemonic::RDRAND => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(6))
            .rex(),
        Mnemonic::RDSEED => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(7))
            .rex(),
        Mnemonic::RDSSPD | Mnemonic::RDSSPQ => GenAPI::new()
            .opcode(&[0x0F, 0x1E])
            .modrm(true, Some(1))
            .opcode_prefix(0xF3)
            .rex(),
        Mnemonic::RDTSC => GenAPI::new().opcode(&[0x0F, 0x31]),
        Mnemonic::RDTSCP => GenAPI::new().opcode(&[0x0F, 0x01, 0xF9]),
        Mnemonic::RORX => GenAPI::new()
            .opcode(&[0xF0])
            .vex(
                VexDetails::new()
                    .map_select(0x3A)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword)
                    .vlength(Some(false)),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .imm_atindex(2, 1),
        Mnemonic::RSM => GenAPI::new().opcode(&[0x0F, 0xAA]),
        Mnemonic::RSTORSSP => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(5))
            .opcode_prefix(0xF3)
            .rex(),
        Mnemonic::SAHF => GenAPI::new().opcode(&[0x9E]),
        Mnemonic::SHLX => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0x66)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SHRX => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SARX => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0xF3)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SBB => add_like_ins(
            ins,
            &[0x1C, 0x1D, 0x80, 0x81, 0x83, 0x18, 0x19, 0x1A, 0x1B],
            3,
            bits,
        ),
        Mnemonic::SCASB => GenAPI::new().fixed_size(Size::Byte).opcode(&[0xAE]),
        Mnemonic::SCASW => GenAPI::new().fixed_size(Size::Word).opcode(&[0xAF]),
        Mnemonic::SCASD => GenAPI::new().fixed_size(Size::Dword).opcode(&[0xAF]),
        Mnemonic::SCASQ => GenAPI::new().fixed_size(Size::Qword).opcode(&[0x48, 0xAF]),
        Mnemonic::SENDUIPI => GenAPI::new()
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(6))
            .can_h66(false)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SERIALIZE => GenAPI::new().opcode(&[0x0F, 0x01, 0xE8]),
        // for some reason NASM generates this as no opcode at all?
        Mnemonic::SETSSBY => GenAPI::new(),

        // setcc
        Mnemonic::SETO => GenAPI::new().opcode(&[0x0F, 0x90]).modrm(true, None).rex(),
        Mnemonic::SETNO => GenAPI::new().opcode(&[0x0F, 0x91]).modrm(true, None).rex(),
        Mnemonic::SETB | Mnemonic::SETC | Mnemonic::SETNAE => {
            GenAPI::new().opcode(&[0x0F, 0x92]).modrm(true, None).rex()
        }

        Mnemonic::SETAE | Mnemonic::SETNB | Mnemonic::SETNC => {
            GenAPI::new().opcode(&[0x0F, 0x93]).modrm(true, None).rex()
        }

        Mnemonic::SETE | Mnemonic::SETZ => {
            GenAPI::new().opcode(&[0x0F, 0x94]).modrm(true, None).rex()
        }
        Mnemonic::SETNE | Mnemonic::SETNZ => {
            GenAPI::new().opcode(&[0x0F, 0x95]).modrm(true, None).rex()
        }

        Mnemonic::SETBE | Mnemonic::SETNA => {
            GenAPI::new().opcode(&[0x0F, 0x96]).modrm(true, None).rex()
        }

        Mnemonic::SETA | Mnemonic::SETNBE => {
            GenAPI::new().opcode(&[0x0F, 0x97]).modrm(true, None).rex()
        }

        Mnemonic::SETS => GenAPI::new().opcode(&[0x0F, 0x98]).modrm(true, None).rex(),
        Mnemonic::SETNS => GenAPI::new().opcode(&[0x0F, 0x99]).modrm(true, None).rex(),

        Mnemonic::SETP | Mnemonic::SETPE => {
            GenAPI::new().opcode(&[0x0F, 0x9A]).modrm(true, None).rex()
        }

        Mnemonic::SETNP | Mnemonic::SETPO => {
            GenAPI::new().opcode(&[0x0F, 0x9B]).modrm(true, None).rex()
        }

        Mnemonic::SETL | Mnemonic::SETNGE => {
            GenAPI::new().opcode(&[0x0F, 0x9C]).modrm(true, None).rex()
        }

        Mnemonic::SETGE | Mnemonic::SETNL => {
            GenAPI::new().opcode(&[0x0F, 0x9D]).modrm(true, None).rex()
        }

        Mnemonic::SETLE | Mnemonic::SETNG => {
            GenAPI::new().opcode(&[0x0F, 0x9E]).modrm(true, None).rex()
        }

        Mnemonic::SETG | Mnemonic::SETNLE => {
            GenAPI::new().opcode(&[0x0F, 0x9F]).modrm(true, None).rex()
        }

        // norm-part5
        Mnemonic::SFENCE => GenAPI::new().opcode(&[0x0F, 0xAE, 0xF8]),
        Mnemonic::STAC => GenAPI::new().opcode(&[0x0F, 0x01, 0xCB]),
        Mnemonic::STC => GenAPI::new().opcode(&[0xF9]),
        Mnemonic::STD => GenAPI::new().opcode(&[0xFD]),
        Mnemonic::STI => GenAPI::new().opcode(&[0xFB]),
        Mnemonic::STUI => GenAPI::new()
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x01, 0xEF]),
        Mnemonic::STOSB => GenAPI::new().opcode(&[0xAA]),
        Mnemonic::STOSW => GenAPI::new().opcode(&[0xAB]).fixed_size(Size::Word),
        Mnemonic::STOSD => GenAPI::new().opcode(&[0xAB]).fixed_size(Size::Dword),
        Mnemonic::STOSQ => GenAPI::new().opcode(&[0x48, 0xAB]),
        Mnemonic::SYSENTER => GenAPI::new().opcode(&[0x0F, 0x34]),
        Mnemonic::SYSEXIT => GenAPI::new().opcode(&[0x0F, 0x35]),
        Mnemonic::SYSRET => GenAPI::new().opcode(&[0x0F, 0x07]),
        Mnemonic::TESTUI => GenAPI::new()
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0x01, 0xED]),
        Mnemonic::UD2 => GenAPI::new().opcode(&[0x0F, 0x0B]),
        Mnemonic::UD0 => GenAPI::new()
            .opcode(&[0x0F, 0xFF])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::UD1 => GenAPI::new()
            .opcode(&[0x0F, 0xB9])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::TPAUSE => GenAPI::new()
            .modrm(true, Some(6))
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0xAE]),
        Mnemonic::UMWAIT => GenAPI::new()
            .modrm(true, Some(6))
            .opcode_prefix(0xF2)
            .opcode(&[0x0F, 0xAE]),
        Mnemonic::UMONITOR => GenAPI::new()
            .modrm(true, Some(6))
            .opcode_prefix(0xF3)
            .opcode(&[0x0F, 0xAE]),
        Mnemonic::SMSW => GenAPI::new()
            .modrm(true, Some(4))
            .opcode(&[0x0F, 0x01])
            .rex(),
        Mnemonic::STR => GenAPI::new().modrm(true, Some(1)).opcode(&[0x0F, 0x00]),
        Mnemonic::VERR => GenAPI::new()
            .modrm(true, Some(4))
            .opcode(&[0x0F, 0x00])
            .can_h66(false),
        Mnemonic::VERW => GenAPI::new()
            .modrm(true, Some(5))
            .opcode(&[0x0F, 0x00])
            .can_h66(false),
        Mnemonic::SHLD => ins_shlx(ins, &[0x0F, 0xA4], &[0x0F, 0xA5]),
        Mnemonic::SHRD => ins_shlx(ins, &[0x0F, 0xAC], &[0x0F, 0xAD]),
        Mnemonic::UIRET => GenAPI::new().opcode(&[0xF3, 0x0F, 0x01, 0xEC]),
        Mnemonic::WAIT | Mnemonic::FWAIT => GenAPI::new().opcode(&[0x9B]),
        Mnemonic::WBINVD => GenAPI::new().opcode(&[0x0F, 0x09]),
        Mnemonic::WRMSR => GenAPI::new().opcode(&[0x0F, 0x30]),
        Mnemonic::WRPKRU => GenAPI::new().opcode(&[0x0F, 0x01, 0xEF]),

        // norm-part6
        Mnemonic::XABORT => GenAPI::new().imm_atindex(0, 1).opcode(&[0xC6, 0xF8]),
        Mnemonic::XACQUIRE => GenAPI::new().opcode(&[0xF2]),
        Mnemonic::XRELEASE => GenAPI::new().opcode(&[0xF3]),
        Mnemonic::XADD => GenAPI::new()
            .opcode(&[0x0F, (0xC0 + ((ins.size() != Size::Byte) as u8))])
            .modrm(true, None)
            .rex(),
        Mnemonic::XBEGIN => ins_xbegin(ins),
        Mnemonic::XCHG => ins_xchg(ins),
        Mnemonic::XEND => GenAPI::new().opcode(&[0x0F, 0x01, 0xD5]),
        Mnemonic::XGETBV => GenAPI::new().opcode(&[0x0F, 0x01, 0xD0]),
        Mnemonic::XLAT | Mnemonic::XLATB => GenAPI::new().opcode(&[0xD7]),
        Mnemonic::XLATB64 => GenAPI::new().opcode(&[0x48, 0xD7]),
        Mnemonic::XRESLDTRK => GenAPI::new()
            .opcode_prefix(0xF2)
            .opcode(&[0x0F, 0x01, 0xE9]),

        Mnemonic::XRSTOR | Mnemonic::XRSTOR64 => {
            GenAPI::new().opcode(&[0x0F, 0xAE]).modrm(true, Some(5))
        }
        Mnemonic::XRSTORS | Mnemonic::XRSTORS64 => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(3))
            .rex(),
        Mnemonic::XSAVE | Mnemonic::XSAVE64 => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(4))
            .rex(),
        Mnemonic::XSAVEC | Mnemonic::XSAVEC64 => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(4))
            .rex(),
        Mnemonic::XSAVEOPT | Mnemonic::XSAVEOPT64 => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(6))
            .rex(),
        Mnemonic::XSAVES | Mnemonic::XSAVES64 => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(5))
            .rex(),
        Mnemonic::XSETBV => GenAPI::new().opcode(&[0x0F, 0x01, 0xD1]),
        Mnemonic::XSUSLDTRK => GenAPI::new()
            .opcode_prefix(0xF2)
            .opcode(&[0x0F, 0x01, 0xE8]),
        Mnemonic::XTEST => GenAPI::new().opcode(&[0x0F, 0x01, 0xD6]),
        // sha.asm
        Mnemonic::SHA1MSG1 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xC9])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(),
        Mnemonic::SHA1NEXTE => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xC8])
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SHA1MSG2 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCA])
            .modrm(true, None)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SHA1RNDS4 => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0xCC])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex()
            .imm_atindex(2, 1),
        Mnemonic::SHA256RNDS2 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCB])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(),
        Mnemonic::SHA256MSG2 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCD])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(),
        Mnemonic::SHA256MSG1 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCC])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(),

        // fxd
        Mnemonic::WRGSBASE => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(3))
            .opcode_prefix(0xF3)
            .rex(),
        Mnemonic::WRFSBASE => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(2))
            .opcode_prefix(0xF3)
            .rex(),
        Mnemonic::LIDT => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(3))
            .rex()
            .can_h66(false),
        Mnemonic::LGDT => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(2))
            .rex()
            .can_h66(false),
        Mnemonic::LOCK => GenAPI::new().opcode(&[0xF0]),
        Mnemonic::REPNE | Mnemonic::REPNZ => GenAPI::new().opcode(&[0xF2]),
        Mnemonic::REP | Mnemonic::REPE | Mnemonic::REPZ => GenAPI::new().opcode(&[0xF3]),

        Mnemonic::VADDPH => GenAPI::new()
            .opcode(&[0x58])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSH => GenAPI::new()
            .opcode(&[0x58])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false).pp(0xF3))
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VALIGNQ => GenAPI::new()
            .opcode(&[0x03])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None)
            .imm_atindex(3, 1),
        Mnemonic::VALIGND => GenAPI::new()
            .opcode(&[0x03])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None)
            .imm_atindex(3, 1),
        Mnemonic::VBCSTNESH2PS => GenAPI::new()
            .opcode(&[0xB1])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VBCSTNEBF162PS => GenAPI::new()
            .opcode(&[0xB1])
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VBLENDMPS => GenAPI::new()
            .opcode(&[0x65])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VBLENDMPD => GenAPI::new()
            .opcode(&[0x65])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VBROADCASTF32X2 => GenAPI::new()
            .opcode(&[0x19])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VBROADCASTF32X4 => GenAPI::new()
            .opcode(&[0x1A])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VBROADCASTF64X2 => GenAPI::new()
            .opcode(&[0x1A])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VBROADCASTF32X8 => GenAPI::new()
            .opcode(&[0x1B])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VBROADCASTF64X4 => GenAPI::new()
            .opcode(&[0x1B])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VCMPPH => GenAPI::new()
            .opcode(&[0xC2])
            .evex(VexDetails::new().map_select(MAP3A))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None)
            .imm_atindex(3, 1),
        Mnemonic::VCMPSH => GenAPI::new()
            .opcode(&[0xC2])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None)
            .imm_atindex(3, 1),
        Mnemonic::VCOMISH => GenAPI::new()
            .opcode(&[0x2F])
            .evex(VexDetails::new().map_select(MAP5))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VCOMPRESSPD | Mnemonic::VCOMPRESSPS => GenAPI::new()
            .opcode(&[0x8A])
            .evex(
                VexDetails::new()
                    .map_select(MAP38)
                    .pp(0x66)
                    .vex_we(ins.mnemonic == Mnemonic::VCOMPRESSPD),
            )
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG]),
        Mnemonic::KADDB => GenAPI::new()
            .opcode(&[0x4A])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KADDW => GenAPI::new()
            .opcode(&[0x4A])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KADDD => GenAPI::new()
            .opcode(&[0x4A])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KADDQ => GenAPI::new()
            .opcode(&[0x4A])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KANDB => GenAPI::new()
            .opcode(&[0x41])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KANDW => GenAPI::new()
            .opcode(&[0x41])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KANDD => GenAPI::new()
            .opcode(&[0x41])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KANDQ => GenAPI::new()
            .opcode(&[0x41])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KANDNB => GenAPI::new()
            .opcode(&[0x42])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KANDNW => GenAPI::new()
            .opcode(&[0x42])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KANDND => GenAPI::new()
            .opcode(&[0x42])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KANDNQ => GenAPI::new()
            .opcode(&[0x42])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KNOTB => GenAPI::new()
            .opcode(&[0x44])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KNOTW => GenAPI::new()
            .opcode(&[0x44])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KNOTD => GenAPI::new()
            .opcode(&[0x44])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KNOTQ => GenAPI::new()
            .opcode(&[0x44])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KORB => GenAPI::new()
            .opcode(&[0x45])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KORW => GenAPI::new()
            .opcode(&[0x45])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KORD => GenAPI::new()
            .opcode(&[0x45])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KORQ => GenAPI::new()
            .opcode(&[0x45])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KXORB => GenAPI::new()
            .opcode(&[0x47])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KXORW => GenAPI::new()
            .opcode(&[0x47])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KXORD => GenAPI::new()
            .opcode(&[0x47])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KXORQ => GenAPI::new()
            .opcode(&[0x47])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KTESTB => GenAPI::new()
            .opcode(&[0x99])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KTESTW => GenAPI::new()
            .opcode(&[0x99])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KTESTD => GenAPI::new()
            .opcode(&[0x99])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KTESTQ => GenAPI::new()
            .opcode(&[0x99])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .strict_pfx()
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KXNORB => GenAPI::new()
            .opcode(&[0x46])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KXNORW => GenAPI::new()
            .opcode(&[0x46])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KXNORD => GenAPI::new()
            .opcode(&[0x46])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KXNORQ => GenAPI::new()
            .opcode(&[0x46])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(true))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KUNPCKBW => GenAPI::new()
            .opcode(&[0x4B])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(true)),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KUNPCKWD => GenAPI::new()
            .opcode(&[0x4B])
            .vex(VexDetails::new().map_select(0x0F).vlength(Some(true)))
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KUNPCKDQ => GenAPI::new()
            .opcode(&[0x4B])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vex_we(true)
                    .vlength(Some(true)),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KORTESTB => GenAPI::new()
            .opcode(&[0x98])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0x66)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KORTESTW => GenAPI::new()
            .opcode(&[0x98])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KORTESTD => GenAPI::new()
            .opcode(&[0x98])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x0F)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KORTESTQ => GenAPI::new()
            .opcode(&[0x98])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KSHIFTLB => GenAPI::new()
            .opcode(&[0x32])
            .vex(
                VexDetails::new()
                    .map_select(0x3A)
                    .pp(0x66)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KSHIFTLW => GenAPI::new()
            .opcode(&[0x32])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x3A)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KSHIFTLD => GenAPI::new()
            .opcode(&[0x33])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x3A)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KSHIFTLQ => GenAPI::new()
            .opcode(&[0x33])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x3A)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KSHIFTRB => GenAPI::new()
            .opcode(&[0x30])
            .vex(
                VexDetails::new()
                    .map_select(0x3A)
                    .pp(0x66)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KSHIFTRW => GenAPI::new()
            .opcode(&[0x30])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x3A)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KSHIFTRD => GenAPI::new()
            .opcode(&[0x31])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x3A)
                    .vlength(Some(false))
                    .vex_we(false),
            )
            .modrm(true, None)
            .strict_pfx()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KSHIFTRQ => GenAPI::new()
            .opcode(&[0x31])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x3A)
                    .vlength(Some(false))
                    .vex_we(true),
            )
            .modrm(true, None)
            .strict_pfx()
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::KMOVB => ins_kmov(ins).vex(
            VexDetails::new()
                .map_select(0x0F)
                .pp(0x66)
                .vex_we(false)
                .vlength(Some(false)),
        ),
        Mnemonic::KMOVW => ins_kmov(ins).vex(
            VexDetails::new()
                .map_select(0x0F)
                .vex_we(false)
                .vlength(Some(false)),
        ),
        Mnemonic::KMOVD => {
            let api = ins_kmov(ins);
            let mut vd = VexDetails::new()
                .map_select(0x0F)
                .pp(0x66)
                .vex_we(true)
                .vlength(Some(false));
            let opc = api.get_opcode();
            let opc = opc[0];

            if opc == 0x92 || opc == 0x93 {
                vd = vd.pp(0xF2).vex_we(false);
            }
            api.vex(vd)
        }
        Mnemonic::KMOVQ => {
            let api = ins_kmov(ins);
            let mut vd = VexDetails::new()
                .map_select(0x0F)
                .vex_we(true)
                .vlength(Some(false));
            let opc = api.get_opcode();
            let opc = opc[0];

            if opc == 0x92 || opc == 0x93 {
                vd = vd.pp(0xF2);
            }
            api.vex(vd)
        }
        Mnemonic::VCVTDQ2PH => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTNE2PS2BF16 => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VCVTNEEBF162PS => GenAPI::new()
            .opcode(&[0xB0])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(0x38).vex_we(false)),
        Mnemonic::VCVTNEEPH2PS => GenAPI::new()
            .opcode(&[0xB0])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VCVTNEOBF162PS => GenAPI::new()
            .opcode(&[0xB0])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),
        Mnemonic::VCVTNEOPH2PS => GenAPI::new()
            .opcode(&[0xB0])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(0x38).vex_we(false)),
        Mnemonic::VCVTNEPS2BF16 => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0xF3).map_select(0x38).vex_we(false)),
        Mnemonic::VCVTPD2PH => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(true)),
        Mnemonic::VCVTPD2QQ => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTPD2UDQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTPD2UQQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTPH2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2PD => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2PS => GenAPI::new()
            .opcode(&[0x13])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VCVTPH2PSX => GenAPI::new()
            .opcode(&[0x13])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VCVTPH2QQ => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2UDQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2UQQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2UW => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2W => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPS2PH => GenAPI::new()
            .opcode(&[0x1D])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VCVTPS2PHX => GenAPI::new()
            .opcode(&[0x1D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPS2QQ => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTPS2UDQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTPS2UQQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTQQ2PD => GenAPI::new()
            .opcode(&[0xE6])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTQQ2PH => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(true)),
        Mnemonic::VCVTQQ2PS => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTSD2SH => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP5).vex_we(true)),
        Mnemonic::VCVTSD2USI => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF2)
                    .map_select(MAP0F)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTSH2SD => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTSH2SI => GenAPI::new()
            .opcode(&[0x2D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTSH2SS => GenAPI::new()
            .opcode(&[0x13])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP6).vex_we(false)),
        Mnemonic::VCVTSH2USI => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTSI2SH => GenAPI::new()
            .opcode(&[0x2A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTSS2SH => GenAPI::new()
            .opcode(&[0x1D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTSS2USI => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP0F)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTTPD2QQ => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTTPD2UDQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTTPD2UQQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTTPH2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2QQ => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2UDQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2UQQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2UW => GenAPI::new()
            .opcode(&[0x7C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2W => GenAPI::new()
            .opcode(&[0x7C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPS2QQ => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTTPS2UDQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTTPS2UQQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTTSD2USI => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF2)
                    .map_select(MAP0F)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTTSH2SI => GenAPI::new()
            .opcode(&[0x2C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTTSH2USI => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTTSS2USI => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP0F)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTUDQ2PD => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTUDQ2PH => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTUDQ2PS => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTUQQ2PD => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTUQQ2PH => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP5).vex_we(true)),
        Mnemonic::VCVTUQQ2PS => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTUSI2SD => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF2)
                    .map_select(MAP0F)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTUSI2SH => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTUSI2SS => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP0F)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTUW2PH => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTW2PH => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),

        Mnemonic::VDBPSADBW => GenAPI::new()
            .opcode(&[0x42])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VDIVPH => GenAPI::new()
            .opcode(&[0x5E])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VDIVSH => GenAPI::new()
            .opcode(&[0x5E])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VDPBF16PS => GenAPI::new()
            .opcode(&[0x52])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VEXPANDPD => GenAPI::new()
            .opcode(&[0x88])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VEXPANDPS => GenAPI::new()
            .opcode(&[0x88])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VEXTRACTF32X4 => GenAPI::new()
            .opcode(&[0x19])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTF64X2 => GenAPI::new()
            .opcode(&[0x19])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VEXTRACTF32X8 => GenAPI::new()
            .opcode(&[0x1B])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTF64X4 => GenAPI::new()
            .opcode(&[0x1B])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VEXTRACTI128 => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTI32X4 => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTI32X8 => GenAPI::new()
            .opcode(&[0x3B])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTI64X4 => GenAPI::new()
            .opcode(&[0x3B])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VEXTRACTI64X2 => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFCMADDCPH => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDCPH => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP6).vex_we(false)),
        Mnemonic::VFCMADDCSH => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDCSH => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP6).vex_we(false)),
        Mnemonic::VFCMULCPH => GenAPI::new()
            .opcode(&[0xD6])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMULCPH => GenAPI::new()
            .opcode(&[0xD6])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP6).vex_we(false)),
        Mnemonic::VFCMULCSH => GenAPI::new()
            .opcode(&[0xD7])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMULCSH => GenAPI::new()
            .opcode(&[0xD7])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP6).vex_we(false)),
        Mnemonic::VFIXUPIMMPD => GenAPI::new()
            .opcode(&[0x54])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFIXUPIMMPS => GenAPI::new()
            .opcode(&[0x54])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VFIXUPIMMSD => GenAPI::new()
            .opcode(&[0x55])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFIXUPIMMSS => GenAPI::new()
            .opcode(&[0x55])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VFMADD132PH => GenAPI::new()
            .opcode(&[0x98])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD213PH => GenAPI::new()
            .opcode(&[0xA8])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD231PH => GenAPI::new()
            .opcode(&[0xB8])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD132PH => GenAPI::new()
            .opcode(&[0x9C])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD213PH => GenAPI::new()
            .opcode(&[0xAC])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD231PH => GenAPI::new()
            .opcode(&[0xBC])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD132SH => GenAPI::new()
            .opcode(&[0x99])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD213SH => GenAPI::new()
            .opcode(&[0xA9])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD231SH => GenAPI::new()
            .opcode(&[0xB9])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD132SH => GenAPI::new()
            .opcode(&[0x9D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD213SH => GenAPI::new()
            .opcode(&[0xAD])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD231SH => GenAPI::new()
            .opcode(&[0xBD])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDSUB132PH => GenAPI::new()
            .opcode(&[0x96])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDSUB213PH => GenAPI::new()
            .opcode(&[0xA6])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDSUB231PH => GenAPI::new()
            .opcode(&[0xB6])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VFMSUB132PH => GenAPI::new()
            .opcode(&[0x9A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUB213PH => GenAPI::new()
            .opcode(&[0xAA])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUB231PH => GenAPI::new()
            .opcode(&[0xBA])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB132PH => GenAPI::new()
            .opcode(&[0x9E])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB213PH => GenAPI::new()
            .opcode(&[0xAE])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB231PH => GenAPI::new()
            .opcode(&[0xBE])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VFMSUB132SH => GenAPI::new()
            .opcode(&[0x9B])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUB213SH => GenAPI::new()
            .opcode(&[0xAB])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUB231SH => GenAPI::new()
            .opcode(&[0xBB])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB132SH => GenAPI::new()
            .opcode(&[0x9F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB213SH => GenAPI::new()
            .opcode(&[0xAF])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB231SH => GenAPI::new()
            .opcode(&[0xBF])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUBADD132PH => GenAPI::new()
            .opcode(&[0x97])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUBADD213PH => GenAPI::new()
            .opcode(&[0xA7])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUBADD231PH => GenAPI::new()
            .opcode(&[0xB7])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VFPCLASSPD => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFPCLASSPH => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VFPCLASSPS => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VFPCLASSSD => GenAPI::new()
            .opcode(&[0x67])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFPCLASSSH => GenAPI::new()
            .opcode(&[0x67])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VFPCLASSSS => GenAPI::new()
            .opcode(&[0x67])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VGETEXPPD => GenAPI::new()
            .opcode(&[0x42])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGETEXPPH => GenAPI::new()
            .opcode(&[0x42])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP6).vex_we(false)),
        Mnemonic::VGETEXPPS => GenAPI::new()
            .opcode(&[0x42])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGETEXPSH => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VGETEXPSS => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGETMANTPD => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VGETMANTPH => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VGETMANTPS => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VGETMANTSD => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VGETMANTSH => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VGETMANTSS => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTF32X4 => GenAPI::new()
            .opcode(&[0x18])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTF64X2 => GenAPI::new()
            .opcode(&[0x18])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VINSERTF32X8 => GenAPI::new()
            .opcode(&[0x1A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTF64X4 => GenAPI::new()
            .opcode(&[0x1A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VINSERTI32X4 => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTI64X2 => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VINSERTI32X8 => GenAPI::new()
            .opcode(&[0x3A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTI64X4 => GenAPI::new()
            .opcode(&[0x3A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VINSERTI128 => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VMASKMOVPS => {
            let mut api = GenAPI::new()
                .strict_pfx()
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None);

            api = if let Some(Operand::Mem(_)) = ins.dst() {
                api.opcode(&[0x2C]).ord(&[MODRM_RM, VEX_VVVV, MODRM_REG])
            } else {
                api.opcode(&[0x2E]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            };

            api
        }
        Mnemonic::VMASKMOVPD => {
            let mut api = GenAPI::new()
                .strict_pfx()
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None);

            api = if let Some(Operand::Mem(_)) = ins.dst() {
                api.opcode(&[0x2D]).ord(&[MODRM_RM, VEX_VVVV, MODRM_REG])
            } else {
                api.opcode(&[0x2F]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            };

            api
        }
        Mnemonic::VMAXPH => GenAPI::new()
            .opcode(&[0x5F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VMAXSH => GenAPI::new()
            .opcode(&[0x5F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VMINPH => GenAPI::new()
            .opcode(&[0x5D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VMINSH => GenAPI::new()
            .opcode(&[0x5D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        // Intel says that for: vmovsh xmm1 {k1}{z}, xmm2, xmm3 you can encode
        // with both: 0x10 and 0x11?
        Mnemonic::VMOVSH => {
            let mut api = GenAPI::new()
                .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false))
                .modrm(true, None);

            api = if let Some(Operand::Mem(_)) = ins.dst() {
                api.opcode(&[0x11]).ord(&[MODRM_RM, MODRM_REG])
            } else if let Some(Operand::Mem(_)) = ins.src() {
                api.opcode(&[0x10]).ord(&[MODRM_REG, MODRM_RM])
            } else {
                api.opcode(&[0x10]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            };

            api
        }
        Mnemonic::VMOVW => {
            let mut api = GenAPI::new()
                .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false))
                .modrm(true, None);
            api = if let Some(Operand::Mem(_)) = ins.dst() {
                api.opcode(&[0x7E]).ord(&[MODRM_RM, MODRM_REG])
            } else {
                api.opcode(&[0x6E]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            };

            api
        }
        Mnemonic::VMULPH => GenAPI::new()
            .opcode(&[0x59])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VMULSH => GenAPI::new()
            .opcode(&[0x59])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VP2INTERSECTD => GenAPI::new()
            .opcode(&[0x68])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VP2INTERSECTQ => GenAPI::new()
            .opcode(&[0x68])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBLENDD => GenAPI::new()
            .opcode(&[0x68])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBLENDMB => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBLENDMW => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBLENDMD => GenAPI::new()
            .opcode(&[0x64])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBLENDMQ => GenAPI::new()
            .opcode(&[0x64])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBROADCASTB => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTW => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTD => GenAPI::new()
            .opcode(&[0x58])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTQ => GenAPI::new()
            .opcode(&[0x59])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(MAP38)
                    .vex_we(ins.needs_evex()),
            ),
        Mnemonic::VPBROADCASTI32X2 => GenAPI::new()
            .opcode(&[0x59])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTI128 => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTI32X4 => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTI64X2 => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBROADCASTI32X8 => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTI64X4 => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBROADCASTMB2Q => GenAPI::new()
            .opcode(&[0x2A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBROADCASTMW2D => GenAPI::new()
            .opcode(&[0x3A])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPCMPB => GenAPI::new()
            .opcode(&[0x3F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPCMPUB => GenAPI::new()
            .opcode(&[0x3E])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPCMPD => GenAPI::new()
            .opcode(&[0x1F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPCMPUD => GenAPI::new()
            .opcode(&[0x1E])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPCMPQ => GenAPI::new()
            .opcode(&[0x1F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPCMPUQ => GenAPI::new()
            .opcode(&[0x1E])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPCMPW => GenAPI::new()
            .opcode(&[0x3F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPCMPUW => GenAPI::new()
            .opcode(&[0x3E])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPCOMPRESSB => GenAPI::new()
            .opcode(&[0x63])
            .modrm(true, None)
            .ord(if ins.dst().unwrap().is_mem() {
                &[MODRM_RM, MODRM_REG]
            } else {
                &[MODRM_REG, MODRM_RM]
            })
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPCOMPRESSW => GenAPI::new()
            .opcode(&[0x63])
            .modrm(true, None)
            .ord(if ins.dst().unwrap().is_mem() {
                &[MODRM_RM, MODRM_REG]
            } else {
                &[MODRM_REG, MODRM_RM]
            })
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPCOMPRESSD => GenAPI::new()
            .opcode(&[0x8B])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPCOMPRESSQ => GenAPI::new()
            .opcode(&[0x8B])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPCONFLICTD => GenAPI::new()
            .opcode(&[0xC4])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPCONFLICTQ => GenAPI::new()
            .opcode(&[0xC4])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPDPBSSD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBSSDS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBSUD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBSUDS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBUUD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBUUDS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBUSD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBUSDS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWSSD => GenAPI::new()
            .opcode(&[0x52])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWSSDS => GenAPI::new()
            .opcode(&[0x53])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),

        Mnemonic::VPDPWUSD => GenAPI::new()
            .opcode(&[0xD2])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWUSDS => GenAPI::new()
            .opcode(&[0xD3])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWSUD => GenAPI::new()
            .opcode(&[0xD2])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWSUDS => GenAPI::new()
            .opcode(&[0xD3])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWUUD => GenAPI::new()
            .opcode(&[0xD2])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWUUDS => GenAPI::new()
            .opcode(&[0xD3])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMB => GenAPI::new()
            .opcode(&[0x8D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMD => GenAPI::new()
            .opcode(&[0x36])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMW => GenAPI::new()
            .opcode(&[0x8D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMI2B => GenAPI::new()
            .opcode(&[0x75])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMI2W => GenAPI::new()
            .opcode(&[0x75])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMI2D => GenAPI::new()
            .opcode(&[0x76])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMI2Q => GenAPI::new()
            .opcode(&[0x76])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMI2PS => GenAPI::new()
            .opcode(&[0x77])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMI2PD => GenAPI::new()
            .opcode(&[0x77])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMPD => {
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                GenAPI::new()
                    .opcode(&[0x01])
                    .modrm(true, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .imm_atindex(2, 1)
                    .vex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true))
            } else {
                GenAPI::new()
                    .opcode(&[0x16])
                    .modrm(true, None)
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            }
        }
        Mnemonic::VPERMPS => GenAPI::new()
            .opcode(&[0x16])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMQ => {
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                GenAPI::new()
                    .opcode(&[0x00])
                    .modrm(true, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .imm_atindex(2, 1)
                    .vex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true))
            } else {
                GenAPI::new()
                    .opcode(&[0x36])
                    .modrm(true, None)
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            }
        }
        Mnemonic::VPERMT2B => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMT2W => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMT2D => GenAPI::new()
            .opcode(&[0x7E])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMT2Q => GenAPI::new()
            .opcode(&[0x7E])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMT2PS => GenAPI::new()
            .opcode(&[0x7F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMT2PD => GenAPI::new()
            .opcode(&[0x7F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPEXPANDB => GenAPI::new()
            .opcode(&[0x62])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPEXPANDW => GenAPI::new()
            .opcode(&[0x62])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPEXPANDD => GenAPI::new()
            .opcode(&[0x89])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPEXPANDQ => GenAPI::new()
            .opcode(&[0x89])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPLZCNTD => GenAPI::new()
            .opcode(&[0x44])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPLZCNTQ => GenAPI::new()
            .opcode(&[0x44])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMADD52LUQ => GenAPI::new()
            .opcode(&[0xB5])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMADD52HUQ => GenAPI::new()
            .opcode(&[0xB5])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMASKMOVD => {
            if let Some(Operand::Mem(_) | Operand::Symbol(_)) = ins.dst() {
                GenAPI::new()
                    .opcode(&[0x8E])
                    .modrm(true, None)
                    .ord(&[MODRM_RM, VEX_VVVV, MODRM_REG])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38))
                    .strict_pfx()
            } else {
                GenAPI::new()
                    .opcode(&[0x8C])
                    .modrm(true, None)
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38))
                    .strict_pfx()
            }
        }
        Mnemonic::VPMASKMOVQ => {
            if let Some(Operand::Mem(_) | Operand::Symbol(_)) = ins.dst() {
                GenAPI::new()
                    .opcode(&[0x8E])
                    .modrm(true, None)
                    .ord(&[MODRM_RM, VEX_VVVV, MODRM_REG])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                    .strict_pfx()
            } else {
                GenAPI::new()
                    .opcode(&[0x8C])
                    .modrm(true, None)
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                    .strict_pfx()
            }
        }
        Mnemonic::VPMOVB2M => GenAPI::new()
            .opcode(&[0x29])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVW2M => GenAPI::new()
            .opcode(&[0x29])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMOVD2M => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVQ2M => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMOVDB => GenAPI::new()
            .opcode(&[0x31])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSDB => GenAPI::new()
            .opcode(&[0x21])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSDB => GenAPI::new()
            .opcode(&[0x11])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVDW => GenAPI::new()
            .opcode(&[0x33])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSDW => GenAPI::new()
            .opcode(&[0x23])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSDW => GenAPI::new()
            .opcode(&[0x13])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMOVM2B => GenAPI::new()
            .opcode(&[0x28])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVM2W => GenAPI::new()
            .opcode(&[0x28])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMOVM2D => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVM2Q => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),

        Mnemonic::VPMOVQB => GenAPI::new()
            .opcode(&[0x32])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSQB => GenAPI::new()
            .opcode(&[0x22])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSQB => GenAPI::new()
            .opcode(&[0x12])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMOVQD => GenAPI::new()
            .opcode(&[0x35])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSQD => GenAPI::new()
            .opcode(&[0x25])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSQD => GenAPI::new()
            .opcode(&[0x15])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMOVQW => GenAPI::new()
            .opcode(&[0x34])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSQW => GenAPI::new()
            .opcode(&[0x24])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSQW => GenAPI::new()
            .opcode(&[0x14])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMOVWB => GenAPI::new()
            .opcode(&[0x30])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSWB => GenAPI::new()
            .opcode(&[0x20])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSWB => GenAPI::new()
            .opcode(&[0x10])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMULTISHIFTQB => GenAPI::new()
            .opcode(&[0x83])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPOPCNTB => GenAPI::new()
            .opcode(&[0x54])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPOPCNTW => GenAPI::new()
            .opcode(&[0x54])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPOPCNTD => GenAPI::new()
            .opcode(&[0x55])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPOPCNTQ => GenAPI::new()
            .opcode(&[0x55])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),

        Mnemonic::VPROLD => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, Some(1))
            .ord(&[VEX_VVVV, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VPROLQ => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, Some(1))
            .ord(&[VEX_VVVV, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VPROLVD => GenAPI::new()
            .opcode(&[0x15])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VPROLVQ => GenAPI::new()
            .opcode(&[0x15])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),

        Mnemonic::VPRORD => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, Some(0))
            .ord(&[VEX_VVVV, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VPRORQ => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, Some(0))
            .ord(&[VEX_VVVV, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VPRORVD => GenAPI::new()
            .opcode(&[0x14])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VPRORVQ => GenAPI::new()
            .opcode(&[0x14])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),

        Mnemonic::VPSHLDW => GenAPI::new()
            .opcode(&[0x70])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPSHLDD => GenAPI::new()
            .opcode(&[0x71])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPSHLDQ => GenAPI::new()
            .opcode(&[0x71])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPSHRDW => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPSHRDD => GenAPI::new()
            .opcode(&[0x73])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPSHRDQ => GenAPI::new()
            .opcode(&[0x73])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),

        Mnemonic::VPSHLDVW => GenAPI::new()
            .opcode(&[0x70])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSHLDVD => GenAPI::new()
            .opcode(&[0x71])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSHLDVQ => GenAPI::new()
            .opcode(&[0x71])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSHRDVW => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSHRDVD => GenAPI::new()
            .opcode(&[0x73])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSHRDVQ => GenAPI::new()
            .opcode(&[0x73])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSHUFBITQMB => GenAPI::new()
            .opcode(&[0x8F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSLLVW => GenAPI::new()
            .opcode(&[0x12])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSLLVD => GenAPI::new()
            .opcode(&[0x47])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPSLLVQ => GenAPI::new()
            .opcode(&[0x47])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),

        Mnemonic::VPSRAVW => GenAPI::new()
            .opcode(&[0x11])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSRAVD => GenAPI::new()
            .opcode(&[0x46])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPSRAVQ => GenAPI::new()
            .opcode(&[0x46])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),

        Mnemonic::VPSRLVW => GenAPI::new()
            .opcode(&[0x10])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSRLVD => GenAPI::new()
            .opcode(&[0x45])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPSRLVQ => GenAPI::new()
            .opcode(&[0x45])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),

        Mnemonic::VPTERNLOGD => GenAPI::new()
            .opcode(&[0x25])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTERNLOGQ => GenAPI::new()
            .opcode(&[0x25])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),

        Mnemonic::VPTESTMB => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTESTMW => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPTESTMD => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTESTMQ => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPTESTNMB => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTESTNMW => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPTESTNMD => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTESTNMQ => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRANGEPS => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VRANGEPD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRANGESS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VRANGESD => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),

        Mnemonic::VRCP14PS => GenAPI::new()
            .opcode(&[0x4C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRCP14PD => GenAPI::new()
            .opcode(&[0x4C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRCP14SS => GenAPI::new()
            .opcode(&[0x4D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRCP14SD => GenAPI::new()
            .opcode(&[0x4D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRCPPH => GenAPI::new()
            .opcode(&[0x4C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VRCPSH => GenAPI::new()
            .opcode(&[0x4D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VREDUCEPS => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VREDUCEPD => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VREDUCEPH => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VREDUCESS => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VREDUCESD => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VREDUCESH => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),

        Mnemonic::VRNDSCALEPS => GenAPI::new()
            .opcode(&[0x08])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VRNDSCALEPD => GenAPI::new()
            .opcode(&[0x09])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRNDSCALEPH => GenAPI::new()
            .opcode(&[0x08])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VRNDSCALESS => GenAPI::new()
            .opcode(&[0x0A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VRNDSCALESD => GenAPI::new()
            .opcode(&[0x0B])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRNDSCALESH => GenAPI::new()
            .opcode(&[0x0A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),

        Mnemonic::VRSQRT14PS => GenAPI::new()
            .opcode(&[0x4E])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRSQRT14PD => GenAPI::new()
            .opcode(&[0x4E])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRSQRT14SS => GenAPI::new()
            .opcode(&[0x4F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRSQRT14SD => GenAPI::new()
            .opcode(&[0x4F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRSQRTPH => GenAPI::new()
            .opcode(&[0x4E])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VRSQRTSH => GenAPI::new()
            .opcode(&[0x4F])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VSCALEFPS => GenAPI::new()
            .opcode(&[0x2C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCALEFPD => GenAPI::new()
            .opcode(&[0x2C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCALEFSS => GenAPI::new()
            .opcode(&[0x2D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCALEFSD => GenAPI::new()
            .opcode(&[0x2D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCALEFPH => GenAPI::new()
            .opcode(&[0x2C])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VSCALEFSH => GenAPI::new()
            .opcode(&[0x2D])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VSHA512MSG1 => GenAPI::new()
            .opcode(&[0xCC])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),
        Mnemonic::VSHA512MSG2 => GenAPI::new()
            .opcode(&[0xCD])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),
        Mnemonic::VSHA512RNDS2 => GenAPI::new()
            .opcode(&[0xCB])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),

        Mnemonic::VSHUFF32X4 => GenAPI::new()
            .opcode(&[0x23])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VSHUFF64X2 => GenAPI::new()
            .opcode(&[0x23])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VSHUFI32X4 => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VSHUFI64X2 => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VSM3MSG1 => GenAPI::new()
            .opcode(&[0xDA])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(0x38).vex_we(false)),
        Mnemonic::VSM3MSG2 => GenAPI::new()
            .opcode(&[0xDA])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VSM3RNDS2 => GenAPI::new()
            .opcode(&[0xDE])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .imm_atindex(3, 1)
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false)),
        Mnemonic::VSM4KEY4 => GenAPI::new()
            .opcode(&[0xDA])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(0x38).vex_we(false)),
        Mnemonic::VSM4RNDS4 => GenAPI::new()
            .opcode(&[0xDA])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),
        Mnemonic::VSQRTPH => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VSQRTSH => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VSUBPH => GenAPI::new()
            .opcode(&[0x5C])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VSUBSH => GenAPI::new()
            .opcode(&[0x5C])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VTESTPS => GenAPI::new()
            .opcode(&[0x0E])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VTESTPD => GenAPI::new()
            .opcode(&[0x0F])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VUCOMISH => GenAPI::new()
            .opcode(&[0x2E])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::PREFETCHWT1 => GenAPI::new().opcode(&[0x0F, 0x0D]).modrm(true, Some(2)),
        Mnemonic::V4FMADDPS => GenAPI::new()
            .opcode(&[0x9A])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::V4FNMADDPS => GenAPI::new()
            .opcode(&[0xAA])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::V4FMADDSS => GenAPI::new()
            .opcode(&[0xAB])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::V4FNMADDSS => GenAPI::new()
            .opcode(&[0xAB])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VEXP2PS => GenAPI::new()
            .opcode(&[0xC8])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VEXP2PD => GenAPI::new()
            .opcode(&[0xC8])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VP4DPWSSDS => GenAPI::new()
            .opcode(&[0x53])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VP4DPWSSD => GenAPI::new()
            .opcode(&[0x52])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),

        Mnemonic::VRCP28PD => GenAPI::new()
            .opcode(&[0xCA])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRCP28SD => GenAPI::new()
            .opcode(&[0xCB])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRCP28PS => GenAPI::new()
            .opcode(&[0xCA])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRCP28SS => GenAPI::new()
            .opcode(&[0xCB])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),

        Mnemonic::VRSQRT28PD => GenAPI::new()
            .opcode(&[0xCC])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRSQRT28SD => GenAPI::new()
            .opcode(&[0xCD])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRSQRT28PS => GenAPI::new()
            .opcode(&[0xCC])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRSQRT28SS => GenAPI::new()
            .opcode(&[0xCD])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPGATHERDD => GenAPI::new()
            .opcode(&[0x90])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPGATHERDQ => GenAPI::new()
            .opcode(&[0x90])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),
        Mnemonic::VPGATHERQD => GenAPI::new()
            .opcode(&[0x91])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPGATHERQQ => GenAPI::new()
            .opcode(&[0x91])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),
        Mnemonic::VSCATTERDPS => GenAPI::new()
            .opcode(&[0xA2])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERDPD => GenAPI::new()
            .opcode(&[0xA2])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCATTERQPS => GenAPI::new()
            .opcode(&[0xA3])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERQPD => GenAPI::new()
            .opcode(&[0xA3])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERPF0DPS => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(1))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGATHERPF0DPD => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(1))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERPF0QPS => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(1))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGATHERPF0QPD => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(1))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERPF1DPS => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(2))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGATHERPF1DPD => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(2))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERPF1QPS => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(2))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGATHERPF1QPD => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(2))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),

        Mnemonic::VSCATTERPF0DPS => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(5))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERPF0DPD => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(5))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCATTERPF0QPS => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(5))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERPF0QPD => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(5))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCATTERPF1DPS => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(6))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERPF1DPD => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(6))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCATTERPF1QPS => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(6))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERPF1QPD => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(6))
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGETEXPSD => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERDPS => GenAPI::new()
            .opcode(&[0x92])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VGATHERDPD => GenAPI::new()
            .opcode(&[0x92])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),
        Mnemonic::VGATHERQPS => GenAPI::new()
            .opcode(&[0x93])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VGATHERQPD => GenAPI::new()
            .opcode(&[0x93])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),

        Mnemonic::VPSCATTERDD => GenAPI::new()
            .opcode(&[0xA0])
            .modrm(true, None)
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSCATTERDQ => GenAPI::new()
            .opcode(&[0xA0])
            .modrm(true, None)
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSCATTERQD => GenAPI::new()
            .opcode(&[0xA1])
            .modrm(true, None)
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSCATTERQQ => GenAPI::new()
            .opcode(&[0xA1])
            .modrm(true, None)
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),

        Mnemonic::AAADD => GenAPI::new()
            .opcode(&[0xFC])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::AAXOR => GenAPI::new()
            .opcode(&[0xFC])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().pp(0xF3).map_select(MAP4),
                false,
            ),
        Mnemonic::AAOR => GenAPI::new()
            .opcode(&[0xFC])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().pp(0xF2).map_select(MAP4),
                false,
            ),
        Mnemonic::AAAND => GenAPI::new()
            .opcode(&[0xFC])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().pp(0x66).map_select(MAP4),
                false,
            ),
        Mnemonic::AADC => ins_aadd(
            ins,
            &[0x10],
            &[0x11],
            &[0x81],
            &[0x83],
            &[0x80],
            &[0x12],
            &[0x13],
            2,
        )
        .apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::AADD => ins_aadd(
            ins,
            &[0x00],
            &[0x01],
            &[0x81],
            &[0x83],
            &[0x80],
            &[0x02],
            &[0x03],
            0,
        )
        .apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::AADCX => ins_aadox(ins, &[0x66]).apx(
            APXVariant::LegacyExtension,
            VexDetails::new()
                .map_select(MAP4)
                .vex_we(ins.size() == Size::Qword)
                .pp(0x66),
            false,
        ),
        Mnemonic::AADOX => ins_aadox(ins, &[0x66]).apx(
            APXVariant::LegacyExtension,
            VexDetails::new()
                .map_select(MAP4)
                .vex_we(ins.size() == Size::Qword)
                .pp(0xF3),
            false,
        ),
        Mnemonic::AAND => ins_aadd(
            ins,
            &[0x20],
            &[0x21],
            &[0x81],
            &[0x83],
            &[0x80],
            &[0x22],
            &[0x23],
            4,
        )
        .apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::AANDN => GenAPI::new()
            .opcode(&[0xF2])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::VexExtension,
                VexDetails::new()
                    .map_select(MAP38)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ABZHI => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .apx(
                APXVariant::VexExtension,
                VexDetails::new()
                    .map_select(MAP38)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ABEXTR => GenAPI::new()
            .opcode(&[0xF7])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .apx(
                APXVariant::VexExtension,
                VexDetails::new()
                    .map_select(MAP38)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ABLSMSK => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(2))
            .ord(&[VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::VexExtension,
                VexDetails::new()
                    .map_select(MAP38)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ABLSR => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(1))
            .ord(&[VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::VexExtension,
                VexDetails::new()
                    .map_select(MAP38)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ABLSI => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(3))
            .ord(&[VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::VexExtension,
                VexDetails::new()
                    .map_select(MAP38)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),

        Mnemonic::ACMOVA => ins_cmovcc(ins, &[0x0F, 0x47], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVAE => ins_cmovcc(ins, &[0x0F, 0x43], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVB => ins_cmovcc(ins, &[0x0F, 0x42], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVBE => ins_cmovcc(ins, &[0x0F, 0x46], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVC => ins_cmovcc(ins, &[0x0F, 0x42], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVE => ins_cmovcc(ins, &[0x0F, 0x44], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVG => ins_cmovcc(ins, &[0x0F, 0x4F], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVGE => ins_cmovcc(ins, &[0x0F, 0x4D], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVL => ins_cmovcc(ins, &[0x0F, 0x4C], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVLE => ins_cmovcc(ins, &[0x0F, 0x4E], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNA => ins_cmovcc(ins, &[0x0F, 0x46], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNB => ins_cmovcc(ins, &[0x0F, 0x43], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNBE => ins_cmovcc(ins, &[0x0F, 0x47], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNC => ins_cmovcc(ins, &[0x0F, 0x43], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNE => ins_cmovcc(ins, &[0x0F, 0x45], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNG => ins_cmovcc(ins, &[0x0F, 0x4E], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNGE => ins_cmovcc(ins, &[0x0F, 0x4C], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNL => ins_cmovcc(ins, &[0x0F, 0x4D], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNLE => ins_cmovcc(ins, &[0x0F, 0x4F], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNAE => ins_cmovcc(ins, &[0x0F, 0x42], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNO => ins_cmovcc(ins, &[0x0F, 0x41], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNP => ins_cmovcc(ins, &[0x0F, 0x4B], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNS => ins_cmovcc(ins, &[0x0F, 0x49], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVNZ => ins_cmovcc(ins, &[0x0F, 0x45], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVO => ins_cmovcc(ins, &[0x0F, 0x40], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVP => ins_cmovcc(ins, &[0x0F, 0x4A], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVPO => ins_cmovcc(ins, &[0x0F, 0x4B], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVS => ins_cmovcc(ins, &[0x0F, 0x48], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVZ => ins_cmovcc(ins, &[0x0F, 0x44], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACMOVPE => ins_cmovcc(ins, &[0x0F, 0x4A], bits)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::ACRC32 => GenAPI::new()
            .opcode(&[0xF1 - (ins.size() == Size::Byte) as u8])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::AIDIV => GenAPI::new()
            .opcode(&[0xF7 - (ins.size() == Size::Byte) as u8])
            .modrm(true, Some(7))
            .ord(&[MODRM_RM, MODRM_REG])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::AIMUL => ins_aimul(ins).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::AIMULZU => ins_imul(ins, bits).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::ADIV => GenAPI::new()
            .opcode(&[0xF7 - (ins.size() == Size::Byte) as u8])
            .modrm(true, Some(6))
            .ord(&[MODRM_RM, MODRM_REG])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::AINC => {
            let mut api = GenAPI::new()
                .opcode(&[0xFF - (ins.size() == Size::Byte) as u8])
                .modrm(true, Some(0))
                .apx(
                    APXVariant::LegacyExtension,
                    VexDetails::new().map_select(MAP4),
                    false,
                );
            api = if ins.src().is_some() {
                api.ord(&[VEX_VVVV, MODRM_RM])
            } else {
                api.ord(&[MODRM_RM, MODRM_REG])
            };
            api
        }
        Mnemonic::ADEC => {
            let mut api = GenAPI::new()
                .opcode(&[0xFF - (ins.size() == Size::Byte) as u8])
                .modrm(true, Some(1))
                .apx(
                    APXVariant::LegacyExtension,
                    VexDetails::new().map_select(MAP4),
                    false,
                );
            api = if ins.src().is_some() {
                api.ord(&[VEX_VVVV, MODRM_RM])
            } else {
                api.ord(&[MODRM_RM, MODRM_REG])
            };
            api
        }
        Mnemonic::AINVEPT => GenAPI::new()
            .opcode(&[0xF0])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF3),
                false,
            ),
        Mnemonic::AINVVPID => GenAPI::new()
            .opcode(&[0xF1])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF3),
                false,
            ),
        Mnemonic::AINVPCID => GenAPI::new()
            .opcode(&[0xF2])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF3),
                false,
            ),
        Mnemonic::AKMOVB => ins_kmov(ins).apx(
            APXVariant::VexExtension,
            VexDetails::new()
                .map_select(0x0F)
                .pp(0x66)
                .vex_we(false)
                .vlength(Some(false)),
            false,
        ),
        Mnemonic::AKMOVW => ins_kmov(ins).apx(
            APXVariant::VexExtension,
            VexDetails::new()
                .map_select(0x0F)
                .vex_we(false)
                .vlength(Some(false)),
            false,
        ),
        Mnemonic::AKMOVD => {
            let api = ins_kmov(ins);
            let mut vd = VexDetails::new()
                .map_select(0x0F)
                .pp(0x66)
                .vex_we(true)
                .vlength(Some(false));
            let opc = api.get_opcode();
            let opc = opc[0];

            if opc == 0x92 || opc == 0x93 {
                vd = vd.pp(0xF2).vex_we(false);
            }
            api.apx(APXVariant::VexExtension, vd, false)
        }
        Mnemonic::AKMOVQ => {
            let api = ins_kmov(ins);
            let mut vd = VexDetails::new()
                .map_select(0x0F)
                .vex_we(true)
                .vlength(Some(false));
            let opc = api.get_opcode();
            let opc = opc[0];

            if opc == 0x92 || opc == 0x93 {
                vd = vd.pp(0xF2);
            }
            api.apx(APXVariant::VexExtension, vd, false)
        }
        Mnemonic::ALZCNT => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP38),
                false,
            ),
        Mnemonic::AMOVBE => {
            if ins.dst().unwrap().is_mem() {
                GenAPI::new()
                    .opcode(&[0x61])
                    .modrm(true, None)
                    .ord(&[MODRM_RM, MODRM_REG])
                    .apx(
                        APXVariant::LegacyExtension,
                        VexDetails::new().map_select(MAP4),
                        false,
                    )
            } else {
                GenAPI::new()
                    .opcode(&[0x60])
                    .modrm(true, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .apx(
                        APXVariant::LegacyExtension,
                        VexDetails::new().map_select(MAP4),
                        false,
                    )
            }
        }
        Mnemonic::AMOVDIRI => GenAPI::new()
            .opcode(&[0xF9])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::AMOVRS => GenAPI::new()
            .opcode(&[0x8B - (ins.size() == Size::Byte) as u8])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::AMUL => GenAPI::new()
            .opcode(&[0xF7 - (ins.size() == Size::Byte) as u8])
            .modrm(true, Some(4))
            .ord(&[MODRM_RM, MODRM_REG])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::AMULX => GenAPI::new()
            .opcode(&[0xF6])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new()
                    .map_select(MAP38)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ANOT => {
            let mut api = GenAPI::new()
                .opcode(&[0xF7 - (ins.size() == Size::Byte) as u8])
                .modrm(true, Some(2))
                .apx(
                    APXVariant::LegacyExtension,
                    VexDetails::new().map_select(MAP4),
                    false,
                );
            api = if ins.src().is_some() {
                api.ord(&[VEX_VVVV, MODRM_RM])
            } else {
                api.ord(&[MODRM_RM, MODRM_REG])
            };
            api
        }
        Mnemonic::ANEG => {
            let mut api = GenAPI::new()
                .opcode(&[0xF7 - (ins.size() == Size::Byte) as u8])
                .modrm(true, Some(3))
                .apx(
                    APXVariant::LegacyExtension,
                    VexDetails::new().map_select(MAP4),
                    false,
                );
            api = if ins.src().is_some() {
                api.ord(&[VEX_VVVV, MODRM_RM])
            } else {
                api.ord(&[MODRM_RM, MODRM_REG])
            };
            api
        }
        Mnemonic::AOR => ins_aadd(
            ins,
            &[0x0A],
            &[0x0B],
            &[0x81],
            &[0x83],
            &[0x80],
            &[0x08],
            &[0x09],
            1,
        )
        .apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::APEXT => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::Auto,
                VexDetails::new()
                    .map_select(MAP38)
                    .pp(0xF3)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::APDEP => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::Auto,
                VexDetails::new()
                    .map_select(MAP38)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::APOPCNT => GenAPI::new()
            .opcode(&[0x88])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(APXVariant::Auto, VexDetails::new().map_select(MAP4), false),
        Mnemonic::ARDMSR => GenAPI::new()
            .opcode(&[0xF6])
            .modrm(true, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(1, 4)
            .apx(
                APXVariant::Auto,
                VexDetails::new().map_select(7).pp(0xF2),
                false,
            ),
        Mnemonic::ASARX => GenAPI::new()
            .opcode(&[0xF7])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .apx(
                APXVariant::Auto,
                VexDetails::new()
                    .map_select(MAP38)
                    .pp(0xF3)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ARORX => GenAPI::new()
            .opcode(&[0xF0])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .apx(
                APXVariant::Auto,
                VexDetails::new()
                    .map_select(MAP3A)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ARCL => ins_ashllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 2).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::ARCR => ins_ashllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 3).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::AROL => ins_ashllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 0).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::AROR => ins_ashllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 1).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::ASAR => ins_ashllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 7).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::ASHL => ins_ashllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 4).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::ASHR => ins_ashllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 5).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::ASHLD => ins_ashlx(ins, &[0x24], &[0xA5]).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::ASHLX => GenAPI::new()
            .opcode(&[0xF7])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .apx(
                APXVariant::VexExtension,
                VexDetails::new()
                    .map_select(MAP38)
                    .pp(0x66)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ASHRD => ins_ashlx(ins, &[0x2C], &[0xAD]).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::ASHRX => GenAPI::new()
            .opcode(&[0xF7])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .apx(
                APXVariant::VexExtension,
                VexDetails::new()
                    .map_select(MAP38)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
                false,
            ),
        Mnemonic::ASUB => ins_aadd(
            ins,
            &[0x28],
            &[0x29],
            &[0x81],
            &[0x83],
            &[0x80],
            &[0x2A],
            &[0x2B],
            5,
        )
        .apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::ATZCNT => GenAPI::new()
            .opcode(&[0xF4])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            ),
        Mnemonic::AXOR => ins_aadd(
            ins,
            &[0x30],
            &[0x31],
            &[0x81],
            &[0x83],
            &[0x80],
            &[0x32],
            &[0x33],
            6,
        )
        .apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::JMPABS => GenAPI::new().opcode(&[0xA1]).imm_atindex(0, 8).apx(
            APXVariant::Rex2,
            VexDetails::new(),
            false,
        ),
        Mnemonic::CCMPB => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_B),
        Mnemonic::CCMPBE => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_BE),
        Mnemonic::CCMPNBE => {
            ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NBE)
        }
        Mnemonic::CCMPNB => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NB),

        Mnemonic::CCMPL => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_L),
        Mnemonic::CCMPLE => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_LE),
        Mnemonic::CCMPNL => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NL),
        Mnemonic::CCMPNLE => {
            ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NLE)
        }

        Mnemonic::CCMPT => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(0b1010),
        Mnemonic::CCMPF => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(0b1011),

        Mnemonic::CCMPNO => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NO),
        Mnemonic::CCMPO => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_O),

        Mnemonic::CCMPNZ => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NZ),
        Mnemonic::CCMPZ => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_Z),

        Mnemonic::CCMPNE => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NE),
        Mnemonic::CCMPE => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_E),

        Mnemonic::CCMPS => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_S),
        Mnemonic::CCMPNS => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NS),

        Mnemonic::CCMPC => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_C),
        Mnemonic::CCMPNC => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NC),

        Mnemonic::CCMPG => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_G),
        Mnemonic::CCMPGE => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_GE),
        Mnemonic::CCMPNGE => {
            ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NGE)
        }
        Mnemonic::CCMPNG => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NG),

        Mnemonic::CCMPA => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_A),
        Mnemonic::CCMPAE => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_AE),
        Mnemonic::CCMPNAE => {
            ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NAE)
        }
        Mnemonic::CCMPNA => ins_ccmp(ins, &[0x38], &[0x3B], &[0x81], &[0x83], 7).apx_cccc(COND_NA),

        // idk, but Intel allows for both 1 and 0 for modrm_ovr for r/m, iX?
        Mnemonic::CTESTB => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_B),
        Mnemonic::CTESTBE => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_BE),
        Mnemonic::CTESTNBE => {
            ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NBE)
        }
        Mnemonic::CTESTNB => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NB),

        Mnemonic::CTESTL => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_L),
        Mnemonic::CTESTLE => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_LE),
        Mnemonic::CTESTNL => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NL),
        Mnemonic::CTESTNLE => {
            ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NLE)
        }

        Mnemonic::CTESTT => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(0b1010),
        Mnemonic::CTESTF => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(0b1011),

        Mnemonic::CTESTNO => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NO),
        Mnemonic::CTESTO => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_O),

        Mnemonic::CTESTNZ => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NZ),
        Mnemonic::CTESTZ => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_Z),

        Mnemonic::CTESTNE => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NE),
        Mnemonic::CTESTE => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_E),

        Mnemonic::CTESTS => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_S),
        Mnemonic::CTESTNS => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NS),

        Mnemonic::CTESTC => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_C),
        Mnemonic::CTESTNC => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NC),

        Mnemonic::CTESTG => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_G),
        Mnemonic::CTESTGE => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_GE),
        Mnemonic::CTESTNGE => {
            ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NGE)
        }
        Mnemonic::CTESTNG => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NG),

        Mnemonic::CTESTA => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_A),
        Mnemonic::CTESTAE => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_AE),
        Mnemonic::CTESTNAE => {
            ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NAE)
        }
        Mnemonic::CTESTNA => ins_ccmp(ins, &[0x85], &[0xF7], &[0x85], &[0xF6], 1).apx_cccc(COND_NA),

        Mnemonic::SETOZU => GenAPI::new().opcode(&[0x40]).modrm(true, None).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4).pp(0xF2),
            false,
        ),
        Mnemonic::SETNOZU => GenAPI::new().opcode(&[0x41]).modrm(true, None).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4).pp(0xF2),
            false,
        ),
        Mnemonic::SETBZU | Mnemonic::SETCZU | Mnemonic::SETNAEZU => {
            GenAPI::new().opcode(&[0x42]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETAEZU | Mnemonic::SETNBZU | Mnemonic::SETNCZU => {
            GenAPI::new().opcode(&[0x43]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETEZU | Mnemonic::SETZZU => GenAPI::new().opcode(&[0x44]).modrm(true, None).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4).pp(0xF2),
            false,
        ),
        Mnemonic::SETNEZU | Mnemonic::SETNZZU => {
            GenAPI::new().opcode(&[0x45]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETBEZU | Mnemonic::SETNAZU => {
            GenAPI::new().opcode(&[0x46]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETAZU | Mnemonic::SETNBEZU => {
            GenAPI::new().opcode(&[0x47]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETSZU => GenAPI::new().opcode(&[0x48]).modrm(true, None).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4).pp(0xF2),
            false,
        ),
        Mnemonic::SETNSZU => GenAPI::new().opcode(&[0x49]).modrm(true, None).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4).pp(0xF2),
            false,
        ),

        Mnemonic::SETPZU | Mnemonic::SETPEZU => {
            GenAPI::new().opcode(&[0x4A]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETNPZU | Mnemonic::SETPOZU => {
            GenAPI::new().opcode(&[0x4B]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETLZU | Mnemonic::SETNGEZU => {
            GenAPI::new().opcode(&[0x4C]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETGEZU | Mnemonic::SETNLZU => {
            GenAPI::new().opcode(&[0x4D]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETLEZU | Mnemonic::SETNGZU => {
            GenAPI::new().opcode(&[0x4E]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }

        Mnemonic::SETGZU | Mnemonic::SETNLEZU => {
            GenAPI::new().opcode(&[0x4F]).modrm(true, None).apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).pp(0xF2),
                false,
            )
        }
        Mnemonic::CFCMOVA => ins_cfcmov(ins, &[0x47]),
        Mnemonic::CFCMOVAE => ins_cfcmov(ins, &[0x43]),
        Mnemonic::CFCMOVB => ins_cfcmov(ins, &[0x42]),
        Mnemonic::CFCMOVBE => ins_cfcmov(ins, &[0x46]),
        Mnemonic::CFCMOVE => ins_cfcmov(ins, &[0x44]),
        Mnemonic::CFCMOVG => ins_cfcmov(ins, &[0x4F]),
        Mnemonic::CFCMOVGE => ins_cfcmov(ins, &[0x4D]),
        Mnemonic::CFCMOVL => ins_cfcmov(ins, &[0x4C]),
        Mnemonic::CFCMOVLE => ins_cfcmov(ins, &[0x4E]),
        Mnemonic::CFCMOVNA => ins_cfcmov(ins, &[0x46]),
        Mnemonic::CFCMOVNB => ins_cfcmov(ins, &[0x43]),
        Mnemonic::CFCMOVNBE => ins_cfcmov(ins, &[0x47]),
        Mnemonic::CFCMOVNE => ins_cfcmov(ins, &[0x45]),
        Mnemonic::CFCMOVNG => ins_cfcmov(ins, &[0x4E]),
        Mnemonic::CFCMOVNGE => ins_cfcmov(ins, &[0x4C]),
        Mnemonic::CFCMOVNL => ins_cfcmov(ins, &[0x4D]),
        Mnemonic::CFCMOVNLE => ins_cfcmov(ins, &[0x4F]),
        Mnemonic::CFCMOVNAE => ins_cfcmov(ins, &[0x42]),
        Mnemonic::CFCMOVNO => ins_cfcmov(ins, &[0x41]),
        Mnemonic::CFCMOVNP => ins_cfcmov(ins, &[0x4B]),
        Mnemonic::CFCMOVNS => ins_cfcmov(ins, &[0x49]),
        Mnemonic::CFCMOVNZ => ins_cfcmov(ins, &[0x45]),
        Mnemonic::CFCMOVO => ins_cfcmov(ins, &[0x40]),
        Mnemonic::CFCMOVP => ins_cfcmov(ins, &[0x4A]),
        Mnemonic::CFCMOVPO => ins_cfcmov(ins, &[0x4B]),
        Mnemonic::CFCMOVS => ins_cfcmov(ins, &[0x48]),
        Mnemonic::CFCMOVZ => ins_cfcmov(ins, &[0x44]),
        Mnemonic::CFCMOVPE => ins_cfcmov(ins, &[0x4A]),
        Mnemonic::CFCMOVC => ins_cfcmov(ins, &[0x42]),
        Mnemonic::CFCMOVNC => ins_cfcmov(ins, &[0x43]),

        Mnemonic::POPP => GenAPI::new()
            .opcode(&[0x58 + (ins.dst().unwrap().get_reg().unwrap().to_byte())])
            .apx(
                APXVariant::Rex2,
                VexDetails::new().map_select(0).vex_we(true),
                false,
            ),
        Mnemonic::PUSHP => GenAPI::new()
            .opcode(&[0x50 + (ins.dst().unwrap().get_reg().unwrap().to_byte())])
            .apx(
                APXVariant::Rex2,
                VexDetails::new().map_select(0).vex_we(true),
                false,
            ),
        Mnemonic::PUSH2 => GenAPI::new()
            .opcode(&[0xFF])
            .modrm(true, Some(0b110))
            .ord(&[VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).vex_we(false),
                false,
            ),
        Mnemonic::PUSH2P => GenAPI::new()
            .opcode(&[0xFF])
            .modrm(true, Some(0b110))
            .ord(&[VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).vex_we(true),
                false,
            ),
        Mnemonic::POP2 => GenAPI::new()
            .opcode(&[0x8F])
            .modrm(true, None)
            .ord(&[VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).vex_we(false),
                false,
            ),
        Mnemonic::POP2P => GenAPI::new()
            .opcode(&[0x8F])
            .modrm(true, None)
            .ord(&[VEX_VVVV, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4).vex_we(true),
                false,
            ),
        Mnemonic::ASBB => ins_aadd(
            ins,
            &[0x18],
            &[0x19],
            &[0x81],
            &[0x83],
            &[0x80],
            &[0x1A],
            &[0x1B],
            3,
        )
        .apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4),
            false,
        ),
        Mnemonic::RDGSBASE => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .opcode_prefix(0xF3)
            .rex()
            .modrm(true, Some(1)),
        Mnemonic::RDFSBASE => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .opcode_prefix(0xF3)
            .rex()
            .modrm(true, Some(0)),
        Mnemonic::MOVDIR64B => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF8])
            .opcode_prefix(0x66)
            .rex()
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::WBNOINVD => GenAPI::new().opcode(&[0x0F, 0x09]).opcode_prefix(0xF3),
        Mnemonic::PREFETCHIT0 => GenAPI::new().opcode(&[0x0F, 0x18]).modrm(true, Some(6)),
        Mnemonic::PREFETCHIT1 => GenAPI::new().opcode(&[0x0F, 0x18]).modrm(true, Some(7)),
        Mnemonic::PTWRITE => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .opcode_prefix(0xF3)
            .modrm(true, Some(4)),
        Mnemonic::PCONFIG => GenAPI::new().opcode(&[0x0F, 0x01, 0xC5]),
        Mnemonic::ENQCMD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF8])
            .opcode_prefix(0xF2)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::AENQCMD => GenAPI::new()
            .opcode(&[0xF8])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().pp(0xF2).map_select(MAP4).vex_we(false),
                false,
            ),
        Mnemonic::ENQCMDS => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF8])
            .opcode_prefix(0xF3)
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::AENQCMDS => GenAPI::new()
            .opcode(&[0xF8])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().pp(0xF3).map_select(MAP4).vex_we(false),
                false,
            ),
        Mnemonic::WRMSRLIST => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xC6])
            .opcode_prefix(0xF3),
        Mnemonic::RDMSRLIST => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xC6])
            .opcode_prefix(0xF2),
        Mnemonic::PBNDKB => GenAPI::new().opcode(&[0x0F, 0x01, 0xC7]),
        Mnemonic::POPAW => GenAPI::new().opcode(&[0x61]).fixed_size(Size::Word),
        Mnemonic::POPAD => GenAPI::new().opcode(&[0x61]).fixed_size(Size::Dword),
        Mnemonic::POPAQ => GenAPI::new().opcode(&[0x61]).fixed_size(Size::Qword),
        Mnemonic::PUSHAW => GenAPI::new().opcode(&[0x60]).fixed_size(Size::Word),
        Mnemonic::PUSHAD => GenAPI::new().opcode(&[0x60]).fixed_size(Size::Dword),
        Mnemonic::PUSHAQ => GenAPI::new().opcode(&[0x60]).fixed_size(Size::Qword),
        Mnemonic::SIDT => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(1))
            .can_h66(false),
        Mnemonic::SGDT => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(0))
            .can_h66(false),
        Mnemonic::SLDT => GenAPI::new()
            .opcode(&[0x0F, 0x00])
            .modrm(true, Some(0))
            .can_h66(false),
        Mnemonic::LFS => GenAPI::new()
            .opcode(&[0x0F, 0xB4])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::LGS => GenAPI::new()
            .opcode(&[0x0F, 0xB5])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVSX => GenAPI::new()
            .opcode(&[0x0F, 0xBF - (ins.src().unwrap().size() == Size::Byte) as u8])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::MOVSXD => GenAPI::new()
            .opcode(&[0x63])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(),
        Mnemonic::NOPL => GenAPI::new().opcode(&[0x0F, 0x1F]).modrm(true, Some(1)),
        Mnemonic::SWAPGS => GenAPI::new().opcode(&[0x0F, 0x01, 0xF8]),
        Mnemonic::INVLPGA => GenAPI::new().opcode(&[0x0F, 0x01, 0xDF]),
        Mnemonic::VMRUN => GenAPI::new().opcode(&[0x0F, 0x01, 0xD8]),
        Mnemonic::VMLOAD => GenAPI::new().opcode(&[0x0F, 0x01, 0xDA]),
        Mnemonic::VMSAVE => GenAPI::new().opcode(&[0x0F, 0x01, 0xDB]),
        Mnemonic::STGI => GenAPI::new().opcode(&[0x0F, 0x01, 0xDC]),
        Mnemonic::CLGI => GenAPI::new().opcode(&[0x0F, 0x01, 0xDD]),
        Mnemonic::VMMCALL => GenAPI::new().opcode(&[0x0F, 0x01, 0xD9]),
        Mnemonic::SKINIT => GenAPI::new().opcode(&[0x0F, 0x01, 0xDE]),
        Mnemonic::VMGEXIT => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xD9])
            .opcode_prefix(0xF2),
        Mnemonic::PSMASH => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xFF])
            .opcode_prefix(0xF3),
        Mnemonic::RMPUPDATE => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xFE])
            .opcode_prefix(0xF2),
        Mnemonic::PVALIDATE => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xFF])
            .opcode_prefix(0xF2),
        Mnemonic::RMPADJUST => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xFE])
            .opcode_prefix(0xF3),
        Mnemonic::RMPQUERY => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xFD])
            .opcode_prefix(0xF3),
        Mnemonic::RMPREAD => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xFD])
            .opcode_prefix(0xF2),
        Mnemonic::VMXON => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .opcode_prefix(0xF3)
            .modrm(true, Some(6)),
        Mnemonic::VMXOFF => GenAPI::new().opcode(&[0x0F, 0x01, 0xC4]),
        Mnemonic::VMPTRLD => GenAPI::new().opcode(&[0x0F, 0xC7]).modrm(true, Some(6)),
        Mnemonic::VMPTRST => GenAPI::new().opcode(&[0x0F, 0xC7]).modrm(true, Some(7)),
        Mnemonic::VMCLEAR => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(6))
            .opcode_prefix(0x66),
        Mnemonic::VMLAUNCH => GenAPI::new().opcode(&[0x0F, 0x01, 0xC2]),
        Mnemonic::VMRESUME => GenAPI::new().opcode(&[0x0F, 0x01, 0xC3]),
        Mnemonic::VMREAD => GenAPI::new().opcode(&[0x0F, 0x78]).modrm(true, None),
        Mnemonic::VMWRITE => GenAPI::new()
            .opcode(&[0x0F, 0x79])
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VMCALL => GenAPI::new().opcode(&[0x0F, 0xC1]),
        Mnemonic::INVEPT => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x80])
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::INVVPID => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x81])
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None),
        Mnemonic::VMFUNC => GenAPI::new().opcode(&[0x0F, 0x01, 0xD4]),
        Mnemonic::SEAMOPS => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xCE])
            .opcode_prefix(0x66),
        Mnemonic::SEAMRET => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xCD])
            .opcode_prefix(0x66),
        Mnemonic::SEAMCALL => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xCF])
            .opcode_prefix(0x66),
        Mnemonic::TDCALL => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xCC])
            .opcode_prefix(0x66),
        Mnemonic::GETSEC => GenAPI::new().opcode(&[0x0F, 0x37]),
        Mnemonic::FNINIT => GenAPI::new().opcode(&[0xDB, 0xE3]),
        Mnemonic::FINIT => GenAPI::new().opcode(&[0x9B, 0xDB, 0xE3]),
        Mnemonic::FNCLEX => GenAPI::new().opcode(&[0xDB, 0xE2]),
        Mnemonic::FCLEX => GenAPI::new().opcode(&[0x9B, 0xDB, 0xE2]),
        Mnemonic::FNENI => GenAPI::new().opcode(&[0xDB, 0xE0]),
        Mnemonic::FENI => GenAPI::new().opcode(&[0x9B, 0xDB, 0xE0]),
        Mnemonic::FNDISI => GenAPI::new().opcode(&[0xDB, 0xE1]),
        Mnemonic::FDISI => GenAPI::new().opcode(&[0x9B, 0xDB, 0xE1]),
        Mnemonic::FNSTCW => GenAPI::new().opcode(&[0xD9]).modrm(true, Some(7)),
        Mnemonic::FSTCW => GenAPI::new().opcode(&[0x9B, 0xD9]).modrm(true, Some(7)),
        Mnemonic::FNSTSW => GenAPI::new().opcode(&[0xDD]).modrm(true, Some(7)),
        Mnemonic::FSTSW => GenAPI::new().opcode(&[0x9B, 0xDD]).modrm(true, Some(7)),
        Mnemonic::FLDCW => GenAPI::new().opcode(&[0xD9]).modrm(true, Some(5)),
        Mnemonic::FLDENV => GenAPI::new().opcode(&[0xD9]).modrm(true, Some(4)),
        Mnemonic::FNSTENV => GenAPI::new().opcode(&[0xD9]).modrm(true, Some(6)),
        Mnemonic::FSTENV => GenAPI::new().opcode(&[0x9B, 0xD9]).modrm(true, Some(6)),
        Mnemonic::FNSAVE => GenAPI::new().opcode(&[0xDD]).modrm(true, Some(6)),
        Mnemonic::FSAVE => GenAPI::new().opcode(&[0x9B, 0xDD]).modrm(true, Some(6)),
        Mnemonic::FRSTOR => GenAPI::new().opcode(&[0xDD]).modrm(true, Some(4)),
        Mnemonic::FLD => ins_fld(ins),
        Mnemonic::FSTP => ins_fstp(ins),
        Mnemonic::FST => ins_fst(ins),
        Mnemonic::FLDZ => GenAPI::new().opcode(&[0xD9, 0xEE]),
        Mnemonic::FLD1 => GenAPI::new().opcode(&[0xD9, 0xE8]),
        Mnemonic::FLDPI => GenAPI::new().opcode(&[0xD9, 0xEB]),
        Mnemonic::FLDL2T => GenAPI::new().opcode(&[0xD9, 0xE9]),
        Mnemonic::FLDL2E => GenAPI::new().opcode(&[0xD9, 0xEA]),
        Mnemonic::FLDLG2 => GenAPI::new().opcode(&[0xD9, 0xEC]),
        Mnemonic::FLDLN2 => GenAPI::new().opcode(&[0xD9, 0xED]),
        Mnemonic::FXCH => GenAPI::new().opcode(&[
            0xD9,
            0xC8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FILD => ins_fild(ins),
        Mnemonic::FISTP => ins_fistp(ins),
        Mnemonic::FIST => ins_fist(ins),
        Mnemonic::FBLD => GenAPI::new().opcode(&[0xDF]).modrm(true, Some(4)),
        Mnemonic::FBSTP => GenAPI::new().opcode(&[0xDF]).modrm(true, Some(6)),
        Mnemonic::FADD => ins_farthmt(ins, 0, 0xC0, 0xC0),
        Mnemonic::FMUL => ins_farthmt(ins, 1, 0xC8, 0xC8),
        Mnemonic::FSUB => ins_farthmt(ins, 4, 0xE0, 0xE8),
        Mnemonic::FSUBR => ins_farthmt(ins, 5, 0xE8, 0xE0),
        Mnemonic::FDIV => ins_farthmt(ins, 6, 0xF0, 0xF8),
        Mnemonic::FDIVR => ins_farthmt(ins, 6, 0xF8, 0xF0),
        Mnemonic::FCOM => ins_fcom(ins),
        Mnemonic::FADDP => GenAPI::new().opcode(&[
            0xDE,
            0xC0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FMULP => GenAPI::new().opcode(&[
            0xDE,
            0xC8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FSUBP => GenAPI::new().opcode(&[
            0xDE,
            0xE8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FSUBRP => GenAPI::new().opcode(&[
            0xDE,
            0xE0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FDIVRP => GenAPI::new().opcode(&[
            0xDE,
            0xF0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FDIVP => GenAPI::new().opcode(&[
            0xDE,
            0xF8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCOMP => ins_fcomp(ins),
        Mnemonic::FCOMPP => GenAPI::new().opcode(&[0xDE, 0xD9]),
        Mnemonic::FIADD => ins_fiarthmt(ins, 0),
        Mnemonic::FIMUL => ins_fiarthmt(ins, 1),
        Mnemonic::FICOM => ins_fiarthmt(ins, 2),
        Mnemonic::FICOMP => ins_fiarthmt(ins, 3),
        Mnemonic::FISUB => ins_fiarthmt(ins, 4),
        Mnemonic::FISUBR => ins_fiarthmt(ins, 5),
        Mnemonic::FIDIV => ins_fiarthmt(ins, 6),
        Mnemonic::FIDIVR => ins_fiarthmt(ins, 7),
        Mnemonic::FCHS => GenAPI::new().opcode(&[0xD9, 0xE0]),
        Mnemonic::FABS => GenAPI::new().opcode(&[0xD9, 0xE1]),
        Mnemonic::FTST => GenAPI::new().opcode(&[0xD9, 0xE4]),
        Mnemonic::FXAM => GenAPI::new().opcode(&[0xD9, 0xE5]),
        Mnemonic::FXTRACT => GenAPI::new().opcode(&[0xD9, 0xF4]),
        Mnemonic::FPREM => GenAPI::new().opcode(&[0xD9, 0xF8]),
        Mnemonic::FSQRT => GenAPI::new().opcode(&[0xD9, 0xFA]),
        Mnemonic::FRNDINT => GenAPI::new().opcode(&[0xD9, 0xFC]),
        Mnemonic::FSCALE => GenAPI::new().opcode(&[0xD9, 0xFD]),
        Mnemonic::F2XM1 => GenAPI::new().opcode(&[0xD9, 0xF0]),
        Mnemonic::FYL2X => GenAPI::new().opcode(&[0xD9, 0xF1]),
        Mnemonic::FPTAN => GenAPI::new().opcode(&[0xD9, 0xF2]),
        Mnemonic::FPATAN => GenAPI::new().opcode(&[0xD9, 0xF3]),
        Mnemonic::FYL2XP1 => GenAPI::new().opcode(&[0xD9, 0xF9]),
        Mnemonic::FNOP => GenAPI::new().opcode(&[0xD9, 0xD0]),
        Mnemonic::FDECSTP => GenAPI::new().opcode(&[0xD9, 0xF6]),
        Mnemonic::FINCSTP => GenAPI::new().opcode(&[0xD9, 0xF7]),
        Mnemonic::FFREE => GenAPI::new().opcode(&[
            0xDD,
            0xC0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FNSTSWAX => GenAPI::new().opcode(&[0xDF, 0xE0]),
        Mnemonic::FSTSWAX => GenAPI::new().opcode(&[0x9B, 0xDF, 0xE0]),
        Mnemonic::FSETPM => GenAPI::new().opcode(&[0x9B, 0xD8, 0xE4]),
        Mnemonic::FNSETPM => GenAPI::new().opcode(&[0xD8, 0xE4]),
        Mnemonic::FUCOM => GenAPI::new().opcode(&[
            0xDD,
            0xE0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FUCOMP => GenAPI::new().opcode(&[
            0xDD,
            0xE8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FUCOMPP => GenAPI::new().opcode(&[0xDA, 0xE9]),
        Mnemonic::FPREM1 => GenAPI::new().opcode(&[0xD9, 0xF5]),
        Mnemonic::FSINCOS => GenAPI::new().opcode(&[0xD9, 0xFB]),
        Mnemonic::FSIN => GenAPI::new().opcode(&[0xD9, 0xFE]),
        Mnemonic::FCOS => GenAPI::new().opcode(&[0xD9, 0xFF]),
        Mnemonic::FCMOVB => GenAPI::new().opcode(&[
            0xDA,
            0xC0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCMOVE => GenAPI::new().opcode(&[
            0xDA,
            0xC8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCMOVBE => GenAPI::new().opcode(&[
            0xDA,
            0xD0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCMOVU => GenAPI::new().opcode(&[
            0xDA,
            0xD8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCMOVNB => GenAPI::new().opcode(&[
            0xDB,
            0xC0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCMOVNE => GenAPI::new().opcode(&[
            0xDB,
            0xC8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCMOVNBE => GenAPI::new().opcode(&[
            0xDB,
            0xD0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCMOVNU => GenAPI::new().opcode(&[
            0xDB,
            0xD8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCOMI => GenAPI::new().opcode(&[
            0xDB,
            0xF0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FCOMIP => GenAPI::new().opcode(&[
            0xDF,
            0xF0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FUCOMIP => GenAPI::new().opcode(&[
            0xDF,
            0xE8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FUCOMI => GenAPI::new().opcode(&[
            0xDF,
            0xE8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]),
        Mnemonic::FXSAVE => GenAPI::new().opcode(&[0x0F, 0xAE]).modrm(true, Some(0)),
        Mnemonic::FXRSTOR => GenAPI::new().opcode(&[0x0F, 0xAE]).modrm(true, Some(1)),
        Mnemonic::FXSAVE64 => GenAPI::new()
            .opcode(&[0x48, 0x0F, 0xAE])
            .modrm(true, Some(0)),
        Mnemonic::FXRSTOR64 => GenAPI::new()
            .opcode(&[0x48, 0x0F, 0xAE])
            .modrm(true, Some(1)),
        Mnemonic::FISTTP => ins_fisttp(ins),
        Mnemonic::LOADIWKEY => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xDC])
            .opcode_prefix(0xF3)
            .modrm(true, None),
        Mnemonic::ENCODEKEY128 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xFA])
            .opcode_prefix(0xF3)
            .modrm(true, None),
        Mnemonic::ENCODEKEY256 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xFB])
            .opcode_prefix(0xF3)
            .modrm(true, None),
        Mnemonic::AESENC128KL => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xDC])
            .opcode_prefix(0xF3)
            .modrm(true, None),
        Mnemonic::AESDEC128KL => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xDD])
            .opcode_prefix(0xF3)
            .modrm(true, None),
        Mnemonic::AESENC256KL => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xDE])
            .opcode_prefix(0xF3)
            .modrm(true, None),
        Mnemonic::AESDEC256KL => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xDF])
            .opcode_prefix(0xF3)
            .modrm(true, None),
        Mnemonic::AESENCWIDE128KL => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xD8])
            .opcode_prefix(0xF3)
            .modrm(true, Some(0)),
        Mnemonic::AESDECWIDE128KL => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xD8])
            .opcode_prefix(0xF3)
            .modrm(true, Some(1)),
        Mnemonic::AESENCWIDE256KL => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xD8])
            .opcode_prefix(0xF3)
            .modrm(true, Some(2)),
        Mnemonic::AESDECWIDE256KL => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xD8])
            .opcode_prefix(0xF3)
            .modrm(true, Some(3)),

        Mnemonic::CMPAXADD => ins_cmpccxadd(ins, &[0xE7]),
        Mnemonic::CMPAEXADD => ins_cmpccxadd(ins, &[0xE3]),
        Mnemonic::CMPBXADD => ins_cmpccxadd(ins, &[0xE2]),
        Mnemonic::CMPBEXADD => ins_cmpccxadd(ins, &[0xE6]),
        Mnemonic::CMPCXADD => ins_cmpccxadd(ins, &[0xE2]),
        Mnemonic::CMPNCXADD => ins_cmpccxadd(ins, &[0xE3]),
        Mnemonic::CMPEXADD => ins_cmpccxadd(ins, &[0xE4]),
        Mnemonic::CMPGXADD => ins_cmpccxadd(ins, &[0xEF]),
        Mnemonic::CMPGEXADD => ins_cmpccxadd(ins, &[0xED]),
        Mnemonic::CMPLXADD => ins_cmpccxadd(ins, &[0xEC]),
        Mnemonic::CMPLEXADD => ins_cmpccxadd(ins, &[0xEE]),
        Mnemonic::CMPNAXADD => ins_cmpccxadd(ins, &[0xE6]),
        Mnemonic::CMPNBXADD => ins_cmpccxadd(ins, &[0xE3]),
        Mnemonic::CMPNBEXADD => ins_cmpccxadd(ins, &[0xE7]),
        Mnemonic::CMPNEXADD => ins_cmpccxadd(ins, &[0xE5]),
        Mnemonic::CMPNGXADD => ins_cmpccxadd(ins, &[0xEE]),
        Mnemonic::CMPNGEXADD => ins_cmpccxadd(ins, &[0xEC]),
        Mnemonic::CMPNLXADD => ins_cmpccxadd(ins, &[0xED]),
        Mnemonic::CMPNLEXADD => ins_cmpccxadd(ins, &[0xEF]),
        Mnemonic::CMPNAEXADD => ins_cmpccxadd(ins, &[0xE2]),
        Mnemonic::CMPNOXADD => ins_cmpccxadd(ins, &[0xE1]),
        Mnemonic::CMPNSXADD => ins_cmpccxadd(ins, &[0xE9]),
        Mnemonic::CMPNZXADD => ins_cmpccxadd(ins, &[0xE5]),
        Mnemonic::CMPOXADD => ins_cmpccxadd(ins, &[0xE0]),
        Mnemonic::CMPSXADD => ins_cmpccxadd(ins, &[0xE8]),
        Mnemonic::CMPZXADD => ins_cmpccxadd(ins, &[0xE4]),

        Mnemonic::ACMPAXADD => ins_acmpccxadd(ins, &[0xE7]),
        Mnemonic::ACMPAEXADD => ins_acmpccxadd(ins, &[0xE3]),
        Mnemonic::ACMPBXADD => ins_acmpccxadd(ins, &[0xE2]),
        Mnemonic::ACMPBEXADD => ins_acmpccxadd(ins, &[0xE6]),
        Mnemonic::ACMPCXADD => ins_acmpccxadd(ins, &[0xE2]),
        Mnemonic::ACMPNCXADD => ins_acmpccxadd(ins, &[0xE3]),
        Mnemonic::ACMPEXADD => ins_acmpccxadd(ins, &[0xE4]),
        Mnemonic::ACMPGXADD => ins_acmpccxadd(ins, &[0xEF]),
        Mnemonic::ACMPGEXADD => ins_acmpccxadd(ins, &[0xED]),
        Mnemonic::ACMPLXADD => ins_acmpccxadd(ins, &[0xEC]),
        Mnemonic::ACMPLEXADD => ins_acmpccxadd(ins, &[0xEE]),
        Mnemonic::ACMPNAXADD => ins_acmpccxadd(ins, &[0xE6]),
        Mnemonic::ACMPNBXADD => ins_acmpccxadd(ins, &[0xE3]),
        Mnemonic::ACMPNBEXADD => ins_acmpccxadd(ins, &[0xE7]),
        Mnemonic::ACMPNEXADD => ins_acmpccxadd(ins, &[0xE5]),
        Mnemonic::ACMPNGXADD => ins_acmpccxadd(ins, &[0xEE]),
        Mnemonic::ACMPNGEXADD => ins_acmpccxadd(ins, &[0xEC]),
        Mnemonic::ACMPNLXADD => ins_acmpccxadd(ins, &[0xED]),
        Mnemonic::ACMPNLEXADD => ins_acmpccxadd(ins, &[0xEF]),
        Mnemonic::ACMPNAEXADD => ins_acmpccxadd(ins, &[0xE2]),
        Mnemonic::ACMPNOXADD => ins_acmpccxadd(ins, &[0xE1]),
        Mnemonic::ACMPNSXADD => ins_acmpccxadd(ins, &[0xE9]),
        Mnemonic::ACMPNZXADD => ins_acmpccxadd(ins, &[0xE5]),
        Mnemonic::ACMPOXADD => ins_acmpccxadd(ins, &[0xE0]),
        Mnemonic::ACMPSXADD => ins_acmpccxadd(ins, &[0xE8]),
        Mnemonic::ACMPZXADD => ins_acmpccxadd(ins, &[0xE4]),
        Mnemonic::ENCLV => GenAPI::new().opcode(&[0x0F, 0x01, 0x20]),
        Mnemonic::ENCLS => GenAPI::new().opcode(&[0x0F, 0x01, 0xCF]),
        Mnemonic::ENCLU => GenAPI::new().opcode(&[0x0F, 0x01, 0xD7]),
        Mnemonic::INCSSPD => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .opcode_prefix(0xF3)
            .modrm(true, Some(5)),
        Mnemonic::INCSSPQ => GenAPI::new()
            .rex()
            .opcode(&[0x0F, 0xAE])
            .opcode_prefix(0xF3)
            .modrm(true, Some(5)),
        Mnemonic::SAVEPREVSSP => GenAPI::new()
            .opcode(&[0x0F, 0x01, 0xEA, 0b00_101_010])
            .opcode_prefix(0xF3),
        Mnemonic::WRSSD => GenAPI::new().opcode(&[0x0F, 0x38, 0xF6]).modrm(true, None),
        Mnemonic::WRSSQ => GenAPI::new()
            .rex()
            .opcode(&[0x0F, 0x38, 0xF6])
            .modrm(true, None),
        Mnemonic::WRUSSD => GenAPI::new()
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xF5])
            .modrm(true, None),
        Mnemonic::WRUSSQ => GenAPI::new()
            .rex()
            .opcode_prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xF5])
            .modrm(true, None),
        Mnemonic::CLRSSBSY => GenAPI::new()
            .rex()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(6))
            .opcode_prefix(0xF3),

        Mnemonic::AWRSSD => GenAPI::new().opcode(&[0x66]).modrm(true, None).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4).vex_we(false),
            false,
        ),
        Mnemonic::AWRSSQ => GenAPI::new().opcode(&[0x66]).modrm(true, None).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4).vex_we(true),
            false,
        ),
        Mnemonic::AWRUSSD => GenAPI::new().opcode(&[0x65]).modrm(true, None).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4).vex_we(false).pp(0x66),
            false,
        ),
        Mnemonic::AWRUSSQ => GenAPI::new().opcode(&[0x65]).modrm(true, None).apx(
            APXVariant::LegacyExtension,
            VexDetails::new().map_select(MAP4).vex_we(true).pp(0x66),
            false,
        ),
        Mnemonic::LJMP => ins_ljmp(ins),
        Mnemonic::LCALL => ins_lcall(ins),

        Mnemonic::PMOVSXBW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x20])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVSXBD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x21])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVSXBQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x22])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVSXWD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x23])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVSXWQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x24])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVSXDQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x25])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVZXBW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x30])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVZXBD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x31])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVZXBQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x32])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVZXWD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x33])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVZXWQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x34])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMOVZXDQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x35])
            .modrm(true, None)
            .opcode_prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::VPMOVSXBW => GenAPI::new()
            .opcode(&[0x20])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVSXBD => GenAPI::new()
            .opcode(&[0x21])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVSXBQ => GenAPI::new()
            .opcode(&[0x22])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVSXWD => GenAPI::new()
            .opcode(&[0x23])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVSXWQ => GenAPI::new()
            .opcode(&[0x24])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVSXDQ => GenAPI::new()
            .opcode(&[0x25])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVZXBW => GenAPI::new()
            .opcode(&[0x30])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVZXBD => GenAPI::new()
            .opcode(&[0x31])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVZXBQ => GenAPI::new()
            .opcode(&[0x32])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVZXWD => GenAPI::new()
            .opcode(&[0x33])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVZXWQ => GenAPI::new()
            .opcode(&[0x34])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMOVZXDQ => GenAPI::new()
            .opcode(&[0x35])
            .modrm(true, None)
            .vex(VexDetails::new().map_select(0x38).pp(0x66))
            .ord(&[MODRM_REG, MODRM_RM]),
    }
}

// #  #   #   ####  #####  #####  #   #   ####  #####  #   ###   #   #   ####
// #  ##  #  #        #    #   #  #   #  #        #    #  #   #  ##  #  #
// #  # # #   ###     #    ####   #   #  #        #    #  #   #  # # #   ###
// #  #  ##      #    #    #   #  #   #  #        #    #  #   #  #  ##      #
// #  #   #  ####     #    #   #   ###    ####    #    #   ###   #   #  ####
// (Instructions)

fn ins_ljmp(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Imm(i)) = ins.dst() {
        let (size, isize) = if let Some(Operand::Imm(i)) = ins.src() {
            if i.signed_size() == Size::Dword {
                (Size::Dword, 4)
            } else {
                (Size::Word, 2)
            }
        } else {
            (Size::Word, 2)
        };
        let itob = i.get_raw_le();
        api = api
            .opcode(&[0xEA, itob[1], itob[0]])
            .imm_atindex(1, isize)
            .fixed_size(size)
    } else if let Some(Operand::Mem(_)) = ins.src() {
        api = api.modrm(true, Some(5)).opcode(&[0xFF]).rex()
    }
    api
}
fn ins_lcall(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Imm(i)) = ins.dst() {
        let (size, isize) = if let Some(Operand::Imm(i)) = ins.src() {
            if i.signed_size() == Size::Dword {
                (Size::Dword, 4)
            } else {
                (Size::Word, 2)
            }
        } else {
            (Size::Word, 2)
        };
        let itob = i.get_raw_le();
        api = api
            .opcode(&[0x9A, itob[1], itob[0]])
            .imm_atindex(1, isize)
            .fixed_size(size)
    } else if let Some(Operand::Mem(_)) = ins.src() {
        api = api.modrm(true, Some(3)).opcode(&[0xFF]).rex()
    }
    api
}

fn ins_acmpccxadd(ins: &Instruction, opc: &[u8]) -> GenAPI {
    GenAPI::new()
        .opcode(opc)
        .modrm(true, None)
        .ord(&[MODRM_RM, MODRM_REG, VEX_VVVV])
        .apx(
            APXVariant::VexExtension,
            VexDetails::new()
                .map_select(MAP38)
                .pp(0x66)
                .vex_we(ins.size() == Size::Qword),
            false,
        )
}
fn ins_cmpccxadd(ins: &Instruction, opc: &[u8]) -> GenAPI {
    GenAPI::new()
        .opcode(opc)
        .modrm(true, None)
        .ord(&[MODRM_RM, MODRM_REG, VEX_VVVV])
        .vex(
            VexDetails::new()
                .map_select(0x38)
                .pp(0x66)
                .vex_we(ins.size() == Size::Qword),
        )
}

fn ins_fisttp(ins: &Instruction) -> GenAPI {
    if let Some(Operand::Mem(m)) = ins.dst() {
        if m.size() == Size::Dword {
            GenAPI::new().opcode(&[0xDB]).modrm(true, Some(1))
        } else if m.size() == Size::Qword {
            GenAPI::new().opcode(&[0xDD]).modrm(true, Some(1))
        } else {
            GenAPI::new().opcode(&[0xDF]).modrm(true, Some(1))
        }
    } else {
        GenAPI::new()
    }
}

fn ins_fcomp(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Mem(m)) = ins.dst() {
        api = match m.size() {
            Size::Dword => api.opcode(&[0xD8]).modrm(true, Some(3)),
            _ => api.opcode(&[0xDC]).modrm(true, Some(3)),
        };
    } else {
        api = api.opcode(&[
            0xD8,
            0xD8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]);
    }
    api
}
fn ins_fcom(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Mem(m)) = ins.dst() {
        api = match m.size() {
            Size::Dword => api.opcode(&[0xD8]).modrm(true, Some(2)),
            _ => api.opcode(&[0xDC]).modrm(true, Some(2)),
        };
    } else {
        api = api.opcode(&[
            0xD8,
            0xD0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]);
    }
    api
}

fn ins_fiarthmt(ins: &Instruction, modrm_ovr: u8) -> GenAPI {
    if let Some(Operand::Mem(m)) = ins.dst() {
        if m.size() == Size::Dword {
            GenAPI::new().opcode(&[0xDE]).modrm(true, Some(modrm_ovr))
        } else {
            GenAPI::new().opcode(&[0xDA]).modrm(true, Some(modrm_ovr))
        }
    } else {
        GenAPI::new()
    }
}

// float arithmetical
// sc_op and sc_op1 are basically for FADD: sc_op = C0, sc_op1 = C0
fn ins_farthmt(ins: &Instruction, modrm_ovr: u8, sc_op: u8, sc_op1: u8) -> GenAPI {
    if let Some(Operand::Mem(m)) = ins.dst() {
        if m.size() == Size::Qword {
            GenAPI::new().opcode(&[0xDC]).modrm(true, Some(modrm_ovr))
        } else {
            GenAPI::new().opcode(&[0xD8]).modrm(true, Some(modrm_ovr))
        }
    } else if let Some(Operand::Register(Register::ST0)) = ins.dst() {
        GenAPI::new().opcode(&[
            0xD8,
            sc_op
                + ins
                    .src()
                    .unwrap_or(Operand::Register(Register::ST0))
                    .get_reg()
                    .unwrap()
                    .to_byte(),
        ])
    } else {
        GenAPI::new().opcode(&[
            0xDC,
            sc_op1
                + ins
                    .src()
                    .unwrap_or(Operand::Register(Register::ST0))
                    .get_reg()
                    .unwrap()
                    .to_byte(),
        ])
    }
}

fn ins_fistp(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Mem(m)) = ins.dst() {
        api = match m.size() {
            Size::Dword => api.opcode(&[0xDB]).modrm(true, Some(3)),
            Size::Qword => api.opcode(&[0xDF]).modrm(true, Some(7)),
            _ => api.opcode(&[0xDF]).modrm(true, Some(3)),
        };
    }
    api
}
fn ins_fist(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Mem(m)) = ins.dst() {
        api = match m.size() {
            Size::Dword => api.opcode(&[0xDB]).modrm(true, Some(2)),
            _ => api.opcode(&[0xDF]).modrm(true, Some(2)),
        };
    }
    api
}

fn ins_fild(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Mem(m)) = ins.dst() {
        api = match m.size() {
            Size::Dword => api.opcode(&[0xDB]).modrm(true, Some(0)),
            Size::Qword => api.opcode(&[0xDF]).modrm(true, Some(5)),
            _ => api.opcode(&[0xDF]).modrm(true, Some(0)),
        };
    }
    api
}

fn ins_fstp(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Mem(m)) = ins.dst() {
        api = match m.size() {
            Size::Dword => api.opcode(&[0xD9]).modrm(true, Some(3)),
            Size::Qword => api.opcode(&[0xDD]).modrm(true, Some(3)),
            _ => api.opcode(&[0xDB]).modrm(true, Some(7)),
        };
    } else {
        api = api.opcode(&[
            0xDD,
            0xD8 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]);
    }
    api
}
fn ins_fst(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Mem(m)) = ins.dst() {
        api = match m.size() {
            Size::Dword => api.opcode(&[0xD9]).modrm(true, Some(2)),
            _ => api.opcode(&[0xDD]).modrm(true, Some(2)),
        };
    } else {
        api = api.opcode(&[
            0xDD,
            0xD0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]);
    }
    api
}

fn ins_fld(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new();
    if let Some(Operand::Mem(m)) = ins.dst() {
        api = match m.size() {
            Size::Dword => api.opcode(&[0xD9]).modrm(true, Some(0)),
            Size::Qword => api.opcode(&[0xDD]).modrm(true, Some(0)),
            _ => api.opcode(&[0xDB]).modrm(true, Some(5)),
        };
    } else {
        api = api.opcode(&[
            0xD9,
            0xC0 + (ins.dst().unwrap().get_reg().unwrap().to_byte()),
        ]);
    }
    api
}

fn ins_cfcmov(ins: &Instruction, opc: &[u8]) -> GenAPI {
    if ins.len() == 3 {
        GenAPI::new()
            .opcode(opc)
            .modrm(true, None)
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            )
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
    } else if let Some(Operand::Register(_)) = ins.dst() {
        GenAPI::new()
            .opcode(opc)
            .modrm(true, None)
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            )
            .ord(&[MODRM_REG, MODRM_RM])
    } else {
        GenAPI::new()
            .opcode(opc)
            .modrm(true, None)
            .apx(
                APXVariant::LegacyExtension,
                VexDetails::new().map_select(MAP4),
                false,
            )
            .ord(&[MODRM_RM, MODRM_REG])
    }
}

fn ins_ccmp(
    ins: &Instruction,
    opc_mr: &[u8],
    opc_rm: &[u8],
    opc_im: &[u8],
    opc_im8: &[u8],
    modrm_ovr: u8,
) -> GenAPI {
    let dst = ins.dst().unwrap();
    let src = ins.src().unwrap();
    match (&dst, src) {
        (Operand::Mem(_) | Operand::Register(_), Operand::Imm(i)) => match i.signed_size() {
            Size::Byte => GenAPI::new()
                .opcode(&[opc_im8[0] - (dst.size() == Size::Byte) as u8])
                .imm_atindex(1, 1)
                .modrm(true, Some(modrm_ovr))
                .apx(APXVariant::CondTestCmpExtension, VexDetails::new(), false)
                .ord(&[MODRM_RM, MODRM_REG]),
            Size::Word => GenAPI::new()
                .opcode(opc_im)
                .imm_atindex(1, 2)
                .modrm(true, Some(modrm_ovr))
                .apx(
                    APXVariant::CondTestCmpExtension,
                    VexDetails::new().pp(0x66),
                    false,
                )
                .ord(&[MODRM_RM, MODRM_REG]),
            _ => GenAPI::new()
                .opcode(opc_im)
                .imm_atindex(1, 4)
                .modrm(true, Some(modrm_ovr))
                .apx(APXVariant::CondTestCmpExtension, VexDetails::new(), false)
                .ord(&[MODRM_RM, MODRM_REG]),
        },
        (Operand::Register(d), Operand::Register(_)) => {
            if d.size() == Size::Byte {
                GenAPI::new()
                    .opcode(&[opc_rm[0] - 1])
                    .modrm(true, None)
                    .ord(&[MODRM_RM, MODRM_REG])
                    .apx(APXVariant::CondTestCmpExtension, VexDetails::new(), false)
            } else {
                GenAPI::new()
                    .opcode(opc_rm)
                    .modrm(true, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .apx(APXVariant::CondTestCmpExtension, VexDetails::new(), false)
            }
        }
        (Operand::Mem(_), Operand::Register(d)) => {
            if d.size() == Size::Byte {
                GenAPI::new()
                    .opcode(&[opc_mr[0] - 1])
                    .modrm(true, None)
                    .ord(&[MODRM_RM, MODRM_REG])
                    .apx(APXVariant::CondTestCmpExtension, VexDetails::new(), false)
            } else {
                GenAPI::new()
                    .opcode(opc_mr)
                    .opcode(&[opc_mr[0] - 1])
                    .modrm(true, None)
                    .ord(&[MODRM_RM, MODRM_REG])
                    .apx(APXVariant::CondTestCmpExtension, VexDetails::new(), false)
            }
        }
        (Operand::Register(d), Operand::Mem(_)) => {
            if d.size() == Size::Byte {
                GenAPI::new()
                    .opcode(&[opc_rm[0] - 1])
                    .modrm(true, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .apx(APXVariant::CondTestCmpExtension, VexDetails::new(), false)
            } else {
                GenAPI::new()
                    .opcode(opc_rm)
                    .modrm(true, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .apx(APXVariant::CondTestCmpExtension, VexDetails::new(), false)
            }
        }
        _ => panic!("todo"),
    }
}

// opc[0] = r/m8, 1
// opc[1] = r/m8, cl
// opc[2] = r/m8, imm8
// opc[3] = r/m16/32/64, 1
// opc[4] = r/m16/32/64, cl
// opc[5] = r/m16/32/64, imm8
#[inline(always)]
fn ins_ashllike(ins: &Instruction, opc: &[u8; 6], ovr: u8) -> GenAPI {
    let mut api = GenAPI::new().modrm(true, Some(ovr));
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();
    let opcd = match src {
        Operand::Register(Register::CL) => match dst.size() {
            Size::Byte => opc[1],
            Size::Word | Size::Dword | Size::Qword => opc[4],
            _ => panic!("CL failure"),
        },
        Operand::Register(_) | Operand::Mem(_) => {
            api = api.ord(&[VEX_VVVV, MODRM_RM]);
            match ins.ssrc().unwrap() {
                Operand::Register(Register::CL) => match dst.size() {
                    Size::Byte => opc[1],
                    Size::Word | Size::Dword | Size::Qword => opc[4],
                    _ => panic!("CL failure"),
                },
                Operand::Imm(imm) => {
                    if imm == Number::uint64(1) {
                        match dst.size() {
                            Size::Byte => opc[0],
                            _ => opc[3],
                        }
                    } else {
                        api = api.imm_atindex(2, 1);
                        match dst.size() {
                            Size::Byte => opc[2],
                            _ => opc[5],
                        }
                    }
                }
                _ => panic!(),
            }
        }
        Operand::Imm(imm) => {
            if imm == Number::uint64(1) {
                match dst.size() {
                    Size::Byte => opc[0],
                    _ => opc[3],
                }
            } else {
                api = api.imm_atindex(1, 1);
                match dst.size() {
                    Size::Byte => opc[2],
                    _ => opc[5],
                }
            }
        }
        _ => panic!("Other {:?}", src),
    };
    api = api.opcode(&[opcd]);
    api
}

fn ins_aimul(ins: &Instruction) -> GenAPI {
    if ins.len() == 3 {
        GenAPI::new()
            .opcode(&[0xAF])
            .modrm(true, None)
            .ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
    } else if ins.len() == 2 {
        GenAPI::new()
            .opcode(&[0xAF])
            .modrm(true, None)
            .ord(&[MODRM_REG, MODRM_RM])
    } else {
        GenAPI::new()
            .opcode(&[0xF7 - (ins.size() == Size::Byte) as u8])
            .modrm(true, Some(5))
    }
}

fn ins_aadox(ins: &Instruction, opc: &[u8]) -> GenAPI {
    let mut api = GenAPI::new().opcode(opc).modrm(true, None);
    api = if ins.ssrc().is_none() {
        api.ord(&[MODRM_REG, MODRM_RM])
    } else {
        api.ord(&[VEX_VVVV, MODRM_REG, MODRM_RM])
    };
    api
}

// yeah, i also do not have idea what this means
//  but since it works for now, i'll leave it be :D
#[allow(clippy::too_many_arguments)]
fn ins_aadd(
    ins: &Instruction,
    opc_rm8: &[u8],
    opc_rm: &[u8],
    opc_rmi: &[u8],
    opc_rmi8: &[u8],
    opc_rm8i8: &[u8],
    opc_r8m8: &[u8],
    opc_rm64: &[u8],
    modrm_ovr: u8,
) -> GenAPI {
    let (dst, src, ssrc) = (ins.dst().unwrap(), ins.src().unwrap(), ins.ssrc());
    match (&dst, src, ssrc) {
        (Operand::Mem(_) | Operand::Register(_), Operand::Imm(i), None) => {
            if dst.size() != Size::Byte {
                let (opc, isz) = match i.signed_size() {
                    Size::Byte => (opc_rmi8, 1),
                    Size::Word => (opc_rmi, 2),
                    _ => (opc_rmi, 4),
                };
                GenAPI::new()
                    .opcode(opc)
                    .modrm(true, Some(modrm_ovr))
                    .ord(&[MODRM_RM, MODRM_REG])
                    .imm_atindex(1, isz)
            } else {
                GenAPI::new()
                    .opcode(opc_rm8i8)
                    .modrm(true, Some(modrm_ovr))
                    .ord(&[MODRM_RM, MODRM_REG])
                    .imm_atindex(1, 1)
            }
        }
        (Operand::Mem(_) | Operand::Symbol(_), Operand::Register(_), ssrc) => {
            if dst.size() == Size::Byte {
                if let Some(ssrc) = ssrc {
                    if let Operand::Imm(_) = ssrc {
                        GenAPI::new()
                            .opcode(opc_rm8i8)
                            .modrm(true, Some(modrm_ovr))
                            .ord(&[VEX_VVVV, MODRM_RM])
                            .imm_atindex(2, 1)
                    } else {
                        panic!("Invalid variant")
                    }
                } else {
                    GenAPI::new()
                        .opcode(opc_rm8)
                        .modrm(true, None)
                        .ord(&[MODRM_RM, MODRM_REG])
                }
            } else if let Some(ssrc) = ssrc {
                if let Operand::Imm(s) = ssrc {
                    let (isz, opc) = match s.signed_size() {
                        Size::Word => (2, opc_rmi),
                        Size::Byte => (1, opc_rmi8),
                        _ => (4, opc_rmi),
                    };
                    GenAPI::new()
                        .opcode(opc)
                        .modrm(true, Some(modrm_ovr))
                        .ord(&[VEX_VVVV, MODRM_RM])
                        .imm_atindex(2, isz)
                } else {
                    panic!("Invalid variant")
                }
            } else {
                GenAPI::new()
                    .opcode(opc_rm)
                    .modrm(true, None)
                    .ord(&[MODRM_RM, MODRM_REG])
            }
        }
        (Operand::Register(dstr), Operand::Mem(_) | Operand::Register(_), ssrc) => {
            if dstr.size() == Size::Byte {
                if let Some(Operand::Register(_)) = ssrc {
                    GenAPI::new()
                        .opcode(opc_r8m8)
                        .modrm(true, None)
                        .ord(&[VEX_VVVV, MODRM_RM, MODRM_REG])
                } else if let Some(Operand::Imm(_)) = ssrc {
                    GenAPI::new()
                        .opcode(opc_rm8i8)
                        .modrm(true, Some(modrm_ovr))
                        .ord(&[VEX_VVVV, MODRM_RM])
                        .imm_atindex(2, 1)
                } else {
                    GenAPI::new()
                        .opcode(opc_rm8)
                        .modrm(true, None)
                        .ord(&[MODRM_REG, MODRM_RM])
                }
            } else if let Some(Operand::Register(_)) = ssrc {
                GenAPI::new()
                    .opcode(opc_rm64)
                    .modrm(true, None)
                    .ord(&[VEX_VVVV, MODRM_RM, MODRM_REG])
            } else if let Some(Operand::Imm(s)) = ssrc {
                let isz = match s.signed_size() {
                    Size::Byte | Size::Word => 2,
                    _ => 4,
                };
                GenAPI::new()
                    .opcode(opc_rmi)
                    .modrm(true, Some(modrm_ovr))
                    .ord(&[VEX_VVVV, MODRM_RM])
                    .imm_atindex(2, isz)
            } else {
                GenAPI::new()
                    .opcode(opc_rm)
                    .modrm(true, None)
                    .ord(&[MODRM_REG, MODRM_RM])
            }
        }
        _ => panic!("todo:"),
    }
}

fn ins_kmov(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new().modrm(true, None).ord(&[MODRM_REG, MODRM_RM]);
    let dst = ins.dst().unwrap();
    let src = ins.src().unwrap();
    if let Operand::Register(r) = dst {
        let purp = r.purpose();
        if purp.is_mask() {
            if let Operand::Mem(_) = src {
                api = api.opcode(&[0x90]);
            } else if let Operand::Register(r) = src {
                let purp = r.purpose();
                if purp.is_mask() {
                    api = api.opcode(&[0x90]);
                } else {
                    api = api.opcode(&[0x92]);
                }
            }
        } else {
            api = api.opcode(&[0x93]);
        }
    } else if let Operand::Mem(_) | Operand::Symbol(_) = dst {
        api = api.opcode(&[0x91]).ord(&[MODRM_RM, MODRM_REG]);
    }
    api
}

fn ins_xchg(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new().rex();
    match ins.size() {
        Size::Byte => {
            api = api.opcode(&[0x86]);
            if let Some(Operand::Register(_)) = ins.dst() {
                api = api.ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            } else {
                api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV])
            }
            api = api.modrm(true, None);
        }
        Size::Word => {
            if let Some(Operand::Register(r)) = ins.dst() {
                if r == Register::AX {
                    let s = if let Some(Operand::Register(r1)) = ins.src() {
                        r1.to_byte()
                    } else {
                        0
                    };
                    api = api.opcode(&[(0x90 + s)]);
                } else {
                    api = api.opcode(&[0x87]);
                    api = api.modrm(true, None);
                    api = api.ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                }
            } else {
                api = api.opcode(&[0x87]);
                api = api.modrm(true, None);
                api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV]);
            }
        }
        Size::Dword | Size::Qword => {
            if let Some(Operand::Register(r)) = ins.dst() {
                if r == Register::EAX || r == Register::RAX {
                    let s = if let Some(Operand::Register(r1)) = ins.src() {
                        r1.to_byte()
                    } else {
                        0
                    };
                    api = api.opcode(&[(0x90 + s)]);
                } else if let Some(Operand::Register(Register::EAX | Register::RAX)) = ins.src() {
                    api = api.opcode(&[(0x90 + r.to_byte())]);
                } else {
                    api = api.opcode(&[0x87]);
                    api = api.modrm(true, None);
                    api = api.ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                }
            } else if let Some(Operand::Register(_)) = ins.src() {
                api = api.opcode(&[0x87]);
                api = api.modrm(true, None);
                api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV]);
            } else {
                api = api.opcode(&[0x87]);
                api = api.modrm(true, None);
                api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV]);
            }
        }
        _ => invalid(3977),
    }
    api
}

#[inline(always)]
fn ins_xbegin(_: &Instruction) -> GenAPI {
    GenAPI::new().opcode(&[0xC7, 0xF8]).imm_atindex(0, 4)
}

#[inline(always)]
fn ins_ashlx(ins: &Instruction, opc_imm: &[u8], opc_rm: &[u8]) -> GenAPI {
    let mut api = GenAPI::new().rex().modrm(true, None);
    if let Some(Operand::Imm(_) | Operand::Symbol(_)) = ins.ssrc() {
        api = api.opcode(opc_imm).imm_atindex(2, 1);
    } else if let Some(Operand::Imm(_) | Operand::Symbol(_)) = ins.tsrc() {
        api = api
            .opcode(opc_imm)
            .imm_atindex(3, 1)
            .ord(&[VEX_VVVV, MODRM_RM, MODRM_REG]);
    } else {
        if ins.len() == 4 {
            api = api.ord(&[VEX_VVVV, MODRM_RM, MODRM_REG])
        }
        api = api.opcode(opc_rm);
    }
    api
}

#[inline(always)]
fn ins_shlx(ins: &Instruction, opc_imm: &[u8], opc_rm: &[u8]) -> GenAPI {
    let mut api = GenAPI::new().rex().modrm(true, None);
    if let Some(Operand::Imm(_) | Operand::Symbol(_)) = ins.ssrc() {
        api = api.opcode(opc_imm).imm_atindex(2, 1);
    } else {
        api = api.opcode(opc_rm);
    }
    api
}

#[inline(always)]
fn ins_bt(ins: &Instruction, opc_noimm: &[u8], opc_imm: &[u8], _: u8, modrm: u8) -> GenAPI {
    let mut api = GenAPI::new().rex();
    if let Some(Operand::Imm(_) | Operand::Symbol(_)) = ins.src() {
        api = api
            .opcode(opc_imm)
            .modrm(true, Some(modrm))
            .imm_atindex(1, 1);
    } else {
        api = api.opcode(opc_noimm).modrm(true, None)
    };
    api
}

#[inline(always)]
fn ins_cmovcc(_: &Instruction, opc: &[u8], _: u8) -> GenAPI {
    GenAPI::new()
        .opcode(opc)
        .modrm(true, None)
        .rex()
        .ord(&[MODRM_REG, MODRM_RM])
}

#[inline(always)]
fn ins_pop(ins: &Instruction, _: u8) -> GenAPI {
    match ins.dst().unwrap() {
        Operand::Register(r) => {
            if r.is_sgmnt() {
                match r {
                    Register::DS => GenAPI::new().opcode(&[0x1F]),
                    Register::ES => GenAPI::new().opcode(&[0x07]),
                    Register::SS => GenAPI::new().opcode(&[0x17]),
                    Register::FS => GenAPI::new().opcode(&[0x0F, 0xA1]),
                    Register::GS => GenAPI::new().opcode(&[0x0F, 0xA9]),
                    Register::CS => GenAPI::new().opcode(&[0x90]),
                    _ => invalid(34),
                }
            } else {
                GenAPI::new().opcode(&[0x58 + r.to_byte()]).rex()
            }
        }
        Operand::Mem(_) | Operand::Symbol(_) => {
            GenAPI::new().opcode(&[0x8F]).rex().modrm(true, None)
        }
        _ => invalid(33),
    }
}

#[inline(always)]
fn ins_push(ins: &Instruction, _: u8) -> GenAPI {
    match ins.dst().unwrap() {
        Operand::Register(r) => {
            if r.is_sgmnt() {
                match r {
                    Register::CS => GenAPI::new().opcode(&[0x0E]),
                    Register::SS => GenAPI::new().opcode(&[0x16]),
                    Register::DS => GenAPI::new().opcode(&[0x1E]),
                    Register::ES => GenAPI::new().opcode(&[0x06]),
                    Register::FS => GenAPI::new().opcode(&[0x0F, 0xA0]),
                    Register::GS => GenAPI::new().opcode(&[0x0F, 0xA8]),
                    _ => invalid(32),
                }
            } else {
                GenAPI::new().opcode(&[0x50 + r.to_byte()]).rex()
            }
        }
        Operand::String(_) => match ins.dst().unwrap().size() {
            Size::Byte => GenAPI::new().opcode(&[0x6A]).imm_atindex(0, 1),
            Size::Word => GenAPI::new()
                .opcode(&[0x68])
                .imm_atindex(0, 2)
                .fixed_size(Size::Word),
            Size::Dword => GenAPI::new()
                .opcode(&[0x68])
                .imm_atindex(0, 4)
                .fixed_size(Size::Dword),
            _ => invalid(31),
        },
        Operand::Imm(nb) => match nb.signed_size() {
            Size::Byte => GenAPI::new().opcode(&[0x6A]).imm_atindex(0, 1),
            Size::Word => GenAPI::new()
                .opcode(&[0x68])
                .imm_atindex(0, 2)
                .fixed_size(Size::Word),
            Size::Dword => GenAPI::new()
                .opcode(&[0x68])
                .imm_atindex(0, 4)
                .fixed_size(Size::Dword),
            _ => invalid(31),
        },
        Operand::Symbol(s) => {
            if s.is_deref() {
                GenAPI::new().opcode(&[0xFF]).modrm(true, Some(6)).rex()
            } else {
                GenAPI::new().opcode(&[0x68]).imm_atindex(0, 4)
            }
        }
        Operand::Mem(_) => GenAPI::new().opcode(&[0xFF]).modrm(true, Some(6)).rex(),
    }
}

#[inline(always)]
fn ins_mov(ins: &Instruction, _: u8) -> GenAPI {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();
    if let Operand::Register(r) = dst {
        let p = r.purpose();
        if p.is_dbg() {
            GenAPI::new()
                .opcode(&[0x0F, 0x23])
                .modrm(true, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                .rex()
        } else if p.is_sgmnt() {
            match src {
                Operand::Register(_) | Operand::Mem(_) => {
                    GenAPI::new().opcode(&[0x8E]).modrm(true, None).rex()
                }
                _ => invalid(25),
            }
        } else if p.is_ctrl() {
            GenAPI::new()
                .opcode(&[0x0F, 0x22])
                .modrm(true, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                .rex()
        } else {
            match src {
                Operand::Imm(i) => {
                    if i.signed_size() == Size::Qword {
                        GenAPI::new()
                            .opcode(&[0xB8 + r.to_byte()])
                            .rex()
                            .imm_atindex(1, 8)
                    } else if r.ebits()[1] && r.size() == Size::Qword {
                        GenAPI::new()
                            .opcode(&[0xC7])
                            .rex()
                            .modrm(true, Some(0))
                            .imm_atindex(1, 4)
                    } else {
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
                            .rex()
                    }
                }
                Operand::Register(r) => {
                    let p = r.purpose();
                    if p.is_sgmnt() {
                        GenAPI::new().opcode(&[0x8C]).modrm(true, None).rex()
                    } else if p.is_ctrl() {
                        GenAPI::new().opcode(&[0x0F, 0x20]).modrm(true, None).rex()
                    } else if p.is_dbg() {
                        GenAPI::new()
                            .opcode(&[0x0F, 0x21])
                            .modrm(true, None)
                            .ord(&[MODRM_RM, MODRM_REG])
                            .rex()
                    } else {
                        let opc = if let Operand::Register(_) = src {
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
                        GenAPI::new().opcode(&[opc]).modrm(true, None).rex()
                    }
                }
                Operand::Symbol(s) => {
                    if s.is_deref() {
                        let opc = if let Operand::Register(_) = src {
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
                            .modrm(true, None)
                            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                            .rex()
                    } else if s.reltype().unwrap_or(RelType::REL32).size() == 8 {
                        GenAPI::new()
                            .opcode(&[0xB8 + r.to_byte()])
                            .rex()
                            .imm_atindex(1, 8)
                    } else if r.ebits()[1] && r.size() == Size::Qword {
                        GenAPI::new()
                            .opcode(&[0xC7])
                            .rex()
                            .imm_atindex(1, 4)
                            .modrm(true, Some(0))
                    } else {
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
                            .rex()
                    }
                }
                Operand::Mem(_) => {
                    let opc = if let Operand::Register(_) = src {
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
                        .modrm(true, None)
                        .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                        .rex()
                }
                _ => invalid(26),
            }
        }
    } else if let Operand::Mem(_) | Operand::Symbol(_) = dst {
        match src {
            Operand::Register(_) => {
                let opc = match dst.size() {
                    Size::Byte => 0x88,
                    Size::Word | Size::Dword | Size::Qword => 0x89,
                    _ => invalid(24),
                };
                GenAPI::new().opcode(&[opc]).modrm(true, None).rex()
            }
            Operand::Imm(_) | Operand::Symbol(_) => {
                let size = dst.size();
                let opc = match size {
                    Size::Byte => 0xC6,
                    Size::Word | Size::Dword | Size::Qword => 0xC7,
                    _ => invalid(23),
                };
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, Some(0))
                    .rex()
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
#[inline(always)]
fn add_like_ins(ins: &Instruction, opc: &[u8; 9], ovrreg: u8, bits: u8) -> GenAPI {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, src) {
        (Operand::Register(dstr), Operand::Symbol(s)) => {
            if s.is_deref() {
                let opc = match dstr.size() {
                    Size::Byte => opc[7],
                    Size::Word | Size::Dword | Size::Qword => opc[6],
                    _ => invalid(17),
                };
                GenAPI::new().opcode(&[opc]).modrm(true, None).rex()
            } else {
                let srci = ins.src().unwrap().size();
                if let Size::Dword | Size::Word = srci {
                    if let Register::RAX | Register::EAX = dstr {
                        return GenAPI::new().opcode(&[opc[1]]).imm_atindex(1, 4).rex();
                    } else if let Register::AX = dstr {
                        return GenAPI::new().opcode(&[opc[1]]).imm_atindex(1, 2).rex();
                    }
                }
                if let Register::AL = dstr {
                    return GenAPI::new().opcode(&[opc[0]]).imm_atindex(1, 1).rex();
                } else if let Register::AX = dstr {
                    return GenAPI::new().opcode(&[opc[1]]).imm_atindex(1, 2).rex();
                }

                let (opc, isz) = match srci {
                    Size::Byte => (opc[2], 1),
                    Size::Dword | Size::Qword | Size::Word => {
                        if srci == Size::Byte {
                            (opc[4], 1)
                        } else {
                            (opc[3], 4)
                        }
                    }
                    _ => invalid(20),
                };
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, Some(ovrreg))
                    .rex()
                    .imm_atindex(1, isz)
            }
        }
        (Operand::Register(dstr), Operand::Imm(_)) => {
            let srci = ins.src().unwrap().size();
            if let Size::Dword | Size::Word = srci {
                if let Register::RAX | Register::EAX = dstr {
                    return GenAPI::new().opcode(&[opc[1]]).imm_atindex(1, 4).rex();
                } else if let Register::AX = dstr {
                    return GenAPI::new().opcode(&[opc[1]]).imm_atindex(1, 2).rex();
                }
            }
            if let Register::AL = dstr {
                return GenAPI::new().opcode(&[opc[0]]).imm_atindex(1, 1).rex();
            } else if let Register::AX = dstr {
                return GenAPI::new().opcode(&[opc[1]]).imm_atindex(1, 2).rex();
            }

            let (opc, isz) = match srci {
                Size::Byte => (opc[4], 1),
                Size::Word => (
                    opc[3],
                    if bits == 16 && dstr.size() == Size::Word {
                        2
                    } else {
                        4
                    },
                ),
                _ => (opc[3], 4),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(ovrreg))
                .rex()
                .imm_atindex(1, isz)
        }
        (Operand::Mem(_) | Operand::Symbol(_), Operand::Imm(_) | Operand::Symbol(_)) => {
            let dstm = ins.dst().unwrap().size();
            let srci = ins.src().unwrap().size();
            let opc = match dstm {
                Size::Byte => opc[2],
                Size::Word => opc[3],
                Size::Dword => opc[3],
                Size::Qword => {
                    if srci == Size::Byte {
                        opc[4]
                    } else {
                        opc[3]
                    }
                }
                _ => invalid(18),
            };
            let size = if (Size::Word, 16) == (srci, bits) {
                2
            } else if srci != Size::Byte {
                4
            } else {
                1
            };

            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(ovrreg))
                .rex()
                .imm_atindex(1, size)
        }
        (Operand::Register(r), Operand::Mem(_) | Operand::Register(_)) => {
            let opc = match r.size() {
                Size::Byte => opc[7],
                Size::Word | Size::Dword | Size::Qword => opc[6],
                _ => invalid(17),
            };
            GenAPI::new().opcode(&[opc]).modrm(true, None).rex()
        }
        (Operand::Mem(_) | Operand::Symbol(_), Operand::Register(_)) => {
            let opc = match ins.dst().unwrap().size() {
                Size::Byte => opc[7],
                Size::Word | Size::Dword | Size::Qword => opc[6],
                _ => invalid(15),
            };
            GenAPI::new().opcode(&[opc]).modrm(true, None).rex()
        }
        _ => invalid(14),
    }
}

#[inline(always)]
fn ins_cmp(ins: &Instruction, _: u8) -> GenAPI {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (dst, &src) {
        (Operand::Register(dstr), Operand::Imm(_) | Operand::Symbol(_)) => {
            let srci = ins.src().unwrap().size();
            if let Size::Dword | Size::Word = srci {
                if let Register::RAX | Register::EAX = dstr {
                    return GenAPI::new().opcode(&[0x3D]).imm_atindex(1, 4).rex();
                } else if let Register::AX = dstr {
                    return GenAPI::new().opcode(&[0x3D]).imm_atindex(1, 2).rex();
                }
            }
            if let Register::AL = dstr {
                return GenAPI::new().opcode(&[0x3C]).imm_atindex(1, 1).rex();
            } else if let Register::AX = dstr {
                return GenAPI::new().opcode(&[0x3D]).imm_atindex(1, 2).rex();
            }

            let (opc, isz) = match dstr.size() {
                Size::Byte => (0x80, 1),
                Size::Dword | Size::Qword | Size::Word => {
                    if srci == Size::Byte {
                        (0x83, 1)
                    } else {
                        (0x81, 4)
                    }
                }
                _ => invalid(13),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(7))
                .rex()
                .imm_atindex(1, isz)
        }
        (Operand::Mem(_) | Operand::Symbol(_), Operand::Imm(_) | Operand::Symbol(_)) => {
            let dstm = ins.dst().unwrap().size();
            let srci = ins.src().unwrap().size();
            let opc = match dstm {
                Size::Byte => 0x80,
                Size::Qword | Size::Word | Size::Dword => {
                    if let Operand::Imm(n) = src {
                        if n.get_as_i32() < 128 && n.get_as_i32() > -128 {
                            0x83
                        } else {
                            0x81
                        }
                    } else if srci == Size::Byte {
                        0x83
                    } else {
                        0x81
                    }
                }
                _ => invalid(11),
            };
            let size = if let (Size::Word | Size::Byte, Size::Word) = (srci, dstm) {
                2
            } else if let (Size::Byte, Size::Dword | Size::Qword) = (srci, dstm) {
                4
            } else if srci != Size::Byte {
                4
            } else {
                1
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(7))
                .rex()
                .imm_atindex(1, size)
        }
        (Operand::Register(r), Operand::Mem(_) | Operand::Register(_)) => {
            let opc = match r.size() {
                Size::Byte => 0x3A,
                Size::Word | Size::Dword | Size::Qword => 0x3B,
                _ => invalid(10),
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex()
        }
        (Operand::Mem(m), Operand::Register(_)) => {
            let opc = match m.size() {
                Size::Byte => 0x38,
                Size::Word | Size::Dword | Size::Qword => 0x39,
                _ => invalid(9),
            };
            GenAPI::new().opcode(&[opc]).modrm(true, None).rex()
        }
        _ => invalid(7),
    }
}

#[inline(always)]
fn ins_test(ins: &Instruction, _: u8) -> GenAPI {
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();

    match (&dst, src) {
        (Operand::Register(dstr), Operand::Imm(_) | Operand::Symbol(_)) => {
            let sz = ins.src().unwrap().size();
            if let Size::Dword | Size::Word = sz {
                if let &Register::RAX | &Register::EAX = dstr {
                    return GenAPI::new().opcode(&[0xA9]).imm_atindex(1, 4).rex();
                } else if let &Register::AX = dstr {
                    return GenAPI::new().opcode(&[0xA9]).imm_atindex(1, 2).rex();
                }
            }
            if let &Register::AL = dstr {
                return GenAPI::new().opcode(&[0xA8]).imm_atindex(1, 1).rex();
            } else if let &Register::AX = dstr {
                return GenAPI::new().opcode(&[0xA9]).imm_atindex(1, 2).rex();
            }

            let (opc, isz, fx) = match (dst.size(), sz) {
                (Size::Byte, _) => (0xF6, 1, None),
                (Size::Word, _) => (0xF7, 2, Some(Size::Word)),
                _ => (0xF7, 4, None),
            };
            if let Some(fx) = fx {
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, Some(0))
                    .fixed_size(fx)
                    .imm_atindex(1, isz)
                    .rex()
            } else {
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, Some(0))
                    .imm_atindex(1, isz)
                    .rex()
            }
        }
        (Operand::Mem(_) | Operand::Symbol(_), Operand::Imm(_) | Operand::Symbol(_)) => {
            let dsts = ins.dst().unwrap().size();
            let srci = ins.src().unwrap().size();
            let opc = match dsts {
                Size::Byte => 0xF6,
                Size::Qword | Size::Word | Size::Dword => 0xF7,
                _ => invalid(4),
            };
            let size = if let (Size::Word | Size::Byte, Size::Word) = (srci, dsts) {
                2
            } else if let (Size::Byte, Size::Dword | Size::Qword) = (srci, dsts) {
                4
            } else if srci != Size::Byte {
                4
            } else {
                1
            };
            GenAPI::new()
                .opcode(&[opc])
                .modrm(true, Some(0))
                .rex()
                .imm_atindex(1, size)
        }
        (Operand::Register(_) | Operand::Mem(_) | Operand::Symbol(_), Operand::Register(_)) => {
            let opc = match dst.size() {
                Size::Byte => 0x84,
                Size::Word | Size::Dword | Size::Qword => 0x85,
                _ => invalid(3),
            };
            GenAPI::new().opcode(&[opc]).modrm(true, None).rex()
        }
        _ => invalid(2),
    }
}

#[inline(always)]
fn ins_imul(ins: &Instruction, _: u8) -> GenAPI {
    match ins.src() {
        None => {
            let opc = match ins.dst().unwrap().size() {
                Size::Byte => &[0xF6],
                _ => &[0xF7],
            };
            GenAPI::new().opcode(opc).modrm(true, Some(5)).rex()
        }
        Some(_) => match ins.get(2) {
            Some(Operand::Imm(_)) => {
                let (opc, size) = match ins.get(2).unwrap().size() {
                    Size::Byte => (0x6B, 1),
                    Size::Word => (0x69, 2),
                    _ => (0x69, 4),
                };
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .rex()
                    .imm_atindex(2, size)
            }
            _ => GenAPI::new().opcode(&[0x0F, 0xAF]).modrm(true, None).rex(),
        },
    }
}

// opc[0] = r/m8, 1
// opc[1] = r/m8, cl
// opc[2] = r/m8, imm8
// opc[3] = r/m16/32/64, 1
// opc[4] = r/m16/32/64, cl
// opc[5] = r/m16/32/64, imm8
#[inline(always)]
fn ins_shllike(ins: &Instruction, opc: &[u8; 6], ovr: u8, _: u8) -> GenAPI {
    let mut api = GenAPI::new().modrm(true, Some(ovr)).rex();
    let src = ins.src().unwrap();
    let dst = ins.dst().unwrap();
    let (opcd, _) = match src {
        Operand::Register(Register::CL) => match dst.size() {
            Size::Byte => (opc[1], None),
            Size::Word | Size::Dword | Size::Qword => (opc[4], None),
            _ => panic!("CL failure"),
        },
        Operand::Imm(imm) => {
            if imm == Number::uint64(1) {
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

#[inline(always)]
fn ins_inclike(ins: &Instruction, opc: &[u8; 2], ovr: u8, _: u8) -> GenAPI {
    let opcd = match ins.dst().unwrap().size() {
        Size::Byte => opc[0],
        _ => opc[1],
    };
    GenAPI::new().opcode(&[opcd]).modrm(true, Some(ovr)).rex()
}

#[inline(always)]
fn ins_lea(_: &Instruction, _: u8) -> GenAPI {
    GenAPI::new()
        .opcode(&[0x8D])
        .modrm(true, None)
        .ord(&[MODRM_REG, MODRM_RM])
}

// opc[0] = rel16/32
// opc[1] = r/m
// opc[2] = rel8
#[inline(always)]
fn ins_jmplike<'a>(ins: &'a Instruction, opc: [&'a [u8]; 3], addt: u8, _: u8) -> GenAPI {
    match ins.dst().unwrap() {
        Operand::Imm(i) => {
            let opc = match i.signed_size() {
                Size::Byte => opc[2],
                _ => opc[0],
            };
            GenAPI::new().opcode(opc).imm_atindex(0, 0)
        }
        Operand::Symbol(s) => {
            let sz = s.reltype().unwrap_or(RelType::REL32).size();
            let base = match sz {
                1 => opc[2].to_vec(),
                _ => opc[0].to_vec(),
            };
            GenAPI::new().opcode(&base).imm_atindex(0, 0)
        }
        Operand::Register(_) | Operand::Mem(_) => {
            GenAPI::new().opcode(opc[1]).modrm(true, Some(addt)).rex()
        }
        _ => invalid(0),
    }
}

#[inline(always)]
fn ins_divmul(ins: &Instruction, ovr: u8, _: u8) -> GenAPI {
    let opc = match ins.dst().unwrap().size() {
        Size::Byte => [0xF6],
        _ => [0xF7],
    };
    GenAPI::new().opcode(&opc).modrm(true, Some(ovr))
}

#[inline(always)]
fn ins_in(ins: &Instruction, _: u8) -> GenAPI {
    if let Operand::Register(_) = ins.src().unwrap() {
        let sz = ins.dst().unwrap().size();
        if sz == Size::Byte {
            GenAPI::new().opcode(&[0xEC]).fixed_size(Size::Byte)
        } else {
            GenAPI::new().opcode(&[0xED]).fixed_size(sz)
        }
    } else if ins.size() == Size::Byte {
        GenAPI::new().opcode(&[0xE4]).imm_atindex(1, 1)
    } else {
        GenAPI::new().opcode(&[0xE5]).imm_atindex(1, 1)
    }
}

#[inline(always)]
fn ins_out(ins: &Instruction, _: u8) -> GenAPI {
    let sz = ins.src().unwrap().size();
    if let Operand::Register(_) = ins.dst().unwrap() {
        if sz == Size::Byte {
            GenAPI::new()
                .opcode(&[0xEE])
                .fixed_size(Size::Byte)
                .can_h66(false)
        } else {
            GenAPI::new().opcode(&[0xEF]).fixed_size(sz)
        }
    } else if sz == Size::Byte {
        GenAPI::new().opcode(&[0xE6]).imm_atindex(0, 1)
    } else {
        GenAPI::new()
            .opcode(&[0xE7])
            .imm_atindex(0, 1)
            .fixed_size(sz)
    }
}

#[inline(always)]
fn ins_shrtjmp(_: &Instruction, opc: Vec<u8>) -> GenAPI {
    GenAPI::new().opcode(&opc).imm_atindex(0, 1)
}

// ==============================
// Utils

fn invalid(ctx: i32) -> ! {
    panic!("Unexpected thing that should not happen - code {ctx}")
}
