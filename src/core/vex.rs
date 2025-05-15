// rasmx86_64 - src/core/vex.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::{IVariant, Instruction, Operand},
    //reg::Register,
    //mem::Mem
};

const TWO_BYTE_PFX: u8 = 0xC5;
const _THREE_BYTE_PFX: u8 = 0xC4;

pub fn gen_vex(
    ins: &Instruction,
    pp: u8,
    _map_select: u8,
    modrm_reg_is_dst: bool,
) -> Option<Vec<u8>> {
    let ssrc = gen_vex4v(ins.src2());
    let pp = match pp {
        0x66 => 0b01,
        0xF3 => 0b10,
        0xF2 => 0b11,
        _ => 0b00,
    };
    let vlength = (ins.which_variant() == IVariant::YMM) as u8;
    let (vexr, vex3) = if modrm_reg_is_dst {
        (!needs_vex3(ins.dst()), needs_vex3(ins.src()))
    } else {
        (!needs_vex3(ins.src()), needs_vex3(ins.dst()))
    };

    if vex3 {
        todo!("VEX3 prefix needs to be done :)")
    } else {
        Some(vec![
            TWO_BYTE_PFX,
            (((vexr as u8) << 7) | ssrc << 3 | vlength << 2 | pp),
        ])
    }
}

fn needs_vex3(op: Option<&Operand>) -> bool {
    if let Some(op) = op {
        match op {
            Operand::Reg(r) => {
                if r.needs_rex() {
                    return true;
                }
            }
            Operand::Mem(m) => {
                let rr = m.needs_rex();
                if rr.0 || rr.1 {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

// VEX.vvvv field
#[allow(clippy::collapsible_match)]
fn gen_vex4v(op: Option<&Operand>) -> u8 {
    if let Some(o) = op {
        match o {
            Operand::Reg(r) => (!(r.needs_rex() as u8) << 3) | !r.to_byte(),
            _ => 0b1111,
        }
    } else {
        0b1111
    }
}
