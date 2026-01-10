// pasm - src/core/modrm.rs
// ------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::api;
use crate::shr::{instruction::Operand, reg::Register, size::Size};

//          aka modrm_rm                aka modrm_reg
pub fn modrm(dst: &Option<Operand>, src: &Option<Operand>, ctx: &api::GenAPI) -> u8 {
    let reg = ctx.get_modrm().deserialize();
    let mut mod_ = if let Some(Operand::Mem(m)) = dst {
        if let Some((_, sz)) = m.offset_x86() {
            if m.is_riprel() {
                0b00
            } else if sz == 1 {
                0b01
            } else {
                0b10
            }
        } else {
            0b00
        }
    } else if let Some(Operand::Symbol(_)) = dst {
        0b00
    } else {
        0b11
    };

    if ctx.flags.at(api::SET_MODRM) {
        mod_ = ctx.get_addt2() & 0b11;
    }

    let reg = if reg.is_none() {
        gen_rmreg(src)
    } else {
        reg.unwrap_or(0)
    };

    let rm = {
        if let Some(Operand::Mem(m)) = dst {
            if m.addrsize() == Size::Word {
                match (m.base(), m.index()) {
                    (Some(Register::BX), Some(Register::SI)) => 0b000,
                    (Some(Register::BX), Some(Register::DI)) => 0b001,
                    (Some(Register::BP), Some(Register::SI)) => 0b010,
                    (Some(Register::BP), Some(Register::DI)) => 0b011,
                    (Some(Register::SI), None) => 0b100,
                    (Some(Register::DI), None) => 0b101,
                    (Some(Register::BX), None) => 0b111,
                    (None, None) => {
                        if m.offset_x86().is_some() {
                            0b101
                        } else {
                            0b000
                        }
                    }
                    _ => 0b000,
                }
            } else if m.is_sib() {
                0b100
            } else if m.is_riprel() {
                0b101
            } else {
                gen_rmreg(dst)
            }
        } else if let Some(Operand::Symbol(s)) = dst {
            if s.is_deref() {
                0b101
            } else {
                gen_rmreg(dst)
            }
        } else {
            gen_rmreg(dst)
        }
    };
    bmodrm(mod_, reg, rm)
}

#[inline(always)]
const fn bmodrm(mod_: u8, reg: u8, rm: u8) -> u8 {
    (mod_ << 6) + (reg << 3) + rm
}

#[inline(always)]
fn gen_rmreg(op: &Option<Operand>) -> u8 {
    match op {
        Some(Operand::Register(r)) => r.to_byte(),
        Some(Operand::Mem(m)) => {
            if let Some(r) = m.base() {
                r.to_byte()
            } else {
                0
            }
        }
        _ => 0,
    }
}
