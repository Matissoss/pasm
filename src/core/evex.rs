// pasm - src/core/evex.rs
// -----------------------
// made by matisoss
// licensed under MPL 2.0

use crate::core::api::*;
use crate::shr::ast::Instruction;

pub const EVEX: u8 = 0x62;

// opcode maps
pub const MAP1: u8 = 0b000; // 0x0F
pub const MAP2: u8 = 0b001; // 0x0F 0x38
pub const MAP3: u8 = 0b010; // 0x0F 0x3A
pub const MAP4: u8 = 0b100;
pub const MAP5: u8 = 0b101;
pub const MAP6: u8 = 0b110;

pub fn evex(ctx: &GenAPI, ins: &Instruction) -> [u8; 4] {
    let [_modrm_rm, _modrm_reg, _evex_vvvv] = ctx.get_ord_oprs(ins);
    [EVEX, 0, 0, 0]
}
