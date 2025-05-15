// rasmx86_64 - src/core/avx.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::{
    core::comp::vex_gen_ins,
    shr::ast::{Instruction, Operand},
};

pub fn avx_ins(
    ins: &Instruction,
    opc_rm: &[u8],
    opc_mr: &[u8],
    modrm1: Option<u8>,
    pp: u8,
    map_select: u8,
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
    )
}
