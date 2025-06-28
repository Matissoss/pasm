// pasm - src/core/evex.rs
// -----------------------
// made by matisoss
// licensed under MPL 2.0

use crate::core::api::*;

use crate::shr::{
    ast::{Instruction, Operand},
    size::Size,
};

pub const EVEX: u8 = 0x62;

// opcode maps
pub const MAP1: u8 = 0b000; // 0x0F
pub const MAP2: u8 = 0b001; // 0x0F 0x38
pub const MAP3: u8 = 0b010; // 0x0F 0x3A
pub const MAP4: u8 = 0b100;
pub const MAP5: u8 = 0b101;
pub const MAP6: u8 = 0b110;

pub fn evex(ctx: &GenAPI, ins: &Instruction) -> [u8; 4] {
    let [_modrm_rm, modrm_reg, evex_vvvv] = ctx.get_ord_oprs(ins);

    let (evex_r0, evex_r1) = ee_bits(modrm_reg);

    [
        EVEX,
        (!evex_r0 as u8) << 7
            | 1 << 6
            | 1 << 5
            | (!evex_r1 as u8) << 4
            | ctx.get_map_select().unwrap() & 0b111,
        (ctx.get_vex_we().unwrap() as u8) << 7
            | gen_evex4v(evex_vvvv) << 3
            | 1 << 2
            | ctx.get_pp().unwrap(),
        (ins.get_z() as u8) << 7
            | ((ins.size() == Size::Zword) as u8) << 6
            | ((ins.size() == Size::Yword) as u8) << 5
            | (ins.get_bcst() as u8) << 4
            | 1 << 3
            | ins.get_mask().unwrap().to_byte(),
    ]
}

const fn andn(num: u8, bits: u8) -> u8 {
    !num & bits
}

#[allow(clippy::collapsible_match)]
fn gen_evex4v(op: Option<&Operand>) -> u8 {
    if let Some(o) = op {
        match o {
            Operand::Reg(r) => andn((r.needs_rex() as u8) << 3 | r.to_byte(), 0b0000_1111),
            _ => 0b1111,
        }
    } else {
        0b1111
    }
}

// extended bits
fn ee_bits(op: Option<&Operand>) -> (bool, bool) {
    if let Some(op) = op {
        match op {
            Operand::Reg(r) => (r.needs_evex(), r.needs_rex()),
            Operand::Mem(m) => {
                if let Some(idx) = m.index() {
                    (idx.needs_evex(), idx.needs_rex())
                } else {
                    (false, false)
                }
            }
            _ => (false, false),
        }
    } else {
        (false, false)
    }
}
