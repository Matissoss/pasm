// rasmx86_64 - src/core/avx.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    core::api::*,
    core::comp::vex_gen_ins,
    shr::ast::{Instruction, Operand},
    shr::ins::Mnemonic as Ins,
    shr::size::Size,
};

pub fn avx_ins_movx(
    ins: &Instruction,
    opc_rx: &[u8],
    opc_xr: &[u8],
    modrm1: Option<u8>,
    pp: u8,
    map_select: u8,
    vex_we: bool,
) -> Vec<u8> {
    use OpOrd::*;
    let (modrm_reg_is_dst, opc) = match (ins.dst(), ins.src()) {
        (_, Some(Operand::Mem(_))) => (true, opc_rx),
        (Some(Operand::Mem(_)), _) => (false, opc_xr),
        (Some(Operand::Reg(r)), _) => {
            if r.size() == Size::Dword || r.size() == Size::Qword {
                if let Some(Operand::Reg(_)) = ins.src() {
                    (true, opc_rx)
                } else {
                    (true, opc_rx)
                }
            } else {
                (false, opc_xr)
            }
        }
        _ => (false, opc_xr),
    };

    let mut api = GenAPI::new().opcode(opc).modrm(true, modrm1, None).vex(
        VexDetails::new()
            .pp(pp)
            .map_select(map_select)
            .vex_we(vex_we),
    );
    if !modrm_reg_is_dst {
        api = api.ord(&[MODRM_REG, MODRM_RM]);
    } else {
        api = api.ord(&[MODRM_RM, MODRM_REG]);
    }
    api.assemble(ins, 64)
}

pub fn avx_ins_oopc(
    ins: &Instruction,
    opc: &[u8],
    modrm1: Option<u8>,
    pp: u8,
    map_select: u8,
    vex_we: bool,
) -> Vec<u8> {
    use OpOrd::*;
    let modrm_reg_is_dst = match (ins.dst(), ins.src()) {
        (_, Some(Operand::Mem(_))) => true,
        (Some(Operand::Mem(_)), _) => false,
        _ => true,
    };
    let mut api = GenAPI::new().opcode(opc).modrm(true, modrm1, None).vex(
        VexDetails::new()
            .pp(pp)
            .map_select(map_select)
            .vex_we(vex_we),
    );
    if !modrm_reg_is_dst {
        api = api.ord(&[MODRM_REG, VEX_VVVV, MODRM_RM]);
    } else {
        api = api.ord(&[MODRM_RM, VEX_VVVV, MODRM_REG]);
    }
    api.assemble(ins, 64)
}
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
    let (mut modrm_reg_is_dst, opc) = match (ins.dst(), ins.src()) {
        (_, Some(Operand::Mem(_))) => (true, opc_rm),
        (Some(Operand::Mem(_)), _) => (false, opc_mr),
        _ => (true, opc_rm),
    };
    if matches!(ins.mnem, Ins::VINSERTF128) {
        modrm_reg_is_dst = false;
    }

    let imm = match ins.oprs.get(3) {
        Some(Operand::Imm(n)) => Some(vec![n.split_into_bytes()[0]]),
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
        _ => {
            if matches!(
                ins.mnem,
                Ins::VEXTRACTPS | Ins::VPEXTRB | Ins::VPEXTRD | Ins::VPEXTRQ
            ) {
                (false, opc_rm)
            } else {
                (true, opc_rm)
            }
        }
    };
    let imm = match ins.oprs.get(2) {
        Some(Operand::Imm(n)) => Some(vec![n.split_into_bytes()[0]]),
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
pub fn avx_ins_shift(
    ins: &Instruction,
    opc_noimm: &[u8],
    opc_imm: &[u8],
    modrm1: Option<u8>,
    pp: u8,
    map_select: u8,
    vex_we: bool,
) -> Vec<u8> {
    let (modrm_reg_is_dst, opc) = match ins.src2() {
        Some(Operand::Imm(_)) => (true, opc_imm),
        _ => (true, opc_noimm),
    };
    let imm = match ins.oprs.get(2) {
        Some(Operand::Imm(n)) => Some(vec![n.split_into_bytes()[0]]),
        _ => None,
    };

    vex_gen_ins(
        ins,
        opc,
        (true, if imm.is_some() { modrm1 } else { None }),
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
        Some(Operand::Reg(r)) => Some(vec![((r.needs_rex() as u8) << 3 | r.to_byte()) << 4]),
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
