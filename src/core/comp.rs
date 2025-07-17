// pasm - src/core/comp.rs
// -----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    core::api::*,
    core::evex::*,
    shr::{
        ast::{IVariant, Instruction, Operand},
        ins::Mnemonic,
        num::Number,
        reg::{Purpose as RPurpose, Register},
        reloc::RelType,
        reloc::Relocation,
        size::Size,
        symbol::{Symbol, SymbolType},
        visibility::Visibility,
    },
};

use OpOrd::*;

#[inline]
pub fn extern_trf<'a>(externs: &'a Vec<&'a str>) -> Vec<Symbol<'a>> {
    let mut symbols = Vec::with_capacity(externs.len());
    for extern_ in externs {
        symbols.push(Symbol {
            name: extern_,
            offset: 0,
            size: 0,
            sindex: 0,
            stype: SymbolType::NoType,
            visibility: Visibility::Extern,
        });
    }
    symbols
}

pub fn compile_label<'a>(
    lbl: (&'a [Instruction<'a>], u16, u8),
    offset: usize,
) -> (Vec<u8>, Vec<Relocation<'a>>) {
    let mut bytes = Vec::with_capacity(lbl.0.len() << 1);
    let mut reallocs = Vec::new();
    let lbl_bits = lbl.2;
    let lbl_align = lbl.1;
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
    for ins in lbl.0 {
        let res = get_genapi(ins, lbl_bits).assemble(ins, lbl_bits);
        for mut rl in res.1.into_iter() {
            rl.offset += bytes.len();
            reallocs.push(rl);
        }
        match res.0 {
            AssembleResult::WLargeImm(i) => bytes.extend(i),
            AssembleResult::NoLargeImm(i) => bytes.extend(i.into_iter()),
        }
    }
    (bytes, reallocs)
}

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
        Mnemonic::JNE | Mnemonic::JNZ => ins_jmplike(ins, [&[0xFF, 0x85], &[], &[0x75]], 0, bits),
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
            .modrm(true, Some(7), None)
            .rex(true),

        Mnemonic::PAUSE => GenAPI::new().opcode(&[0xF3, 0x90]),
        Mnemonic::MWAIT => GenAPI::new().opcode(&[0x0F, 0x01, 0xC9]),

        Mnemonic::CMOVA => ins_cmovcc(ins, &[0x0F, 0x47], bits),
        Mnemonic::CMOVAE => ins_cmovcc(ins, &[0x0F, 0x43], bits),
        Mnemonic::CMOVB => ins_cmovcc(ins, &[0x0F, 0x42], bits),
        Mnemonic::CMOVBE => ins_cmovcc(ins, &[0x0F, 0x46], bits),
        Mnemonic::CMOVC => ins_cmovcc(ins, &[0x0F, 0x42], bits),
        Mnemonic::CMOVE => ins_cmovcc(ins, &[0x0F, 0x44], bits),
        Mnemonic::CMOVG => ins_cmovcc(ins, &[0x0F, 0x4F], bits),
        Mnemonic::CMOVGE => ins_cmovcc(ins, &[0x0F, 0x4D], bits),
        Mnemonic::CMOVL => ins_cmovcc(ins, &[0x0F, 0x4C], bits),
        Mnemonic::CMOVLE => ins_cmovcc(ins, &[0x0F, 0x4E], bits),
        Mnemonic::CMOVNA => ins_cmovcc(ins, &[0x0F, 0x46], bits),
        Mnemonic::CMOVNB => ins_cmovcc(ins, &[0x0F, 0x43], bits),
        Mnemonic::CMOVNBE => ins_cmovcc(ins, &[0x0F, 0x47], bits),
        Mnemonic::CMOVNC => ins_cmovcc(ins, &[0x0F, 0x43], bits),
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
            let mut api = GenAPI::new().modrm(true, None, None).rex(true).prefix(0xF3);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVHLPS => GenAPI::new()
            .modrm(true, None, None)
            .rex(true)
            .opcode(&[0x0F, 0x12])
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVLHPS => GenAPI::new()
            .modrm(true, None, None)
            .rex(true)
            .opcode(&[0x0F, 0x16])
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVAPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x29]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVUPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVLPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x13]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x12]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVHPS => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x17]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x16]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }

        Mnemonic::ADDPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x58])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ADDSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x58])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SUBPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x5C])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SUBSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5C])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MULPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x59])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MULSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x59])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DIVPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x5E])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DIVSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5E])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MINPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x5D])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MINSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5D])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MAXPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x5F])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MAXSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5F])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::RSQRTPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x52])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::RSQRTSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x52])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SHUFPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0xC6])
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SQRTPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x51])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SQRTSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x51])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CMPPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0xC2])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::CMPSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0xC2])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::RCPPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x53])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::RCPSS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x53])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::UCOMISS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x2E])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::COMISS => GenAPI::new()
            .modrm(true, None, None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0x2F])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ORPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x56])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ANDPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x54])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ANDNPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x55])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::XORPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x57])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::UNPCKLPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x14])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::UNPCKHPS => GenAPI::new()
            .modrm(true, None, None)
            .opcode(&[0x0F, 0x15])
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

        // SSE2
        Mnemonic::MOVNTI => GenAPI::new()
            .opcode(&[0x0F, 0xC3])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::MFENCE => GenAPI::new().opcode(&[0xF0, 0xAE, 0xF0]),
        Mnemonic::LFENCE => GenAPI::new().opcode(&[0xF0, 0xAE, 0xE8]),

        Mnemonic::MOVNTPD => GenAPI::new()
            .opcode(&[0x0F, 0x2B])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true),
        Mnemonic::MOVNTDQ => GenAPI::new()
            .opcode(&[0x0F, 0xE7])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true),
        Mnemonic::MOVAPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x29]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x28]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVUPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVLPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x13]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x12]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVHPD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x17]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x16]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVSD => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0xF2).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x11]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x10]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVDQA => {
            let mut api = GenAPI::new().modrm(true, None, None).prefix(0x66).rex(true);
            if let Some(Operand::Mem(_)) = ins.dst() {
                api = api.opcode(&[0x0F, 0x7F]).ord(&[MODRM_RM, MODRM_REG]);
            } else {
                api = api.opcode(&[0x0F, 0x6F]).ord(&[MODRM_REG, MODRM_RM]);
            }
            api
        }
        Mnemonic::MOVDQ2Q => GenAPI::new()
            .opcode(&[0x0F, 0xD6])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true),
        Mnemonic::MOVQ2DQ => GenAPI::new()
            .opcode(&[0x0F, 0xD6])
            .prefix(0xF3)
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::MOVMSKPD => GenAPI::new()
            .opcode(&[0x0F, 0x50])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::ADDPD => GenAPI::new()
            .opcode(&[0x0F, 0x58])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ADDSD => GenAPI::new()
            .opcode(&[0x0F, 0x58])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SUBPD => GenAPI::new()
            .opcode(&[0x0F, 0x5C])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SUBSD => GenAPI::new()
            .opcode(&[0x0F, 0x5C])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MULPD => GenAPI::new()
            .opcode(&[0x0F, 0x59])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MULSD => GenAPI::new()
            .opcode(&[0x0F, 0x59])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DIVPD => GenAPI::new()
            .opcode(&[0x0F, 0x5E])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DIVSD => GenAPI::new()
            .opcode(&[0x0F, 0x5E])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MINPD => GenAPI::new()
            .opcode(&[0x0F, 0x5D])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MINSD => GenAPI::new()
            .opcode(&[0x0F, 0x5D])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MAXPD => GenAPI::new()
            .opcode(&[0x0F, 0x5F])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MAXSD => GenAPI::new()
            .opcode(&[0x0F, 0x5F])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SQRTPD => GenAPI::new()
            .opcode(&[0x0F, 0x51])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::SQRTSD => GenAPI::new()
            .opcode(&[0x0F, 0x51])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CMPPD => GenAPI::new()
            .opcode(&[0x0F, 0xC2])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CMPSD => GenAPI::new()
            .opcode(&[0x0F, 0xC2])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::COMISD => GenAPI::new()
            .opcode(&[0x0F, 0x2F])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::UCOMISD => GenAPI::new()
            .opcode(&[0x0F, 0x2E])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ORPD => GenAPI::new()
            .opcode(&[0x0F, 0x56])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ANDPD => GenAPI::new()
            .opcode(&[0x0F, 0x54])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ANDNPD => GenAPI::new()
            .opcode(&[0x0F, 0x55])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::XORPD => GenAPI::new()
            .opcode(&[0x0F, 0x57])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PSHUFLW => GenAPI::new()
            .opcode(&[0x0F, 0x70])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PSHUFHW => GenAPI::new()
            .opcode(&[0x0F, 0x70])
            .prefix(0xF3)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PSHUFD => GenAPI::new()
            .opcode(&[0x0F, 0x70])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::PSLLDQ => GenAPI::new()
            .opcode(&[0x0F, 0x73])
            .prefix(0x66)
            .modrm(true, Some(7), None)
            .rex(true)
            .imm_atindex(1, 1),
        Mnemonic::PSRLDQ => GenAPI::new()
            .opcode(&[0x0F, 0x73])
            .prefix(0x66)
            .modrm(true, Some(3), None)
            .rex(true)
            .imm_atindex(1, 1),
        Mnemonic::PUNPCKHQDQ => GenAPI::new()
            .opcode(&[0x0F, 0x6D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .prefix(0x66)
            .rex(true),
        Mnemonic::PUNPCKLQDQ => GenAPI::new()
            .opcode(&[0x0F, 0x6C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .prefix(0x66)
            .rex(true),
        // MMX/SSE2
        Mnemonic::MOVD | Mnemonic::MOVQ => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66);
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
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PADDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFD])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PADDD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFE])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PADDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD4])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PADDUSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PADDUSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDD])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PADDSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEC])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PADDSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xED])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PSUBUSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD8])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PSUBUSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PSUBB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF8])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PSUBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PSUBD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFA])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PSUBQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xFB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::MASKMOVDQU => GenAPI::new()
            .opcode(&[0x0F, 0xF7])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::PSUBSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE8])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PSUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE9])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PMULLW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xD5])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PMULHW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE5])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PMULUDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF4])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PMADDWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xF5])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PCMPEQB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x74])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PCMPEQW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x75])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PCMPEQD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x76])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PCMPGTB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x64])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PCMPGTW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x65])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PCMPGTD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x66])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PACKUSWB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x67])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PACKSSWB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x63])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PACKSSDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x6B])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PUNPCKLBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x60])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PUNPCKLWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x61])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PUNPCKLDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x62])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PUNPCKHBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x68])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PUNPCKHWD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x69])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PUNPCKHDQ => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x6A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PSLLQ => {
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
        Mnemonic::PSLLD => {
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
        Mnemonic::PSLLW => {
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
        Mnemonic::PSRLW => {
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
        Mnemonic::PSRLD => {
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
        Mnemonic::PSRLQ => {
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
        Mnemonic::PSRAW => {
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
        Mnemonic::PSRAD => {
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

        Mnemonic::POR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PAND => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDB])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PANDN => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xDF])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PXOR => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEF])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::EMMS => GenAPI::new().opcode(&[0x0F, 0x77]),

        // sse3
        Mnemonic::ADDSUBPD => GenAPI::new()
            .opcode(&[0x0F, 0xD0])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::ADDSUBPS => GenAPI::new()
            .opcode(&[0x0F, 0xD0])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),

        Mnemonic::HADDPD => GenAPI::new()
            .opcode(&[0x0F, 0x7C])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::HADDPS => GenAPI::new()
            .opcode(&[0x0F, 0x7C])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::HSUBPD => GenAPI::new()
            .opcode(&[0x0F, 0x7D])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::HSUBPS => GenAPI::new()
            .opcode(&[0x0F, 0x7D])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),

        Mnemonic::MOVSLDUP => GenAPI::new()
            .opcode(&[0x0F, 0x12])
            .modrm(true, None, None)
            .prefix(0xF3)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVSHDUP => GenAPI::new()
            .opcode(&[0x0F, 0x16])
            .modrm(true, None, None)
            .prefix(0xF3)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVDDUP => GenAPI::new()
            .opcode(&[0x0F, 0x12])
            .modrm(true, None, None)
            .prefix(0xF2)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::LDDQU => GenAPI::new()
            .opcode(&[0x0F, 0xF0])
            .modrm(true, None, None)
            .prefix(0xF2)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::MONITOR => GenAPI::new().opcode(&[0x0F, 0x01, 0xC8]),

        // ssse3
        Mnemonic::PABSB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1C])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PABSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1D])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PABSD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x1E])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PSIGNB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x08])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PSIGNW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x09])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PSIGND => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x0A])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }

        Mnemonic::PSHUFB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x00])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PHADDW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x01])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PHADDD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x02])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PHADDSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x03])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PHSUBW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x05])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PHSUBD => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x06])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PHSUBSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x07])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PALIGNR => {
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
        Mnemonic::PMULHRSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0x38, 0x0B])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PMADDUBSW => {
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
        Mnemonic::DPPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x40])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::DPPD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x41])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PTEST => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x17])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PEXTRW => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x15])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1),
        Mnemonic::PEXTRB => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x14])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1),
        Mnemonic::PEXTRD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x16])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1),
        Mnemonic::PEXTRQ => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x16])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1),
        Mnemonic::PINSRB => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x20])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PINSRD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x22])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PINSRQ => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x22])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMAXSB => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3C])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMAXSD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3D])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMAXUW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3E])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMINSB => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x38])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMINSD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x39])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMINUW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3A])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMULDQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x28])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PMULLD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x40])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::BLENDPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0C])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::BLENDPD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0D])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PBLENDW => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0E])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPEQQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x29])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ROUNDPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x08])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ROUNDPD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x09])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ROUNDSS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0A])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ROUNDSD => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x0B])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MPSADBW => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x42])
            .prefix(0x66)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPGTQ => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x37])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::BLENDVPS => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x14])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::BLENDVPD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x15])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PBLENDVB => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x10])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::INSERTPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x21])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PACKUSDW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x2B])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::MOVNTDQA => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x2A])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPESTRM => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x60])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPESTRI => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x61])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPISTRM => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x62])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::PCMPISTRI => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x63])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::EXTRACTPS => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x17])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_RM, MODRM_REG]),
        Mnemonic::PHMINPOSUW => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x41])
            .prefix(0x66)
            .modrm(true, None, None)
            .rex(true)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CRC32 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF0])
            .prefix(0xF2)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::POPCNT => GenAPI::new()
            .opcode(&[0x0F, 0xB8])
            .prefix(0xF3)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),

        // AVX
        Mnemonic::VMOVDQA => {
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
        Mnemonic::VMOVSLDUP => GenAPI::new()
            .opcode(&[0x12])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VLDDQU => GenAPI::new()
            .opcode(&[0xF0])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VMOVDDUP => GenAPI::new()
            .opcode(&[0x12])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VMOVSHDUP => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VMOVMSKPD => GenAPI::new()
            .opcode(&[0x50])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VMOVAPS => {
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
        Mnemonic::VMOVAPD => {
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
        Mnemonic::VMOVUPS => {
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
        Mnemonic::VMOVUPD => {
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
        Mnemonic::VADDPS => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSUBPS => GenAPI::new()
            .opcode(&[0xD0])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSUBPD => GenAPI::new()
            .opcode(&[0xD0])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VHADDPS => GenAPI::new()
            .opcode(&[0x7C])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VHADDPD => GenAPI::new()
            .opcode(&[0x7C])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VHSUBPS => GenAPI::new()
            .opcode(&[0x7D])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VHSUBPD => GenAPI::new()
            .opcode(&[0x7D])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDPD => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSS => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSD => GenAPI::new()
            .opcode(&[0x58])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSUBPS => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSUBPD => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSUBSS => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSUBSD => GenAPI::new()
            .opcode(&[0x5C])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VMULPS => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMULPD => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMULSS => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMULSD => GenAPI::new()
            .opcode(&[0x59])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VDIVPS => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VDIVPD => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VDIVSS => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VDIVSD => GenAPI::new()
            .opcode(&[0x5E])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VRCPPS => GenAPI::new()
            .opcode(&[0x53])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VRCPSS => GenAPI::new()
            .opcode(&[0x53])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),

        Mnemonic::VSQRTPS => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VSQRTPD => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VSQRTSS => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VSQRTSD => GenAPI::new()
            .opcode(&[0x51])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VRSQRTPS => GenAPI::new()
            .opcode(&[0x52])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VRSQRTSS => GenAPI::new()
            .opcode(&[0x52])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMULDQ => GenAPI::new()
            .opcode(&[0x28])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULLD => GenAPI::new()
            .opcode(&[0x40])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINSB => GenAPI::new()
            .opcode(&[0x38])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINSD => GenAPI::new()
            .opcode(&[0x39])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINUB => GenAPI::new()
            .opcode(&[0xDA])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINUW => GenAPI::new()
            .opcode(&[0x3A])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXSB => GenAPI::new()
            .opcode(&[0x3C])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXSD => GenAPI::new()
            .opcode(&[0x3D])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXUB => GenAPI::new()
            .opcode(&[0xDE])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXUW => GenAPI::new()
            .opcode(&[0x3E])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VMINPS => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMINPD => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMINSS => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMINSD => GenAPI::new()
            .opcode(&[0x5D])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMAXPS => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMAXPD => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMAXSS => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMAXSD => GenAPI::new()
            .opcode(&[0x5F])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VORPS => GenAPI::new()
            .opcode(&[0x56])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VORPD => GenAPI::new()
            .opcode(&[0x56])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VANDPS => GenAPI::new()
            .opcode(&[0x54])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VANDPD => GenAPI::new()
            .opcode(&[0x54])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VANDNPD => GenAPI::new()
            .opcode(&[0x55])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VXORPD => GenAPI::new()
            .opcode(&[0x57])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        Mnemonic::VBLENDVPS => GenAPI::new()
            .opcode(&[0x4A])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VPBLENDVB => GenAPI::new()
            .opcode(&[0x4C])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VBLENDVPD => GenAPI::new()
            .opcode(&[0x4B])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),

        Mnemonic::VPHMINPOSUW => GenAPI::new()
            .opcode(&[0x41])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VEXTRACTPS => GenAPI::new()
            .opcode(&[0x17])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),

        Mnemonic::VMOVNTDQA => GenAPI::new()
            .opcode(&[0x2A])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPACKUSDW => GenAPI::new()
            .opcode(&[0x2B])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPESTRM => GenAPI::new()
            .opcode(&[0x60])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VPCMPESTRI => GenAPI::new()
            .opcode(&[0x61])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VPCMPISTRM => GenAPI::new()
            .opcode(&[0x62])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VPCMPISTRI => GenAPI::new()
            .opcode(&[0x63])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VINSERTPS => GenAPI::new()
            .opcode(&[0x21])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VBLENDPS => GenAPI::new()
            .opcode(&[0x0C])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VBLENDPD => GenAPI::new()
            .opcode(&[0x0D])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VPCMPGTQ => GenAPI::new()
            .opcode(&[0x37])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPEQQ => GenAPI::new()
            .opcode(&[0x29])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMPSADBW => GenAPI::new()
            .opcode(&[0x42])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VROUNDSS => GenAPI::new()
            .opcode(&[0x0A])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VROUNDSD => GenAPI::new()
            .opcode(&[0x0B])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VROUNDPS => GenAPI::new()
            .opcode(&[0x08])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VROUNDPD => GenAPI::new()
            .opcode(&[0x09])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1),
        Mnemonic::VPBLENDW => GenAPI::new()
            .opcode(&[0x0E])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VCMPPD => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VANDNPS => GenAPI::new()
            .opcode(&[0x55])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VXORPS => GenAPI::new()
            .opcode(&[0x57])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPTEST => GenAPI::new()
            .opcode(&[0x17])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VDPPS => GenAPI::new()
            .opcode(&[0x40])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VDPPD => GenAPI::new()
            .opcode(&[0x41])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VCMPPS => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VCMPSS => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VCMPSD => GenAPI::new()
            .opcode(&[0xC2])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VUCOMISS => GenAPI::new()
            .opcode(&[0x2E])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VUCOMISD => GenAPI::new()
            .opcode(&[0x2E])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCOMISS => GenAPI::new()
            .opcode(&[0x2F])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCOMISD => GenAPI::new()
            .opcode(&[0x2F])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VUNPCKLPS => GenAPI::new()
            .opcode(&[0x14])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VUNPCKHPS => GenAPI::new()
            .opcode(&[0x15])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VSHUFPS => GenAPI::new()
            .opcode(&[0xC6])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VMOVSS => {
            let mut api = GenAPI::new()
                .modrm(true, None, None)
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
                .modrm(true, None, None)
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
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false));
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
                .modrm(true, None, None)
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
                .modrm(true, None, None)
                .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false));
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
                .modrm(true, None, None)
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
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMOVHLPS => GenAPI::new()
            .opcode(&[0x12])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPEXTRB => GenAPI::new()
            .opcode(&[0x14])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Mnemonic::VPEXTRW => GenAPI::new()
            .opcode(&[0xC5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Mnemonic::VPEXTRD => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Mnemonic::VPEXTRQ => GenAPI::new()
            .opcode(&[0x16])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(true))
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1),
        Mnemonic::VPINSRB => GenAPI::new()
            .opcode(&[0x20])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VPINSRD => GenAPI::new()
            .opcode(&[0x22])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),
        Mnemonic::VPINSRQ => GenAPI::new()
            .opcode(&[0x22])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(true))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1),

        // MMX derived part 1
        Mnemonic::VPOR => GenAPI::new()
            .opcode(&[0xEB])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VMOVD | Mnemonic::VMOVQ => {
            let mut api = GenAPI::new().modrm(true, None, None).vex(
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
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPXOR => GenAPI::new()
            .opcode(&[0xEF])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDB => GenAPI::new()
            .opcode(&[0xFC])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDW => GenAPI::new()
            .opcode(&[0xFD])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDD => GenAPI::new()
            .opcode(&[0xFE])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDQ => GenAPI::new()
            .opcode(&[0xD4])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBB => GenAPI::new()
            .opcode(&[0xF8])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBW => GenAPI::new()
            .opcode(&[0xF9])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBD => GenAPI::new()
            .opcode(&[0xFA])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBQ => GenAPI::new()
            .opcode(&[0xFB])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPANDN => GenAPI::new()
            .opcode(&[0xDF])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSLLW => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::VPSLLD => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::VPSLLQ => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::VPSRLW => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::VPSRLD => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::VPSRLQ => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::VPSRAW => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::VPSRAD => {
            let mut api =
                GenAPI::new().vex(VexDetails::new().vex_we(false).pp(0x66).map_select(0x0F));
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::VPSUBSB => GenAPI::new()
            .opcode(&[0xE8])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBSW => GenAPI::new()
            .opcode(&[0xE9])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDSB => GenAPI::new()
            .opcode(&[0xEC])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDSW => GenAPI::new()
            .opcode(&[0xED])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULHW => GenAPI::new()
            .opcode(&[0xE5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULLW => GenAPI::new()
            .opcode(&[0xD5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        // part 2
        Mnemonic::VPADDUSB => GenAPI::new()
            .opcode(&[0xDC])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPADDUSW => GenAPI::new()
            .opcode(&[0xDD])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBUSB => GenAPI::new()
            .opcode(&[0xD8])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSUBUSW => GenAPI::new()
            .opcode(&[0xD9])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMADDWD => GenAPI::new()
            .opcode(&[0xF5])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPEQB => GenAPI::new()
            .opcode(&[0x74])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPEQW => GenAPI::new()
            .opcode(&[0x75])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPEQD => GenAPI::new()
            .opcode(&[0x76])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPGTB => GenAPI::new()
            .opcode(&[0x64])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPGTW => GenAPI::new()
            .opcode(&[0x65])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPCMPGTD => GenAPI::new()
            .opcode(&[0x66])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPACKUSWB => GenAPI::new()
            .opcode(&[0x67])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPACKSSWB => GenAPI::new()
            .opcode(&[0x63])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPACKSSDW => GenAPI::new()
            .opcode(&[0x6B])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKLBW => GenAPI::new()
            .opcode(&[0x60])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKLWD => GenAPI::new()
            .opcode(&[0x61])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKLDQ => GenAPI::new()
            .opcode(&[0x62])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKHBW => GenAPI::new()
            .opcode(&[0x68])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKHWD => GenAPI::new()
            .opcode(&[0x69])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPUNPCKHDQ => GenAPI::new()
            .opcode(&[0x6A])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),

        // part2a
        Mnemonic::PAVGB => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE0])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PAVGW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xE3])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::VPAVGB => GenAPI::new()
            .opcode(&[0xE0])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPAVGW => GenAPI::new()
            .opcode(&[0xE3])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPHADDW => GenAPI::new()
            .opcode(&[0x01])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPHADDD => GenAPI::new()
            .opcode(&[0x02])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPHSUBW => GenAPI::new()
            .opcode(&[0x05])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPHSUBD => GenAPI::new()
            .opcode(&[0x06])
            .vex(VexDetails::new().map_select(0x38).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VZEROUPPER => GenAPI::new().opcode(&[0xC5, 0xF8, 0x77]),
        Mnemonic::VZEROALL => GenAPI::new().opcode(&[0xC5, 0xFC, 0x77]),
        Mnemonic::VPALIGNR => GenAPI::new()
            .opcode(&[0x0F])
            .vex(VexDetails::new().map_select(0x3A).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .imm_atindex(3, 1)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VINSERTF128 => GenAPI::new()
            .opcode(&[0x18])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .modrm(true, None, None)
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
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .ord(&[MODRM_RM, MODRM_REG]),
        Mnemonic::VBROADCASTSS => GenAPI::new()
            .opcode(&[0x18])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VBROADCASTSD => GenAPI::new()
            .opcode(&[0x19])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(0x38)
                    .vex_we(ins.needs_evex()),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VBROADCASTF128 => GenAPI::new()
            .opcode(&[0x1A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::STMXCSR => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(3), None)
            .rex(true),
        Mnemonic::LDMXCSR => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(2), None)
            .rex(true),
        Mnemonic::VSTMXCSR => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false))
            .modrm(true, Some(3), None),
        Mnemonic::VLDMXCSR => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false))
            .modrm(true, Some(2), None),
        Mnemonic::VMOVMSKPS => GenAPI::new()
            .opcode(&[0x50])
            .vex(VexDetails::new().pp(0).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPERMILPS => {
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::VPERMILPD => {
            if let Some(Operand::Imm(_)) = ins.ssrc() {
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
        Mnemonic::PCLMULQDQ => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0x44])
            .prefix(0x66)
            .rex(true)
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPCLMULQDQ => GenAPI::new()
            .opcode(&[0x44])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPERM2F128 => GenAPI::new()
            .opcode(&[0x06])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPERM2I128 => GenAPI::new()
            .opcode(&[0x46])
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        // part2c
        Mnemonic::VPINSRW => GenAPI::new()
            .opcode(&[0xC4])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .imm_atindex(3, 1)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMAXSW => GenAPI::new()
            .opcode(&[0xEE])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMINSW => GenAPI::new()
            .opcode(&[0xEA])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSRLDQ => GenAPI::new()
            .opcode(&[0x73])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .imm_atindex(2, 1)
            .modrm(true, Some(3), None)
            .ord(&[VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSIGNB => GenAPI::new()
            .opcode(&[0x08])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSIGNW => GenAPI::new()
            .opcode(&[0x09])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPSIGND => GenAPI::new()
            .opcode(&[0x0A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULUDQ => GenAPI::new()
            .opcode(&[0xF4])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULHUW => GenAPI::new()
            .opcode(&[0xE4])
            .vex(VexDetails::new().pp(0x66).map_select(0x0F).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VPMULHRSW => GenAPI::new()
            .opcode(&[0x0B])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        // part2c-ext
        Mnemonic::PMAXSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEE])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PINSRW => {
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
        Mnemonic::PMINSW => {
            let mut api = GenAPI::new()
                .opcode(&[0x0F, 0xEA])
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM]);
            if ins.which_variant() != IVariant::MMX {
                api = api.prefix(0x66).rex(true);
            }
            api
        }
        Mnemonic::PMAXUD => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0x3F])
            .prefix(0x66)
            .rex(true)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VPMAXUD => GenAPI::new()
            .opcode(&[0x3F])
            .modrm(true, None, None)
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::PMULHUW => {
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
        Mnemonic::VFMADD132PS => GenAPI::new()
            .opcode(&[0x98])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD213PS => GenAPI::new()
            .opcode(&[0xA8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD231PS => GenAPI::new()
            .opcode(&[0xB8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD132PD => GenAPI::new()
            .opcode(&[0x98])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD213PD => GenAPI::new()
            .opcode(&[0xA8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD231PD => GenAPI::new()
            .opcode(&[0xB8])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD132SD => GenAPI::new()
            .opcode(&[0x99])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD213SD => GenAPI::new()
            .opcode(&[0xA9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD231SD => GenAPI::new()
            .opcode(&[0xB9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD132SS => GenAPI::new()
            .opcode(&[0x99])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD213SS => GenAPI::new()
            .opcode(&[0xA9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADD231SS => GenAPI::new()
            .opcode(&[0xB9])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFMSUB132PS => GenAPI::new()
            .opcode(&[0x9A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB213PS => GenAPI::new()
            .opcode(&[0xAA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB231PS => GenAPI::new()
            .opcode(&[0xBA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFMSUB132PD => GenAPI::new()
            .opcode(&[0x9A])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB213PD => GenAPI::new()
            .opcode(&[0xAA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB231PD => GenAPI::new()
            .opcode(&[0xBA])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB132SD => GenAPI::new()
            .opcode(&[0x9B])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB213SD => GenAPI::new()
            .opcode(&[0xAB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB231SD => GenAPI::new()
            .opcode(&[0xBB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB132SS => GenAPI::new()
            .opcode(&[0x9B])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB213SS => GenAPI::new()
            .opcode(&[0xAB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUB231SS => GenAPI::new()
            .opcode(&[0xBB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        // fma-part2
        Mnemonic::VFNMADD132PS => GenAPI::new()
            .opcode(&[0x9C])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD213PS => GenAPI::new()
            .opcode(&[0xAC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD231PS => GenAPI::new()
            .opcode(&[0xBC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMADD132PD => GenAPI::new()
            .opcode(&[0x9C])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD213PD => GenAPI::new()
            .opcode(&[0xAC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD231PD => GenAPI::new()
            .opcode(&[0xBC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMADD132SS => GenAPI::new()
            .opcode(&[0x9D])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD213SS => GenAPI::new()
            .opcode(&[0xAD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD231SS => GenAPI::new()
            .opcode(&[0xBD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMADD132SD => GenAPI::new()
            .opcode(&[0x9D])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD213SD => GenAPI::new()
            .opcode(&[0xAD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMADD231SD => GenAPI::new()
            .opcode(&[0xBD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMSUB132PS => GenAPI::new()
            .opcode(&[0x9E])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB213PS => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB231PS => GenAPI::new()
            .opcode(&[0xBE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMSUB132PD => GenAPI::new()
            .opcode(&[0x9E])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB213PD => GenAPI::new()
            .opcode(&[0xAE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB231PD => GenAPI::new()
            .opcode(&[0xBE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMSUB132SS => GenAPI::new()
            .opcode(&[0x9F])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB213SS => GenAPI::new()
            .opcode(&[0xAF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB231SS => GenAPI::new()
            .opcode(&[0xBF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFNMSUB132SD => GenAPI::new()
            .opcode(&[0x9F])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB213SD => GenAPI::new()
            .opcode(&[0xAF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFNMSUB231SD => GenAPI::new()
            .opcode(&[0xBF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        // fma-part3
        Mnemonic::VFMADDSUB132PS => GenAPI::new()
            .opcode(&[0x96])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB213PS => GenAPI::new()
            .opcode(&[0xA6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB231PS => GenAPI::new()
            .opcode(&[0xB6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB132PD => GenAPI::new()
            .opcode(&[0x96])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB213PD => GenAPI::new()
            .opcode(&[0xA6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMADDSUB231PD => GenAPI::new()
            .opcode(&[0xB6])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),

        Mnemonic::VFMSUBADD132PS => GenAPI::new()
            .opcode(&[0x97])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD213PS => GenAPI::new()
            .opcode(&[0xA7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD231PS => GenAPI::new()
            .opcode(&[0xB7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD132PD => GenAPI::new()
            .opcode(&[0x97])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD213PD => GenAPI::new()
            .opcode(&[0xA7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VFMSUBADD231PD => GenAPI::new()
            .opcode(&[0xB7])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        // aes
        Mnemonic::AESDEC => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDE])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::AESENC => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDC])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::AESIMC => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDB])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::AESDECLAST => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDF])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::AESENCLAST => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x38, 0xDD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),

        Mnemonic::VAESDEC => GenAPI::new()
            .opcode(&[0xDE])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VAESENC => GenAPI::new()
            .opcode(&[0xDC])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VAESIMC => GenAPI::new()
            .opcode(&[0xDB])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM]),
        Mnemonic::VAESENCLAST => GenAPI::new()
            .opcode(&[0xDD])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VAESDECLAST => GenAPI::new()
            .opcode(&[0xDF])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::VEX_VVVV, OpOrd::MODRM_RM]),
        Mnemonic::VAESKEYGENASSIST => GenAPI::new()
            .opcode(&[0xDF])
            .imm_atindex(2, 1)
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false))
            .modrm(true, None, None)
            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM]),
        Mnemonic::AESKEYGENASSIST => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0xDF])
            .modrm(true, None, None)
            .imm_atindex(2, 1)
            .prefix(0x66)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        // cvt-part1
        Mnemonic::CVTPD2PI => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x2D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTSS2SD => GenAPI::new()
            .prefix(0xF3)
            .opcode(&[0x0F, 0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTPD2PS => GenAPI::new()
            .prefix(0x66)
            .opcode(&[0x0F, 0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTPS2PD => GenAPI::new()
            .opcode(&[0x0F, 0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTPI2PD => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTPD2DQ => GenAPI::new()
            .opcode(&[0x0F, 0xE6])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTSD2SS => GenAPI::new()
            .opcode(&[0x0F, 0x5A])
            .prefix(0xF2)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTPS2DQ => GenAPI::new()
            .opcode(&[0x0F, 0x5B])
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTDQ2PS => GenAPI::new()
            .opcode(&[0x0F, 0x5B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTDQ2PD => GenAPI::new()
            .opcode(&[0x0F, 0xE6])
            .modrm(true, None, None)
            .prefix(0xF3)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTSD2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2D])
            .modrm(true, None, None)
            .prefix(0xF2)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTSI2SD => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .modrm(true, None, None)
            .prefix(0xF2)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),

        Mnemonic::CVTTPS2DQ => GenAPI::new()
            .opcode(&[0x0F, 0x5B])
            .prefix(0xF3)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTTSD2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None, None)
            .prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTTPD2PI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None, None)
            .prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTSI2SS => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .prefix(0xF3)
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::CVTPS2PI => GenAPI::new()
            .opcode(&[0x0F, 0x2D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTTPS2PI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTPI2PS => GenAPI::new()
            .opcode(&[0x0F, 0x2A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTTPD2DQ => GenAPI::new()
            .opcode(&[0x0F, 0xE6])
            .modrm(true, None, None)
            .prefix(0x66)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::CVTTSS2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .prefix(0xF3)
            .rex(true),
        Mnemonic::CVTSS2SI => GenAPI::new()
            .opcode(&[0x0F, 0x2D])
            .prefix(0xF3)
            .rex(true)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        // cvt-part2
        Mnemonic::VCVTPD2DQ => GenAPI::new()
            .opcode(&[0xE6])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTPD2PS => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTPS2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTPS2PD => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTSD2SI => GenAPI::new()
            .opcode(&[0x2D])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF2)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTSD2SS => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF2).vex_we(false))
            .modrm(true, None, None)
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
            .modrm(true, None, None),
        Mnemonic::VCVTSI2SS => GenAPI::new()
            .opcode(&[0x2A])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF3)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VCVTSS2SD => GenAPI::new()
            .opcode(&[0x5A])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
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
            .modrm(true, None, None),
        Mnemonic::VCVTDQ2PD => GenAPI::new()
            .opcode(&[0xE6])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTDQ2PS => GenAPI::new()
            .opcode(&[0x5B])
            .vex(VexDetails::new().map_select(0x0F).pp(0).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTTPD2DQ => GenAPI::new()
            .opcode(&[0xE6])
            .vex(VexDetails::new().map_select(0x0F).pp(0x66).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTTPS2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .vex(VexDetails::new().map_select(0x0F).pp(0xF3).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTTSD2SI => GenAPI::new()
            .opcode(&[0x2C])
            .vex(
                VexDetails::new()
                    .map_select(0x0F)
                    .pp(0xF2)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::VCVTTSS2SI => GenAPI::new()
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
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::BSR => GenAPI::new()
            .opcode(&[0x0F, 0xBD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        // part b
        Mnemonic::ADCX => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF6])
            .modrm(true, None, None)
            .prefix(0x66)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM]),
        Mnemonic::ADOX => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF6])
            .modrm(true, None, None)
            .prefix(0xF3)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::ANDN => GenAPI::new()
            .opcode(&[0xF2])
            .modrm(true, None, None)
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
        Mnemonic::CLUI => GenAPI::new().prefix(0xF3).opcode(&[0x0F, 0x01, 0xEE]),
        Mnemonic::CLWB => {
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .prefix(0x66)
                .modrm(true, Some(6), None)
        }
        Mnemonic::ARPL => GenAPI::new().opcode(&[0x63]).modrm(true, None, None),

        Mnemonic::BLSR => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(1), None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[VEX_VVVV, MODRM_RM]),
        Mnemonic::BLSI => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(3), None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[VEX_VVVV, MODRM_RM]),
        Mnemonic::BZHI => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None, None)
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
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::BLSMSK => GenAPI::new()
            .opcode(&[0xF3])
            .modrm(true, Some(2), None)
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[VEX_VVVV, MODRM_RM]),
        Mnemonic::BSWAP => GenAPI::new()
            .opcode(&[0x0F, 0xC8 + ins.reg_byte(0).unwrap_or(0)])
            .modrm(false, None, None)
            .rex(true),
        // part c
        Mnemonic::CMPSTRB => GenAPI::new().opcode(&[0xA6]).fixed_size(Size::Byte),
        Mnemonic::CMPSTRW => GenAPI::new().opcode(&[0xA7]).fixed_size(Size::Word),
        Mnemonic::CMPSTRD => GenAPI::new().opcode(&[0xA7]).fixed_size(Size::Dword),
        Mnemonic::CMPSTRQ => GenAPI::new().opcode(&[0x48, 0xA7]).fixed_size(Size::Qword),
        Mnemonic::ENDBR64 => GenAPI::new().prefix(0xF3).opcode(&[0x0F, 0x1E, 0xFA]),
        Mnemonic::ENDBR32 => GenAPI::new().prefix(0xF3).opcode(&[0x0F, 0x1E, 0xFB]),
        Mnemonic::CMPXCHG => GenAPI::new()
            .opcode(&[0x0F, (0xB1 - ((ins.size() == Size::Byte) as u8))])
            .modrm(true, None, None)
            .rex(true),
        Mnemonic::CLDEMOTE => GenAPI::new()
            .opcode(&[0x0F, 0x1C])
            .modrm(true, Some(0), None),
        Mnemonic::CLRSSBSY => {
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .prefix(0xF3)
                .modrm(true, Some(6), None)
        }
        Mnemonic::CMPXCHG8B => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(1), None),
        Mnemonic::CMPXCHG16B => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(1), None)
            .rex(true),
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
            .prefix(0xF3)
            .opcode(&[0x0F, 0x3A, 0xF0, 0xC0])
            .modrm(false, None, None)
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
            .prefix(0x66)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::IRET | Mnemonic::IRETD => GenAPI::new().opcode(&[0xCF]),
        Mnemonic::IRETQ => GenAPI::new().opcode(&[0x48, 0xCF]),
        Mnemonic::LAHF => GenAPI::new().opcode(&[0x9F]),
        Mnemonic::LAR => GenAPI::new()
            .opcode(&[0x0F, 0x02])
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::LEAVE => GenAPI::new().opcode(&[0xC9]),
        Mnemonic::LLDT => GenAPI::new()
            .opcode(&[0x0F, 0x00])
            .modrm(true, Some(2), None)
            .can_h66(false),
        Mnemonic::LMSW => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(6), None)
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
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::LTR => GenAPI::new()
            .opcode(&[0x0F, 0x00])
            .modrm(true, Some(3), None)
            .can_h66(false),
        Mnemonic::LZCNT => GenAPI::new()
            .opcode(&[0x0F, 0xBD])
            .prefix(0xF3)
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::MOVBE => {
            let mut api = GenAPI::new().modrm(true, None, None).rex(true);
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
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .rex(true),
        Mnemonic::MOVDIRI => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xF9])
            .modrm(true, None, None)
            .rex(true),
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
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::OUTSB => GenAPI::new().opcode(&[0x6E]).fixed_size(Size::Byte),
        Mnemonic::OUTSW => GenAPI::new().opcode(&[0x6F]).fixed_size(Size::Word),
        Mnemonic::OUTSD => GenAPI::new().opcode(&[0x6F]).fixed_size(Size::Dword),
        Mnemonic::PEXT => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None, None)
            .vex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::PDEP => GenAPI::new()
            .opcode(&[0xF5])
            .modrm(true, None, None)
            .vex(
                VexDetails::new()
                    .pp(0xF2)
                    .map_select(0x38)
                    .vex_we(ins.size() == Size::Qword),
            )
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::PREFETCHW => GenAPI::new()
            .opcode(&[0x0F, 0x0D])
            .modrm(true, Some(1), None),
        Mnemonic::PREFETCH0 => GenAPI::new()
            .opcode(&[0x0F, 0x18])
            .modrm(true, Some(1), None),
        Mnemonic::PREFETCH1 => GenAPI::new()
            .opcode(&[0x0F, 0x18])
            .modrm(true, Some(2), None),
        Mnemonic::PREFETCH2 => GenAPI::new()
            .opcode(&[0x0F, 0x18])
            .modrm(true, Some(3), None),
        Mnemonic::PREFETCHA => GenAPI::new()
            .opcode(&[0x0F, 0x18])
            .modrm(true, Some(0), None),

        Mnemonic::ROL => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 0, bits),
        Mnemonic::ROR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 1, bits),
        Mnemonic::RCL => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 2, bits),
        Mnemonic::RCR => ins_shllike(ins, &[0xD0, 0xD2, 0xC0, 0xD1, 0xD3, 0xC1], 3, bits),
        // part 4
        Mnemonic::RDMSR => GenAPI::new().opcode(&[0x0F, 0x32]),
        Mnemonic::RDPID => {
            GenAPI::new()
                .opcode(&[0x0F, 0xC7])
                .prefix(0xF3)
                .modrm(true, Some(7), None)
        }
        Mnemonic::RDPKRU => GenAPI::new().opcode(&[0x0F, 0x01, 0xEE]),
        Mnemonic::RDPMC => GenAPI::new().opcode(&[0x0F, 0x33]),
        Mnemonic::RDRAND => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(6), None)
            .rex(true),
        Mnemonic::RDSEED => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(7), None)
            .rex(true),
        Mnemonic::RDSSPD | Mnemonic::RDSSPQ => GenAPI::new()
            .opcode(&[0x0F, 0x1E])
            .modrm(true, Some(1), None)
            .prefix(0xF3)
            .rex(true),
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
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .imm_atindex(2, 1),
        Mnemonic::RSM => GenAPI::new().opcode(&[0x0F, 0xAA]),
        Mnemonic::RSTORSSP => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(5), None)
            .prefix(0xF3)
            .rex(true),
        Mnemonic::SAHF => GenAPI::new().opcode(&[0x9E]),
        Mnemonic::SHLX => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0x66)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SHRX => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0xF2)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SARX => GenAPI::new()
            .opcode(&[0xF7])
            .vex(
                VexDetails::new()
                    .map_select(0x38)
                    .pp(0xF3)
                    .vex_we(ins.size() == Size::Qword),
            )
            .modrm(true, None, None)
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
            .prefix(0xF3)
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(6), None)
            .can_h66(false)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SERIALIZE => GenAPI::new().opcode(&[0x0F, 0x01, 0xE8]),
        // for some reason NASM generates this as no opcode at all?
        Mnemonic::SETSSBY => GenAPI::new(),

        // setcc
        Mnemonic::SETO => GenAPI::new()
            .opcode(&[0x0F, 0x90])
            .modrm(true, None, None)
            .rex(true),
        Mnemonic::SETNO => GenAPI::new()
            .opcode(&[0x0F, 0x91])
            .modrm(true, None, None)
            .rex(true),
        Mnemonic::SETB | Mnemonic::SETC | Mnemonic::SETNAE => GenAPI::new()
            .opcode(&[0x0F, 0x92])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETAE | Mnemonic::SETNB | Mnemonic::SETNC => GenAPI::new()
            .opcode(&[0x0F, 0x93])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETE | Mnemonic::SETZ => GenAPI::new()
            .opcode(&[0x0F, 0x94])
            .modrm(true, None, None)
            .rex(true),
        Mnemonic::SETNE | Mnemonic::SETNZ => GenAPI::new()
            .opcode(&[0x0F, 0x95])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETBE | Mnemonic::SETNA => GenAPI::new()
            .opcode(&[0x0F, 0x96])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETA | Mnemonic::SETNBE => GenAPI::new()
            .opcode(&[0x0F, 0x97])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETS => GenAPI::new()
            .opcode(&[0x0F, 0x98])
            .modrm(true, None, None)
            .rex(true),
        Mnemonic::SETNS => GenAPI::new()
            .opcode(&[0x0F, 0x99])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETP | Mnemonic::SETPE => GenAPI::new()
            .opcode(&[0x0F, 0x9A])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETNP | Mnemonic::SETPO => GenAPI::new()
            .opcode(&[0x0F, 0x9B])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETL | Mnemonic::SETNGE => GenAPI::new()
            .opcode(&[0x0F, 0x9C])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETGE | Mnemonic::SETNL => GenAPI::new()
            .opcode(&[0x0F, 0x9D])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETLE | Mnemonic::SETNG => GenAPI::new()
            .opcode(&[0x0F, 0x9E])
            .modrm(true, None, None)
            .rex(true),

        Mnemonic::SETG | Mnemonic::SETNLE => GenAPI::new()
            .opcode(&[0x0F, 0x9F])
            .modrm(true, None, None)
            .rex(true),

        // norm-part5
        Mnemonic::SFENCE => GenAPI::new().opcode(&[0x0F, 0xAE, 0xF8]),
        Mnemonic::STAC => GenAPI::new().opcode(&[0x0F, 0x01, 0xCB]),
        Mnemonic::STC => GenAPI::new().opcode(&[0xF9]),
        Mnemonic::STD => GenAPI::new().opcode(&[0xFD]),
        Mnemonic::STI => GenAPI::new().opcode(&[0xFB]),
        Mnemonic::STUI => GenAPI::new().prefix(0xF3).opcode(&[0x0F, 0x01, 0xEF]),
        Mnemonic::STOSB => GenAPI::new().opcode(&[0xAA]),
        Mnemonic::STOSW => GenAPI::new().opcode(&[0xAB]).fixed_size(Size::Word),
        Mnemonic::STOSD => GenAPI::new().opcode(&[0xAB]).fixed_size(Size::Dword),
        Mnemonic::STOSQ => GenAPI::new().opcode(&[0x48, 0xAB]),
        Mnemonic::SYSENTER => GenAPI::new().opcode(&[0x0F, 0x34]),
        Mnemonic::SYSEXIT => GenAPI::new().opcode(&[0x0F, 0x35]),
        Mnemonic::SYSRET => GenAPI::new().opcode(&[0x0F, 0x07]),
        Mnemonic::TESTUI => GenAPI::new().prefix(0xF3).opcode(&[0x0F, 0x01, 0xED]),
        Mnemonic::UD2 => GenAPI::new().opcode(&[0x0F, 0x0B]),
        Mnemonic::UD0 => GenAPI::new()
            .opcode(&[0x0F, 0xFF])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::UD1 => GenAPI::new()
            .opcode(&[0x0F, 0xB9])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::TPAUSE => GenAPI::new()
            .modrm(true, Some(6), None)
            .prefix(0x66)
            .opcode(&[0x0F, 0xAE]),
        Mnemonic::UMWAIT => GenAPI::new()
            .modrm(true, Some(6), None)
            .prefix(0xF2)
            .opcode(&[0x0F, 0xAE]),
        Mnemonic::UMONITOR => GenAPI::new()
            .modrm(true, Some(6), None)
            .prefix(0xF3)
            .opcode(&[0x0F, 0xAE]),
        Mnemonic::SMSW => GenAPI::new()
            .modrm(true, Some(4), None)
            .opcode(&[0x0F, 0x01])
            .rex(true),
        Mnemonic::STR => GenAPI::new()
            .modrm(true, Some(1), None)
            .opcode(&[0x0F, 0x00]),
        Mnemonic::VERR => GenAPI::new()
            .modrm(true, Some(4), None)
            .opcode(&[0x0F, 0x00])
            .can_h66(false),
        Mnemonic::VERW => GenAPI::new()
            .modrm(true, Some(5), None)
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
            .modrm(true, None, None)
            .rex(true),
        Mnemonic::XBEGIN => ins_xbegin(ins),
        Mnemonic::XCHG => ins_xchg(ins),
        Mnemonic::XEND => GenAPI::new().opcode(&[0x0F, 0x01, 0xD5]),
        Mnemonic::XGETBV => GenAPI::new().opcode(&[0x0F, 0x01, 0xD0]),
        Mnemonic::XLAT | Mnemonic::XLATB => GenAPI::new().opcode(&[0xD7]),
        Mnemonic::XLATB64 => GenAPI::new().opcode(&[0x48, 0xD7]),
        Mnemonic::XRESLDTRK => GenAPI::new().prefix(0xF2).opcode(&[0x0F, 0x01, 0xE9]),

        Mnemonic::XRSTOR | Mnemonic::XRSTOR64 => {
            GenAPI::new()
                .opcode(&[0x0F, 0xAE])
                .modrm(true, Some(5), None)
        }
        Mnemonic::XRSTORS | Mnemonic::XRSTORS64 => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(3), None)
            .rex(true),
        Mnemonic::XSAVE | Mnemonic::XSAVE64 => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(4), None)
            .rex(true),
        Mnemonic::XSAVEC | Mnemonic::XSAVEC64 => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(4), None)
            .rex(true),
        Mnemonic::XSAVEOPT | Mnemonic::XSAVEOPT64 => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(6), None)
            .rex(true),
        Mnemonic::XSAVES | Mnemonic::XSAVES64 => GenAPI::new()
            .opcode(&[0x0F, 0xC7])
            .modrm(true, Some(5), None)
            .rex(true),
        Mnemonic::XSETBV => GenAPI::new().opcode(&[0x0F, 0x01, 0xD1]),
        Mnemonic::XSUSLDTRK => GenAPI::new().prefix(0xF2).opcode(&[0x0F, 0x01, 0xE8]),
        Mnemonic::XTEST => GenAPI::new().opcode(&[0x0F, 0x01, 0xD6]),
        // sha.asm
        Mnemonic::SHA1MSG1 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xC9])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true),
        Mnemonic::SHA1NEXTE => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xC8])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SHA1MSG2 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCA])
            .modrm(true, None, None)
            .rex(true)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV]),
        Mnemonic::SHA1RNDS4 => GenAPI::new()
            .opcode(&[0x0F, 0x3A, 0xCC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true)
            .imm_atindex(2, 1),
        Mnemonic::SHA256RNDS2 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCB])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true),
        Mnemonic::SHA256MSG2 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true),
        Mnemonic::SHA256MSG1 => GenAPI::new()
            .opcode(&[0x0F, 0x38, 0xCC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .rex(true),

        // fxd
        Mnemonic::WRGSBASE => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(3), None)
            .prefix(0xF3)
            .rex(true),
        Mnemonic::WRFSBASE => GenAPI::new()
            .opcode(&[0x0F, 0xAE])
            .modrm(true, Some(2), None)
            .prefix(0xF3)
            .rex(true),
        Mnemonic::LIDT => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(3), None)
            .rex(true)
            .can_h66(false),
        Mnemonic::LGDT => GenAPI::new()
            .opcode(&[0x0F, 0x01])
            .modrm(true, Some(2), None)
            .rex(true)
            .can_h66(false),
        Mnemonic::LOCK => GenAPI::new().opcode(&[0xF0]),
        Mnemonic::REPNE | Mnemonic::REPNZ => GenAPI::new().opcode(&[0xF2]),
        Mnemonic::REP | Mnemonic::REPE | Mnemonic::REPZ => GenAPI::new().opcode(&[0xF3]),

        Mnemonic::VADDPH => GenAPI::new()
            .opcode(&[0x58])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VADDSH => GenAPI::new()
            .opcode(&[0x58])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false).pp(0xF3))
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::VALIGNQ => GenAPI::new()
            .opcode(&[0x03])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None, None)
            .imm_atindex(3, 1),
        Mnemonic::VALIGND => GenAPI::new()
            .opcode(&[0x03])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None, None)
            .imm_atindex(3, 1),
        Mnemonic::VBCSTNESH2PS => GenAPI::new()
            .opcode(&[0xB1])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VBCSTNEBF162PS => GenAPI::new()
            .opcode(&[0xB1])
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VBLENDMPS => GenAPI::new()
            .opcode(&[0x65])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VBLENDMPD => GenAPI::new()
            .opcode(&[0x65])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VBROADCASTF32X2 => GenAPI::new()
            .opcode(&[0x19])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VBROADCASTF32X4 => GenAPI::new()
            .opcode(&[0x1A])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VBROADCASTF64X2 => GenAPI::new()
            .opcode(&[0x1A])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VBROADCASTF32X8 => GenAPI::new()
            .opcode(&[0x1B])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VBROADCASTF64X4 => GenAPI::new()
            .opcode(&[0x1B])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VCMPPH => GenAPI::new()
            .opcode(&[0xC2])
            .evex(VexDetails::new().map_select(MAP3A))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None, None)
            .imm_atindex(3, 1),
        Mnemonic::VCMPSH => GenAPI::new()
            .opcode(&[0xC2])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A))
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .modrm(true, None, None)
            .imm_atindex(3, 1),
        Mnemonic::VCOMISH => GenAPI::new()
            .opcode(&[0x2F])
            .evex(VexDetails::new().map_select(MAP5))
            .ord(&[MODRM_REG, MODRM_RM])
            .modrm(true, None, None),
        Mnemonic::VCOMPRESSPD | Mnemonic::VCOMPRESSPS => GenAPI::new()
            .opcode(&[0x8A])
            .evex(
                VexDetails::new()
                    .map_select(MAP38)
                    .pp(0x66)
                    .vex_we(ins.mnemonic == Mnemonic::VCOMPRESSPD),
            )
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
            .strict_pfx()
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]),
        Mnemonic::KUNPCKWD => GenAPI::new()
            .opcode(&[0x4B])
            .vex(VexDetails::new().map_select(0x0F).vlength(Some(true)))
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
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
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTNE2PS2BF16 => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VCVTNEEBF162PS => GenAPI::new()
            .opcode(&[0xB0])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(0x38).vex_we(false)),
        Mnemonic::VCVTNEEPH2PS => GenAPI::new()
            .opcode(&[0xB0])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VCVTNEOBF162PS => GenAPI::new()
            .opcode(&[0xB0])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),
        Mnemonic::VCVTNEOPH2PS => GenAPI::new()
            .opcode(&[0xB0])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(0x38).vex_we(false)),
        Mnemonic::VCVTNEPS2BF16 => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0xF3).map_select(0x38).vex_we(false)),
        Mnemonic::VCVTPD2PH => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(true)),
        Mnemonic::VCVTPD2QQ => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTPD2UDQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTPD2UQQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTPH2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2PD => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2PS => GenAPI::new()
            .opcode(&[0x13])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VCVTPH2PSX => GenAPI::new()
            .opcode(&[0x13])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VCVTPH2QQ => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2UDQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2UQQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2UW => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPH2W => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPS2PH => GenAPI::new()
            .opcode(&[0x1D])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VCVTPS2PHX => GenAPI::new()
            .opcode(&[0x1D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTPS2QQ => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTPS2UDQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTPS2UQQ => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTQQ2PD => GenAPI::new()
            .opcode(&[0xE6])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTQQ2PH => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(true)),
        Mnemonic::VCVTQQ2PS => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTSD2SH => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP5).vex_we(true)),
        Mnemonic::VCVTSD2USI => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF2)
                    .map_select(MAP0F)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTSH2SD => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTSH2SI => GenAPI::new()
            .opcode(&[0x2D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTSH2SS => GenAPI::new()
            .opcode(&[0x13])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP6).vex_we(false)),
        Mnemonic::VCVTSH2USI => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTSI2SH => GenAPI::new()
            .opcode(&[0x2A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTSS2SH => GenAPI::new()
            .opcode(&[0x1D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTSS2USI => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP0F)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTTPD2QQ => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTTPD2UDQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTTPD2UQQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTTPH2DQ => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2QQ => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2UDQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2UQQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2UW => GenAPI::new()
            .opcode(&[0x7C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPH2W => GenAPI::new()
            .opcode(&[0x7C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTTPS2QQ => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTTPS2UDQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTTPS2UQQ => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTTSD2USI => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF2)
                    .map_select(MAP0F)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTTSH2SI => GenAPI::new()
            .opcode(&[0x2C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTTSH2USI => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTTSS2USI => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP0F)
                    .vex_we(ins.dst().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTUDQ2PD => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTUDQ2PH => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTUDQ2PS => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP0F).vex_we(false)),
        Mnemonic::VCVTUQQ2PD => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTUQQ2PH => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP5).vex_we(true)),
        Mnemonic::VCVTUQQ2PS => GenAPI::new()
            .opcode(&[0x7A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP0F).vex_we(true)),
        Mnemonic::VCVTUSI2SD => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF2)
                    .map_select(MAP0F)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTUSI2SH => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP5)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTUSI2SS => GenAPI::new()
            .opcode(&[0x7B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(
                VexDetails::new()
                    .pp(0xF3)
                    .map_select(MAP0F)
                    .vex_we(ins.ssrc().unwrap().size() == Size::Qword),
            ),
        Mnemonic::VCVTUW2PH => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP5).vex_we(false)),
        Mnemonic::VCVTW2PH => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),

        Mnemonic::VDBPSADBW => GenAPI::new()
            .opcode(&[0x42])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VDIVPH => GenAPI::new()
            .opcode(&[0x5E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VDIVSH => GenAPI::new()
            .opcode(&[0x5E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VDPBF16PS => GenAPI::new()
            .opcode(&[0x52])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VEXPANDPD => GenAPI::new()
            .opcode(&[0x88])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VEXPANDPS => GenAPI::new()
            .opcode(&[0x88])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VEXTRACTF32X4 => GenAPI::new()
            .opcode(&[0x19])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTF64X2 => GenAPI::new()
            .opcode(&[0x19])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VEXTRACTF32X8 => GenAPI::new()
            .opcode(&[0x1B])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTF64X4 => GenAPI::new()
            .opcode(&[0x1B])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VEXTRACTI128 => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTI32X4 => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTI32X8 => GenAPI::new()
            .opcode(&[0x3B])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VEXTRACTI64X4 => GenAPI::new()
            .opcode(&[0x3B])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VEXTRACTI64X2 => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFCMADDCPH => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDCPH => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP6).vex_we(false)),
        Mnemonic::VFCMADDCSH => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDCSH => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP6).vex_we(false)),
        Mnemonic::VFCMULCPH => GenAPI::new()
            .opcode(&[0xD6])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMULCPH => GenAPI::new()
            .opcode(&[0xD6])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP6).vex_we(false)),
        Mnemonic::VFCMULCSH => GenAPI::new()
            .opcode(&[0xD7])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMULCSH => GenAPI::new()
            .opcode(&[0xD7])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP6).vex_we(false)),
        Mnemonic::VFIXUPIMMPD => GenAPI::new()
            .opcode(&[0x54])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFIXUPIMMPS => GenAPI::new()
            .opcode(&[0x54])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VFIXUPIMMSD => GenAPI::new()
            .opcode(&[0x55])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFIXUPIMMSS => GenAPI::new()
            .opcode(&[0x55])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VFMADD132PH => GenAPI::new()
            .opcode(&[0x98])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD213PH => GenAPI::new()
            .opcode(&[0xA8])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD231PH => GenAPI::new()
            .opcode(&[0xB8])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD132PH => GenAPI::new()
            .opcode(&[0x9C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD213PH => GenAPI::new()
            .opcode(&[0xAC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD231PH => GenAPI::new()
            .opcode(&[0xBC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD132SH => GenAPI::new()
            .opcode(&[0x99])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD213SH => GenAPI::new()
            .opcode(&[0xA9])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADD231SH => GenAPI::new()
            .opcode(&[0xB9])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD132SH => GenAPI::new()
            .opcode(&[0x9D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD213SH => GenAPI::new()
            .opcode(&[0xAD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMADD231SH => GenAPI::new()
            .opcode(&[0xBD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDSUB132PH => GenAPI::new()
            .opcode(&[0x96])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDSUB213PH => GenAPI::new()
            .opcode(&[0xA6])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMADDSUB231PH => GenAPI::new()
            .opcode(&[0xB6])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VFMSUB132PH => GenAPI::new()
            .opcode(&[0x9A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUB213PH => GenAPI::new()
            .opcode(&[0xAA])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUB231PH => GenAPI::new()
            .opcode(&[0xBA])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB132PH => GenAPI::new()
            .opcode(&[0x9E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB213PH => GenAPI::new()
            .opcode(&[0xAE])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB231PH => GenAPI::new()
            .opcode(&[0xBE])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VFMSUB132SH => GenAPI::new()
            .opcode(&[0x9B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUB213SH => GenAPI::new()
            .opcode(&[0xAB])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUB231SH => GenAPI::new()
            .opcode(&[0xBB])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB132SH => GenAPI::new()
            .opcode(&[0x9F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB213SH => GenAPI::new()
            .opcode(&[0xAF])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFNMSUB231SH => GenAPI::new()
            .opcode(&[0xBF])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUBADD132PH => GenAPI::new()
            .opcode(&[0x97])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUBADD213PH => GenAPI::new()
            .opcode(&[0xA7])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VFMSUBADD231PH => GenAPI::new()
            .opcode(&[0xB7])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VFPCLASSPD => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFPCLASSPH => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VFPCLASSPS => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VFPCLASSSD => GenAPI::new()
            .opcode(&[0x67])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VFPCLASSSH => GenAPI::new()
            .opcode(&[0x67])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VFPCLASSSS => GenAPI::new()
            .opcode(&[0x67])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VGETEXPPD => GenAPI::new()
            .opcode(&[0x42])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGETEXPPH => GenAPI::new()
            .opcode(&[0x42])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP6).vex_we(false)),
        Mnemonic::VGETEXPPS => GenAPI::new()
            .opcode(&[0x42])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGETEXPSH => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VGETEXPSS => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGETMANTPD => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VGETMANTPH => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VGETMANTPS => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VGETMANTSD => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VGETMANTSH => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VGETMANTSS => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTF32X4 => GenAPI::new()
            .opcode(&[0x18])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTF64X2 => GenAPI::new()
            .opcode(&[0x18])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VINSERTF32X8 => GenAPI::new()
            .opcode(&[0x1A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTF64X4 => GenAPI::new()
            .opcode(&[0x1A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VINSERTI32X4 => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTI64X2 => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VINSERTI32X8 => GenAPI::new()
            .opcode(&[0x3A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VINSERTI64X4 => GenAPI::new()
            .opcode(&[0x3A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VINSERTI128 => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VMASKMOVPS => {
            let mut api = GenAPI::new()
                .strict_pfx()
                .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false))
                .modrm(true, None, None);

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
                .modrm(true, None, None);

            api = if let Some(Operand::Mem(_)) = ins.dst() {
                api.opcode(&[0x2D]).ord(&[MODRM_RM, VEX_VVVV, MODRM_REG])
            } else {
                api.opcode(&[0x2F]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            };

            api
        }
        Mnemonic::VMAXPH => GenAPI::new()
            .opcode(&[0x5F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VMAXSH => GenAPI::new()
            .opcode(&[0x5F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VMINPH => GenAPI::new()
            .opcode(&[0x5D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VMINSH => GenAPI::new()
            .opcode(&[0x5D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        // Intel says that for: vmovsh xmm1 {k1}{z}, xmm2, xmm3 you can encode
        // with both: 0x10 and 0x11?
        Mnemonic::VMOVSH => {
            let mut api = GenAPI::new()
                .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false))
                .modrm(true, None, None);

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
                .modrm(true, None, None);
            api = if let Some(Operand::Mem(_)) = ins.dst() {
                api.opcode(&[0x7E]).ord(&[MODRM_RM, MODRM_REG])
            } else {
                api.opcode(&[0x6E]).ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            };

            api
        }
        Mnemonic::VMULPH => GenAPI::new()
            .opcode(&[0x59])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VMULSH => GenAPI::new()
            .opcode(&[0x59])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VP2INTERSECTD => GenAPI::new()
            .opcode(&[0x68])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VP2INTERSECTQ => GenAPI::new()
            .opcode(&[0x68])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBLENDD => GenAPI::new()
            .opcode(&[0x68])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBLENDMB => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBLENDMW => GenAPI::new()
            .opcode(&[0x66])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBLENDMD => GenAPI::new()
            .opcode(&[0x64])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBLENDMQ => GenAPI::new()
            .opcode(&[0x64])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBROADCASTB => GenAPI::new()
            .opcode(&[0x78])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTW => GenAPI::new()
            .opcode(&[0x79])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTD => GenAPI::new()
            .opcode(&[0x58])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTQ => GenAPI::new()
            .opcode(&[0x59])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .vex(
                VexDetails::new()
                    .pp(0x66)
                    .map_select(MAP38)
                    .vex_we(ins.needs_evex()),
            ),
        Mnemonic::VPBROADCASTI32X2 => GenAPI::new()
            .opcode(&[0x59])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTI128 => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTI32X4 => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTI64X2 => GenAPI::new()
            .opcode(&[0x5A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBROADCASTI32X8 => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPBROADCASTI64X4 => GenAPI::new()
            .opcode(&[0x5B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBROADCASTMB2Q => GenAPI::new()
            .opcode(&[0x2A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),
        Mnemonic::VPBROADCASTMW2D => GenAPI::new()
            .opcode(&[0x3A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPCMPB => GenAPI::new()
            .opcode(&[0x3F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPCMPUB => GenAPI::new()
            .opcode(&[0x3E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPCMPD => GenAPI::new()
            .opcode(&[0x1F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPCMPUD => GenAPI::new()
            .opcode(&[0x1E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPCMPQ => GenAPI::new()
            .opcode(&[0x1F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPCMPUQ => GenAPI::new()
            .opcode(&[0x1E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPCMPW => GenAPI::new()
            .opcode(&[0x3F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPCMPUW => GenAPI::new()
            .opcode(&[0x3E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPCOMPRESSB => GenAPI::new()
            .opcode(&[0x63])
            .modrm(true, None, None)
            .ord(if ins.dst().unwrap().is_mem() {
                &[MODRM_RM, MODRM_REG]
            } else {
                &[MODRM_REG, MODRM_RM]
            })
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPCOMPRESSW => GenAPI::new()
            .opcode(&[0x63])
            .modrm(true, None, None)
            .ord(if ins.dst().unwrap().is_mem() {
                &[MODRM_RM, MODRM_REG]
            } else {
                &[MODRM_REG, MODRM_RM]
            })
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPCOMPRESSD => GenAPI::new()
            .opcode(&[0x8B])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPCOMPRESSQ => GenAPI::new()
            .opcode(&[0x8B])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPCONFLICTD => GenAPI::new()
            .opcode(&[0xC4])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPCONFLICTQ => GenAPI::new()
            .opcode(&[0xC4])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPDPBSSD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBSSDS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBSUD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBSUDS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBUUD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBUUDS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBUSD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPBUSDS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWSSD => GenAPI::new()
            .opcode(&[0x52])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWSSDS => GenAPI::new()
            .opcode(&[0x53])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),

        Mnemonic::VPDPWUSD => GenAPI::new()
            .opcode(&[0xD2])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWUSDS => GenAPI::new()
            .opcode(&[0xD3])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWSUD => GenAPI::new()
            .opcode(&[0xD2])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWSUDS => GenAPI::new()
            .opcode(&[0xD3])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWUUD => GenAPI::new()
            .opcode(&[0xD2])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(MAP38).vex_we(false)),
        Mnemonic::VPDPWUUDS => GenAPI::new()
            .opcode(&[0xD3])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMB => GenAPI::new()
            .opcode(&[0x8D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMD => GenAPI::new()
            .opcode(&[0x36])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMW => GenAPI::new()
            .opcode(&[0x8D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMI2B => GenAPI::new()
            .opcode(&[0x75])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMI2W => GenAPI::new()
            .opcode(&[0x75])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMI2D => GenAPI::new()
            .opcode(&[0x76])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMI2Q => GenAPI::new()
            .opcode(&[0x76])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMI2PS => GenAPI::new()
            .opcode(&[0x77])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMI2PD => GenAPI::new()
            .opcode(&[0x77])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMPD => {
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                GenAPI::new()
                    .opcode(&[0x01])
                    .modrm(true, None, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .imm_atindex(2, 1)
                    .vex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true))
            } else {
                GenAPI::new()
                    .opcode(&[0x16])
                    .modrm(true, None, None)
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            }
        }
        Mnemonic::VPERMPS => GenAPI::new()
            .opcode(&[0x16])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMQ => {
            if let Some(Operand::Imm(_)) = ins.ssrc() {
                GenAPI::new()
                    .opcode(&[0x00])
                    .modrm(true, None, None)
                    .ord(&[MODRM_REG, MODRM_RM])
                    .imm_atindex(2, 1)
                    .vex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true))
            } else {
                GenAPI::new()
                    .opcode(&[0x36])
                    .modrm(true, None, None)
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true))
            }
        }
        Mnemonic::VPERMT2B => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMT2W => GenAPI::new()
            .opcode(&[0x7D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMT2D => GenAPI::new()
            .opcode(&[0x7E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMT2Q => GenAPI::new()
            .opcode(&[0x7E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPERMT2PS => GenAPI::new()
            .opcode(&[0x7F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPERMT2PD => GenAPI::new()
            .opcode(&[0x7F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPEXPANDB => GenAPI::new()
            .opcode(&[0x62])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPEXPANDW => GenAPI::new()
            .opcode(&[0x62])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPEXPANDD => GenAPI::new()
            .opcode(&[0x89])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPEXPANDQ => GenAPI::new()
            .opcode(&[0x89])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPLZCNTD => GenAPI::new()
            .opcode(&[0x44])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPLZCNTQ => GenAPI::new()
            .opcode(&[0x44])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMADD52LUQ => GenAPI::new()
            .opcode(&[0xB5])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMADD52HUQ => GenAPI::new()
            .opcode(&[0xB5])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMASKMOVD => {
            if let Some(Operand::Mem(_) | Operand::Symbol(_)) = ins.dst() {
                GenAPI::new()
                    .opcode(&[0x8E])
                    .modrm(true, None, None)
                    .ord(&[MODRM_RM, VEX_VVVV, MODRM_REG])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38))
                    .strict_pfx()
            } else {
                GenAPI::new()
                    .opcode(&[0x8C])
                    .modrm(true, None, None)
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38))
                    .strict_pfx()
            }
        }
        Mnemonic::VPMASKMOVQ => {
            if let Some(Operand::Mem(_) | Operand::Symbol(_)) = ins.dst() {
                GenAPI::new()
                    .opcode(&[0x8E])
                    .modrm(true, None, None)
                    .ord(&[MODRM_RM, VEX_VVVV, MODRM_REG])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                    .strict_pfx()
            } else {
                GenAPI::new()
                    .opcode(&[0x8C])
                    .modrm(true, None, None)
                    .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
                    .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true))
                    .strict_pfx()
            }
        }
        Mnemonic::VPMOVB2M => GenAPI::new()
            .opcode(&[0x29])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVW2M => GenAPI::new()
            .opcode(&[0x29])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMOVD2M => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVQ2M => GenAPI::new()
            .opcode(&[0x39])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMOVDB => GenAPI::new()
            .opcode(&[0x31])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSDB => GenAPI::new()
            .opcode(&[0x21])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSDB => GenAPI::new()
            .opcode(&[0x11])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVDW => GenAPI::new()
            .opcode(&[0x33])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSDW => GenAPI::new()
            .opcode(&[0x23])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSDW => GenAPI::new()
            .opcode(&[0x13])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMOVM2B => GenAPI::new()
            .opcode(&[0x28])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVM2W => GenAPI::new()
            .opcode(&[0x28])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),
        Mnemonic::VPMOVM2D => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVM2Q => GenAPI::new()
            .opcode(&[0x38])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(true)),

        Mnemonic::VPMOVQB => GenAPI::new()
            .opcode(&[0x32])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSQB => GenAPI::new()
            .opcode(&[0x22])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSQB => GenAPI::new()
            .opcode(&[0x12])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMOVQD => GenAPI::new()
            .opcode(&[0x35])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSQD => GenAPI::new()
            .opcode(&[0x25])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSQD => GenAPI::new()
            .opcode(&[0x15])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMOVQW => GenAPI::new()
            .opcode(&[0x34])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSQW => GenAPI::new()
            .opcode(&[0x24])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSQW => GenAPI::new()
            .opcode(&[0x14])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMOVWB => GenAPI::new()
            .opcode(&[0x30])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVSWB => GenAPI::new()
            .opcode(&[0x20])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),
        Mnemonic::VPMOVUSWB => GenAPI::new()
            .opcode(&[0x10])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP38).vex_we(false)),

        Mnemonic::VPMULTISHIFTQB => GenAPI::new()
            .opcode(&[0x83])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPOPCNTB => GenAPI::new()
            .opcode(&[0x54])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPOPCNTW => GenAPI::new()
            .opcode(&[0x54])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPOPCNTD => GenAPI::new()
            .opcode(&[0x55])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPOPCNTQ => GenAPI::new()
            .opcode(&[0x55])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),

        Mnemonic::VPROLD => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, Some(1), None)
            .ord(&[VEX_VVVV, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VPROLQ => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, Some(1), None)
            .ord(&[VEX_VVVV, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VPROLVD => GenAPI::new()
            .opcode(&[0x15])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VPROLVQ => GenAPI::new()
            .opcode(&[0x15])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),

        Mnemonic::VPRORD => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, Some(0), None)
            .ord(&[VEX_VVVV, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VPRORQ => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, Some(0), None)
            .ord(&[VEX_VVVV, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),
        Mnemonic::VPRORVD => GenAPI::new()
            .opcode(&[0x14])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(false)),
        Mnemonic::VPRORVQ => GenAPI::new()
            .opcode(&[0x14])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP0F).vex_we(true)),

        Mnemonic::VPSHLDW => GenAPI::new()
            .opcode(&[0x70])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPSHLDD => GenAPI::new()
            .opcode(&[0x71])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPSHLDQ => GenAPI::new()
            .opcode(&[0x71])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPSHRDW => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPSHRDD => GenAPI::new()
            .opcode(&[0x73])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPSHRDQ => GenAPI::new()
            .opcode(&[0x73])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),

        Mnemonic::VPSHLDVW => GenAPI::new()
            .opcode(&[0x70])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSHLDVD => GenAPI::new()
            .opcode(&[0x71])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSHLDVQ => GenAPI::new()
            .opcode(&[0x71])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSHRDVW => GenAPI::new()
            .opcode(&[0x72])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSHRDVD => GenAPI::new()
            .opcode(&[0x73])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSHRDVQ => GenAPI::new()
            .opcode(&[0x73])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSHUFBITQMB => GenAPI::new()
            .opcode(&[0x8F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSLLVW => GenAPI::new()
            .opcode(&[0x12])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSLLVD => GenAPI::new()
            .opcode(&[0x47])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPSLLVQ => GenAPI::new()
            .opcode(&[0x47])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),

        Mnemonic::VPSRAVW => GenAPI::new()
            .opcode(&[0x11])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSRAVD => GenAPI::new()
            .opcode(&[0x46])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPSRAVQ => GenAPI::new()
            .opcode(&[0x46])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),

        Mnemonic::VPSRLVW => GenAPI::new()
            .opcode(&[0x10])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSRLVD => GenAPI::new()
            .opcode(&[0x45])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPSRLVQ => GenAPI::new()
            .opcode(&[0x45])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),

        Mnemonic::VPTERNLOGD => GenAPI::new()
            .opcode(&[0x25])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTERNLOGQ => GenAPI::new()
            .opcode(&[0x25])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),

        Mnemonic::VPTESTMB => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTESTMW => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPTESTMD => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTESTMQ => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPTESTNMB => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTESTNMW => GenAPI::new()
            .opcode(&[0x26])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A).vex_we(true)),
        Mnemonic::VPTESTNMD => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A).vex_we(false)),
        Mnemonic::VPTESTNMQ => GenAPI::new()
            .opcode(&[0x27])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRANGEPS => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VRANGEPD => GenAPI::new()
            .opcode(&[0x50])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRANGESS => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VRANGESD => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),

        Mnemonic::VRCP14PS => GenAPI::new()
            .opcode(&[0x4C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRCP14PD => GenAPI::new()
            .opcode(&[0x4C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRCP14SS => GenAPI::new()
            .opcode(&[0x4D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRCP14SD => GenAPI::new()
            .opcode(&[0x4D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRCPPH => GenAPI::new()
            .opcode(&[0x4C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VRCPSH => GenAPI::new()
            .opcode(&[0x4D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VREDUCEPS => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VREDUCEPD => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VREDUCEPH => GenAPI::new()
            .opcode(&[0x56])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VREDUCESS => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VREDUCESD => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VREDUCESH => GenAPI::new()
            .opcode(&[0x57])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),

        Mnemonic::VRNDSCALEPS => GenAPI::new()
            .opcode(&[0x08])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VRNDSCALEPD => GenAPI::new()
            .opcode(&[0x09])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRNDSCALEPH => GenAPI::new()
            .opcode(&[0x08])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .imm_atindex(2, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),
        Mnemonic::VRNDSCALESS => GenAPI::new()
            .opcode(&[0x0A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VRNDSCALESD => GenAPI::new()
            .opcode(&[0x0B])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRNDSCALESH => GenAPI::new()
            .opcode(&[0x0A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().map_select(MAP3A).vex_we(false)),

        Mnemonic::VRSQRT14PS => GenAPI::new()
            .opcode(&[0x4E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRSQRT14PD => GenAPI::new()
            .opcode(&[0x4E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRSQRT14SS => GenAPI::new()
            .opcode(&[0x4F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRSQRT14SD => GenAPI::new()
            .opcode(&[0x4F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VRSQRTPH => GenAPI::new()
            .opcode(&[0x4E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VRSQRTSH => GenAPI::new()
            .opcode(&[0x4F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VSCALEFPS => GenAPI::new()
            .opcode(&[0x2C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCALEFPD => GenAPI::new()
            .opcode(&[0x2C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCALEFSS => GenAPI::new()
            .opcode(&[0x2D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCALEFSD => GenAPI::new()
            .opcode(&[0x2D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCALEFPH => GenAPI::new()
            .opcode(&[0x2C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),
        Mnemonic::VSCALEFSH => GenAPI::new()
            .opcode(&[0x2D])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP6).vex_we(false)),

        Mnemonic::VSHA512MSG1 => GenAPI::new()
            .opcode(&[0xCC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),
        Mnemonic::VSHA512MSG2 => GenAPI::new()
            .opcode(&[0xCD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),
        Mnemonic::VSHA512RNDS2 => GenAPI::new()
            .opcode(&[0xCB])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),

        Mnemonic::VSHUFF32X4 => GenAPI::new()
            .opcode(&[0x23])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VSHUFF64X2 => GenAPI::new()
            .opcode(&[0x23])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VSHUFI32X4 => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(false)),
        Mnemonic::VSHUFI64X2 => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .imm_atindex(3, 1)
            .evex(VexDetails::new().pp(0x66).map_select(MAP3A).vex_we(true)),
        Mnemonic::VSM3MSG1 => GenAPI::new()
            .opcode(&[0xDA])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().map_select(0x38).vex_we(false)),
        Mnemonic::VSM3MSG2 => GenAPI::new()
            .opcode(&[0xDA])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VSM3RNDS2 => GenAPI::new()
            .opcode(&[0xDE])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .imm_atindex(3, 1)
            .vex(VexDetails::new().pp(0x66).map_select(0x3A).vex_we(false)),
        Mnemonic::VSM4KEY4 => GenAPI::new()
            .opcode(&[0xDA])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF3).map_select(0x38).vex_we(false)),
        Mnemonic::VSM4RNDS4 => GenAPI::new()
            .opcode(&[0xDA])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0xF2).map_select(0x38).vex_we(false)),
        Mnemonic::VSQRTPH => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VSQRTSH => GenAPI::new()
            .opcode(&[0x51])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VSUBPH => GenAPI::new()
            .opcode(&[0x5C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::VSUBSH => GenAPI::new()
            .opcode(&[0x5C])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF3).map_select(MAP5).vex_we(false)),
        Mnemonic::VTESTPS => GenAPI::new()
            .opcode(&[0x0E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VTESTPD => GenAPI::new()
            .opcode(&[0x0F])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .strict_pfx()
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VUCOMISH => GenAPI::new()
            .opcode(&[0x2E])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().map_select(MAP5).vex_we(false)),
        Mnemonic::PREFETCHWT1 => GenAPI::new()
            .opcode(&[0x0F, 0x0D])
            .modrm(true, Some(2), None),
        Mnemonic::V4FMADDPS => GenAPI::new()
            .opcode(&[0x9A])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::V4FNMADDPS => GenAPI::new()
            .opcode(&[0xAA])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::V4FMADDSS => GenAPI::new()
            .opcode(&[0xAB])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::V4FNMADDSS => GenAPI::new()
            .opcode(&[0xAB])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VEXP2PS => GenAPI::new()
            .opcode(&[0xC8])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VEXP2PD => GenAPI::new()
            .opcode(&[0xC8])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VP4DPWSSDS => GenAPI::new()
            .opcode(&[0x53])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),
        Mnemonic::VP4DPWSSD => GenAPI::new()
            .opcode(&[0x52])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0xF2).map_select(MAP38).vex_we(false)),

        Mnemonic::VRCP28PD => GenAPI::new()
            .opcode(&[0xCA])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRCP28SD => GenAPI::new()
            .opcode(&[0xCB])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRCP28PS => GenAPI::new()
            .opcode(&[0xCA])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRCP28SS => GenAPI::new()
            .opcode(&[0xCB])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),

        Mnemonic::VRSQRT28PD => GenAPI::new()
            .opcode(&[0xCC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRSQRT28SD => GenAPI::new()
            .opcode(&[0xCD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VRSQRT28PS => GenAPI::new()
            .opcode(&[0xCC])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VRSQRT28SS => GenAPI::new()
            .opcode(&[0xCD])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPGATHERDD => GenAPI::new()
            .opcode(&[0x90])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPGATHERDQ => GenAPI::new()
            .opcode(&[0x90])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),
        Mnemonic::VPGATHERQD => GenAPI::new()
            .opcode(&[0x91])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VPGATHERQQ => GenAPI::new()
            .opcode(&[0x91])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),
        Mnemonic::VSCATTERDPS => GenAPI::new()
            .opcode(&[0xA2])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERDPD => GenAPI::new()
            .opcode(&[0xA2])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCATTERQPS => GenAPI::new()
            .opcode(&[0xA3])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERQPD => GenAPI::new()
            .opcode(&[0xA3])
            .modrm(true, None, None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERPF0DPS => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(1), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGATHERPF0DPD => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(1), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERPF0QPS => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(1), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGATHERPF0QPD => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(1), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERPF1DPS => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(2), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGATHERPF1DPD => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(2), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERPF1QPS => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(2), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VGATHERPF1QPD => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(2), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),

        Mnemonic::VSCATTERPF0DPS => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(5), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERPF0DPD => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(5), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCATTERPF0QPS => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(5), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERPF0QPD => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(5), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCATTERPF1DPS => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(6), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERPF1DPD => GenAPI::new()
            .opcode(&[0xC6])
            .modrm(true, Some(6), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VSCATTERPF1QPS => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(6), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VSCATTERPF1QPD => GenAPI::new()
            .opcode(&[0xC7])
            .modrm(true, Some(6), None)
            .ord(&[MODRM_RM, MODRM_REG])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGETEXPSD => GenAPI::new()
            .opcode(&[0x43])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, VEX_VVVV, MODRM_RM])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VGATHERDPS => GenAPI::new()
            .opcode(&[0x92])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VGATHERDPD => GenAPI::new()
            .opcode(&[0x92])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(true)),
        Mnemonic::VGATHERQPS => GenAPI::new()
            .opcode(&[0x93])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .vex(VexDetails::new().pp(0x66).map_select(0x38).vex_we(false)),
        Mnemonic::VGATHERQPD => GenAPI::new()
            .opcode(&[0x93])
            .modrm(true, None, None)
            .ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),

        Mnemonic::VPSCATTERDD => GenAPI::new()
            .opcode(&[0xA0])
            .modrm(true, None, None)
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSCATTERDQ => GenAPI::new()
            .opcode(&[0xA0])
            .modrm(true, None, None)
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
        Mnemonic::VPSCATTERQD => GenAPI::new()
            .opcode(&[0xA1])
            .modrm(true, None, None)
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(false)),
        Mnemonic::VPSCATTERQQ => GenAPI::new()
            .opcode(&[0xA1])
            .modrm(true, None, None)
            .evex(VexDetails::new().pp(0x66).map_select(MAP38).vex_we(true)),
    }
}

// #  #   #   ####  #####  #####  #   #   ####  #####  #   ###   #   #   ####
// #  ##  #  #        #    #   #  #   #  #        #    #  #   #  ##  #  #
// #  # # #   ###     #    ####   #   #  #        #    #  #   #  # # #   ###
// #  #  ##      #    #    #   #  #   #  #        #    #  #   #  #  ##      #
// #  #   #  ####     #    #   #   ###    ####    #    #   ###   #   #  ####
// (Instructions)

fn ins_kmov(ins: &Instruction) -> GenAPI {
    let mut api = GenAPI::new()
        .modrm(true, None, None)
        .ord(&[MODRM_REG, MODRM_RM]);
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
    let mut api = GenAPI::new().rex(true);
    match ins.size() {
        Size::Byte => {
            api = api.opcode(&[0x86]);
            if let Some(Operand::Register(_)) = ins.dst() {
                api = api.ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
            } else {
                api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV])
            }
            api = api.modrm(true, None, None);
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
                    api = api.modrm(true, None, None);
                    api = api.ord(&[MODRM_REG, MODRM_RM, VEX_VVVV])
                }
            } else if let Some(Operand::Register(_)) = ins.src() {
                api = api.opcode(&[0x87]);
                api = api.modrm(true, None, None);
                api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV]);
            } else {
                api = api.opcode(&[0x87]);
                api = api.modrm(true, None, None);
                api = api.ord(&[MODRM_RM, MODRM_REG, VEX_VVVV]);
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
    if let Some(Operand::Imm(_) | Operand::Symbol(_)) = ins.ssrc() {
        api = api.opcode(opc_imm).imm_atindex(2, 1);
    } else {
        api = api.opcode(opc_rm);
    }
    api
}

fn ins_bt(ins: &Instruction, opc_noimm: &[u8], opc_imm: &[u8], _: u8, modrm: u8) -> GenAPI {
    let mut api = GenAPI::new().rex(true);
    if let Some(Operand::Imm(_) | Operand::Symbol(_)) = ins.src() {
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
                GenAPI::new().opcode(&[0x58 + r.to_byte()]).rex(true)
            }
        }
        Operand::Mem(_) | Operand::Symbol(_) => {
            GenAPI::new()
                .opcode(&[0x8F])
                .rex(true)
                .modrm(true, None, Some(0))
        }
        _ => invalid(33),
    }
}

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
                GenAPI::new().opcode(&[0x50 + r.to_byte()]).rex(true)
            }
        }
        Operand::Imm(nb) => match nb.size() {
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
                GenAPI::new()
                    .opcode(&[0xFF])
                    .modrm(true, Some(6), None)
                    .rex(true)
            } else {
                GenAPI::new().opcode(&[0x68]).imm_atindex(0, 4)
            }
        }
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
    if let Operand::Register(r) = dst {
        let p = r.purpose();
        if p.is_dbg() {
            GenAPI::new()
                .opcode(&[0x0F, 0x23])
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                .rex(true)
        } else if p.is_sgmnt() {
            match src {
                Operand::Register(_) | Operand::Mem(_) => GenAPI::new()
                    .opcode(&[0x8E])
                    .modrm(true, None, None)
                    .rex(true),
                _ => invalid(25),
            }
        } else if p.is_ctrl() {
            GenAPI::new()
                .opcode(&[0x0F, 0x22])
                .modrm(true, None, None)
                .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                .rex(true)
        } else {
            match src {
                Operand::Imm(i) => {
                    if i.size() == Size::Qword {
                        GenAPI::new()
                            .opcode(&[0xB8 + r.to_byte()])
                            .rex(true)
                            .imm_atindex(1, 8)
                    } else if r.get_ext_bits()[1] && r.size() == Size::Qword {
                        GenAPI::new()
                            .opcode(&[0xC7])
                            .rex(true)
                            .modrm(true, Some(0), None)
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
                            .rex(true)
                    }
                }
                Operand::Register(r) => {
                    let p = r.purpose();
                    if p.is_sgmnt() {
                        GenAPI::new()
                            .opcode(&[0x8C])
                            .modrm(true, None, None)
                            .rex(true)
                    } else if p.is_ctrl() {
                        GenAPI::new()
                            .opcode(&[0x0F, 0x20])
                            .modrm(true, None, None)
                            .rex(true)
                    } else if p.is_dbg() {
                        GenAPI::new()
                            .opcode(&[0x0F, 0x21])
                            .modrm(true, None, None)
                            .ord(&[MODRM_RM, MODRM_REG])
                            .rex(true)
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
                        GenAPI::new()
                            .opcode(&[opc])
                            .modrm(true, None, None)
                            .rex(true)
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
                            .modrm(true, None, None)
                            .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                            .rex(true)
                    } else if s.reltype().unwrap_or(RelType::REL32).size() == 8 {
                        GenAPI::new()
                            .opcode(&[0xB8 + r.to_byte()])
                            .rex(true)
                            .imm_atindex(1, 8)
                    } else if r.get_ext_bits()[1] && r.size() == Size::Qword {
                        GenAPI::new()
                            .opcode(&[0xC7])
                            .rex(true)
                            .imm_atindex(1, 4)
                            .modrm(true, Some(0), None)
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
                            .rex(true)
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
                        .modrm(true, None, None)
                        .ord(&[OpOrd::MODRM_REG, OpOrd::MODRM_RM])
                        .rex(true)
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
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, None, None)
                    .rex(true)
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
        (Operand::Register(dstr), Operand::Symbol(s)) => {
            if s.is_deref() {
                let opc = match dstr.size() {
                    Size::Byte => opc[7],
                    Size::Word | Size::Dword | Size::Qword => opc[6],
                    _ => invalid(17),
                };
                GenAPI::new()
                    .opcode(&[opc])
                    .modrm(true, None, None)
                    .rex(true)
            } else {
                let srci = ins.src().unwrap().size();
                if let Size::Dword | Size::Word = srci {
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
                        if srci == Size::Byte {
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
        }
        (Operand::Register(dstr), Operand::Imm(_)) => {
            let srci = ins.src().unwrap().size();
            if let Size::Dword | Size::Word = srci {
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
                    if srci == Size::Byte {
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
            let size = if let (Size::Word | Size::Byte, Size::Word) = (srci, dstm) {
                2
            } else if let (Size::Byte, Size::Dword) = (srci, dstm) {
                4
            } else if srci != Size::Byte {
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
        (Operand::Register(r), Operand::Mem(_) | Operand::Register(_)) => {
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
        (Operand::Mem(_) | Operand::Symbol(_), Operand::Register(_)) => {
            let opc = match ins.dst().unwrap().size() {
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

    match (dst, &src) {
        (Operand::Register(dstr), Operand::Imm(_) | Operand::Symbol(_)) => {
            let srci = ins.src().unwrap().size();
            if let Size::Dword | Size::Word = srci {
                if let Register::RAX | Register::EAX = dstr {
                    return GenAPI::new()
                        .opcode(&[0x3D])
                        .imm_atindex(1, 4)
                        .rex(true)
                        .ord(&[]);
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
                    if srci == Size::Byte {
                        0x83
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
                .modrm(true, Some(7), None)
                .rex(true)
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
                .modrm(true, None, None)
                .ord(&[MODRM_REG, MODRM_RM])
                .rex(true)
        }
        (Operand::Mem(m), Operand::Register(_)) => {
            let opc = match m.size() {
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

    match (&dst, src) {
        (Operand::Register(dstr), Operand::Imm(_) | Operand::Symbol(_)) => {
            let sz = ins.src().unwrap().size();
            if let Size::Dword | Size::Word = sz {
                if let &Register::RAX | &Register::EAX = dstr {
                    return GenAPI::new().opcode(&[0xA9]).imm_atindex(1, 4).rex(true);
                } else if let &Register::AX = dstr {
                    return GenAPI::new().opcode(&[0xA9]).imm_atindex(1, 2).rex(true);
                }
            }
            if let &Register::AL = dstr {
                return GenAPI::new().opcode(&[0xA8]).imm_atindex(1, 1).rex(true);
            } else if let &Register::AX = dstr {
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
                .modrm(true, Some(0), None)
                .rex(true)
                .imm_atindex(1, size)
        }
        (Operand::Register(_) | Operand::Mem(_) | Operand::Symbol(_), Operand::Register(_)) => {
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
        Some(_) => match ins.get(2) {
            Some(Operand::Imm(_)) => {
                let (opc, size) = match ins.get(2).unwrap().size() {
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
        Operand::Symbol(s) => {
            let sz = s.reltype().unwrap_or(RelType::REL32).size();
            let base = match sz {
                1 => opc[2].to_vec(),
                _ => opc[0].to_vec(),
            };
            GenAPI::new().opcode(&base).imm_atindex(0, 0)
        }
        Operand::Register(_) | Operand::Mem(_) => GenAPI::new()
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

fn ins_shrtjmp(_: &Instruction, opc: Vec<u8>) -> GenAPI {
    GenAPI::new().opcode(&opc).imm_atindex(0, 1)
}

// ==============================
// Utils

fn invalid(ctx: i32) -> ! {
    panic!("Unexpected thing that should not happen - code {ctx}")
}
