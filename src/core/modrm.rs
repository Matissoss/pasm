// pasm - src/core/modrm.rs
// ------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::api;
use crate::shr::ast::{Instruction, Operand};

pub fn modrm(ins: &Instruction, ctx: &api::GenAPI) -> u8 {
    let [mut dst, mut src, _] = ctx.get_ord_oprs(ins);

    // fallback to default
    if let (None, None) = (dst, src) {
        dst = ins.dst();
        src = ins.src();
    }

    let (mut reg, mut rm) = ctx.get_modrm().deserialize();
    let mut mod_ = if let Some(m) = ins.get_mem() {
        if let Some((_, sz)) = m.offset_x86() {
            if m.is_riprel() {
                0b00
            } else {
                if sz == 1 {
                    0b01
                } else {
                    0b10
                }
            }
        } else {
            0b00
        }
    } else if let [Some(_), _] = ins.get_symbs() {
        0b00
    } else {
        0b11
    };

    if let Some(true) = ctx.get_flag(api::SET_MODRM) {
        mod_ = ctx.get_addt2() & 0b11;
    }

    if reg.is_none() {
        reg = Some(gen_rmreg(&src));
    }

    if rm.is_none() {
        rm = if ins.uses_sib() {
            Some(0b100)
        } else if ins.uses_rip() {
            Some(0b101)
        } else {
            Some(gen_rmreg(&dst))
        }
    }
    bmodrm(mod_, reg.unwrap_or(0), rm.unwrap_or(0))
}

const fn bmodrm(mod_: u8, reg: u8, rm: u8) -> u8 {
    (mod_ << 6) + (reg << 3) + rm
}

fn gen_rmreg(op: &Option<&Operand>) -> u8 {
    if op.is_none() {
        return 0;
    };
    match op.unwrap() {
        Operand::DbgReg(r) | Operand::CtrReg(r) | Operand::Reg(r) => r.to_byte(),
        Operand::Mem(m) => {
            if let Some(r) = m.base() {
                r.to_byte()
            } else {
                0
            }
        }
        Operand::SegReg(r) => r.to_byte(),
        _ => 0,
    }
}
