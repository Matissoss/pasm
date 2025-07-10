// pasm - src/core/evex.rs
// -----------------------
// made by matisoss
// licensed under MPL 2.0

use crate::core::api::*;

use crate::shr::{
    ast::{Instruction, Operand},
    size::Size,
};

const EVEX: u8 = 0x62;

// opcode maps
pub const MAP0F: u8 = 0b000; // 0x0F
pub const MAP38: u8 = 0b010; // 0x0F 0x38
pub const MAP3A: u8 = 0b011; // 0x0F 0x3A
pub const MAP4: u8 = 0b100;
pub const MAP5: u8 = 0b101;
pub const MAP6: u8 = 0b110;

pub fn evex(ctx: &GenAPI, ins: &Instruction) -> [u8; 4] {
    let [modrm_rm, modrm_reg, evex_vvvv] = ctx.get_ord_oprs(ins);

    let [[evex_r0, evex_r1], [_, _]] = ee_bits(&modrm_reg);
    let [[evex_b0, evex_b], [_, mut evex_x1]] = ee_bits(&modrm_rm);
    let [[evex_vd, _], [_, _]] = ee_bits(&evex_vvvv);

    if evex_b0 {
        evex_x1 = true;
    }

    let evex3 = {
        (ins.get_z() as u8) << 7
            | ((ins.size() == Size::Zword) as u8) << 6
            | ((ins.size() == Size::Yword) as u8) << 5
            | ((ins.get_sae() | ins.get_bcst()) as u8) << 4
            | (!evex_vd as u8) << 3
            | ins.get_mask().unwrap_or(0)
    };

    [
        EVEX,
        (!evex_r1 as u8) << 7
            | (!evex_x1 as u8) << 6
            | (!evex_b as u8) << 5
            | (!evex_r0 as u8) << 4
            | ctx.get_map_select().unwrap() & 0b111,
        (ctx.get_vex_we().unwrap() as u8) << 7
            | gen_evex4v(&evex_vvvv) << 3
            | 1 << 2
            | ctx.get_pp().unwrap(),
        evex3,
    ]
}

const fn andn(num: u8, bits: u8) -> u8 {
    !num & bits
}

#[allow(clippy::collapsible_match)]
fn gen_evex4v(op: &Option<Operand>) -> u8 {
    if let Some(o) = op {
        match o {
            Operand::Register(r) => {
                andn((r.get_ext_bits()[1] as u8) << 3 | r.to_byte(), 0b0000_1111)
            }
            _ => 0b1111,
        }
    } else {
        0b1111
    }
}

// extended bits
fn ee_bits(op: &Option<Operand>) -> [[bool; 2]; 2] {
    if let Some(op) = op {
        match op {
            Operand::Register(r) => [r.get_ext_bits(), [false; 2]],
            Operand::Mem(m) => {
                let mut base = [false; 2];
                if let Some(i) = m.base() {
                    base = i.get_ext_bits();
                }
                let mut idx = [false; 2];
                if let Some(i) = m.index() {
                    idx = i.get_ext_bits();
                }
                [base, idx]
            }
            _ => [[false; 2]; 2],
        }
    } else {
        [[false; 2]; 2]
    }
}
