// pasm - src/core/vex.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::api;
use crate::shr::ast::{IVariant, Instruction, Operand};

const TWO_BYTE_PFX: u8 = 0xC5;
const THREE_BYTE_PFX: u8 = 0xC4;

pub fn vex(ins: &Instruction, ctx: &api::GenAPI) -> Option<Vec<u8>> {
    let [mut modrm_rm, mut modrm_reg, mut vex_opr] = ctx.get_ord_oprs(ins);

    if let (None, None, None) = (&modrm_reg, &modrm_rm, &vex_opr) {
        modrm_reg = ins.src2();
        modrm_rm = ins.dst();
        vex_opr = ins.src();
    }

    let vvvv = gen_vex4v(&vex_opr);
    let pp = ctx.get_pp().unwrap();
    let map_select = ctx.get_map_select().unwrap();
    let vex_we = ctx.get_vex_we().unwrap();

    let tmp = ins.which_variant() == IVariant::YMM;
    let vlength = {
        if let Some(mg) = ctx.get_vex_vlength() {
            if let Some(ob) = mg.get() {
                ob as u8
            } else {
                tmp as u8
            }
        } else {
            tmp as u8
        }
    };

    let vex_b = needs_vex3(&modrm_rm);
    let vex_r = needs_vex3(&modrm_reg).0;

    if (vex_b.0 || vex_b.1) || (map_select == 0b00011 || map_select == 0b00010) || vex_we {
        Some(vec![
            THREE_BYTE_PFX,
            (((!vex_r) as u8) << 7
                | andn(vex_b.1 as u8, 0b0000_0001) << 6
                | (andn(vex_b.0 as u8, 0b0000_0001) << 5 | map_select)),
            ((vex_we as u8) << 7 | vvvv << 3 | vlength << 2 | pp),
        ])
    } else {
        Some(vec![
            TWO_BYTE_PFX,
            ((((!vex_r) as u8) << 7) | vvvv << 3 | vlength << 2 | pp),
        ])
    }
}
fn needs_vex3(op: &Option<Operand>) -> (bool, bool) {
    if let Some(op) = op {
        match op {
            Operand::Register(r) => {
                if r.get_ext_bits()[1] {
                    return (true, false);
                }
            }
            Operand::Mem(m) => {
                let rr = m.needs_rex();
                if rr.0 || rr.1 {
                    return rr;
                }
            }
            _ => {}
        }
    }
    (false, false)
}

const fn andn(num: u8, bits: u8) -> u8 {
    !num & bits
}

// VEX.vvvv field
#[allow(clippy::collapsible_match)]
fn gen_vex4v(op: &Option<Operand>) -> u8 {
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
