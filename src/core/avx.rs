// rasmx86_64 - src/core/avx.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    core::comp::vex_gen_ins,
    shr::ast::{Instruction, Operand},
    shr::num::Number,
};

pub fn avx_ins(
    ins: &Instruction,
    opc_rm: &[u8],
    opc_mr: &[u8],
    modrm1: Option<u8>,
    pp: u8,
    map_select: u8,
    vex_we: bool,
) -> Vec<u8> {
    let (modrm_reg_is_dst, opc) = match (ins.dst(), ins.src()) {
        (_, Some(Operand::Mem(_))) => (true, opc_rm),
        (Some(Operand::Mem(_)), _) => (false, opc_mr),
        _ => (true, opc_rm),
    };

    vex_gen_ins(
        ins,
        opc,
        (true, modrm1),
        None,
        modrm_reg_is_dst,
        pp,
        map_select,
        vex_we,
    )
}

// w immediate at index 3 (4th operand)
pub fn avx_ins_wimm3(
    ins: &Instruction,
    opc_rm: &[u8],
    opc_mr: &[u8],
    modrm1: Option<u8>,
    pp: u8,
    map_select: u8,
    vex_we: bool,
) -> Vec<u8> {
    let (modrm_reg_is_dst, opc) = match (ins.dst(), ins.src()) {
        (_, Some(Operand::Mem(_))) => (true, opc_rm),
        (Some(Operand::Mem(_)), _) => (false, opc_mr),
        _ => (true, opc_rm),
    };
    let imm = match ins.oprs.get(3) {
        Some(Operand::Imm(Number::UInt8(n))) => Some(vec![*n]),
        _ => None,
    };

    vex_gen_ins(
        ins,
        opc,
        (true, modrm1),
        imm,
        modrm_reg_is_dst,
        pp,
        map_select,
        vex_we,
    )
}
// w immediate at index 2 (3th operand)
pub fn avx_ins_wimm2(
    ins: &Instruction,
    opc_rm: &[u8],
    opc_mr: &[u8],
    modrm1: Option<u8>,
    pp: u8,
    map_select: u8,
    vex_we: bool,
) -> Vec<u8> {
    let (modrm_reg_is_dst, opc) = match (ins.dst(), ins.src()) {
        (_, Some(Operand::Mem(_))) => (true, opc_rm),
        (Some(Operand::Mem(_)), _) => (false, opc_mr),
        _ => (true, opc_rm),
    };
    let imm = match ins.oprs.get(2) {
        Some(Operand::Imm(Number::UInt8(n))) => Some(vec![*n]),
        _ => None,
    };

    vex_gen_ins(
        ins,
        opc,
        (true, modrm1),
        imm,
        modrm_reg_is_dst,
        pp,
        map_select,
        vex_we,
    )
}

// RVMR instructions - VBLENDVPS/-D
pub fn avx_ins_rvmr(
    ins: &Instruction,
    opc_rm: &[u8],
    opc_mr: &[u8],
    modrm1: Option<u8>,
    pp: u8,
    map_select: u8,
    vex_we: bool,
) -> Vec<u8> {
    let (modrm_reg_is_dst, opc) = match (ins.dst(), ins.src()) {
        (_, Some(Operand::Mem(_))) => (true, opc_rm),
        (Some(Operand::Mem(_)), _) => (false, opc_mr),
        _ => (true, opc_rm),
    };
    let imm = match ins.oprs.get(3) {
        Some(Operand::Reg(r)) => Some(vec![((r.needs_rex() as u8) << 3 | r.to_byte())]),
        _ => None,
    };

    vex_gen_ins(
        ins,
        opc,
        (true, modrm1),
        imm,
        modrm_reg_is_dst,
        pp,
        map_select,
        vex_we,
    )
}
