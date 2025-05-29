// rasmx86_64 - src/core/modrm.rs
// ------------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::core::api;
use crate::shr::{
    ast::{Instruction as Ins, Operand as Op},
    ins::Mnemonic,
    mem::Mem,
    num::Number,
    reg::Purpose as RPurpose,
};

type Instruction = Ins;
type Operand = Op;
pub fn modrm(ins: &Instruction, ctx: &api::GenAPI) -> u8 {
    use api::OpOrd::*;

    let ord = &ctx.get_ord()[0..3];
    let (dst, src) = match ord {
        [MODRM_REG, VEX_VVVV, MODRM_RM] => (ins.src2(), ins.dst()),
        [MODRM_RM, VEX_VVVV, MODRM_REG] => (ins.dst(), ins.src2()),
        [VEX_VVVV, MODRM_REG, _] => (None, ins.src()),
        [VEX_VVVV, MODRM_RM, _] => (ins.src(), None),
        [MODRM_REG, MODRM_RM, _] => (ins.src(), ins.dst()),
        [MODRM_RM, MODRM_REG, _] => (ins.dst(), ins.src()),
        _ => (ins.dst(), ins.src()),
    };

    let (mut reg, mut rm) = ctx.get_modrm().deserialize();
    let mod_ = {
        if let Some(sibidx) = ins.get_sib_idx() {
            match ins.oprs.get(sibidx).unwrap() {
                Operand::Mem(Mem::SIB(_, _, _, _) | Mem::Index(_, _, _)) => 0b00,
                Operand::Mem(Mem::SIBOffset(_, _, _, o, _) | Mem::IndexOffset(_, o, _, _)) => {
                    match o {
                        -127..255 => 0b01,
                        _ => 0b10,
                    }
                }
                _ => 0b11,
            }
        } else {
            match (dst, src) {
                (Some(Operand::Mem(Mem::Direct(_, _))), _)
                | (_, Some(Operand::Mem(Mem::Direct(_, _)))) => 0b00,
                (Some(Operand::Mem(Mem::Offset(_, o, _))), _)
                | (_, Some(Operand::Mem(Mem::Offset(_, o, _)))) => match o {
                    -127..=255 => 0b01,
                    _ => 0b10,
                },
                _ => 0b11,
            }
        }
    };

    if reg.is_none() {
        reg = Some(gen_rmreg(src));
    }

    if rm.is_none() {
        rm = if ins.uses_sib() {
            Some(0b100)
        } else {
            Some(gen_rmreg(dst))
        }
    }
    bmodrm(mod_, reg.unwrap_or(0), rm.unwrap_or(0))
}

const fn bmodrm(mod_: u8, reg: u8, rm: u8) -> u8 {
    (mod_ << 6) + (reg << 3) + rm
}

// man, i love pattern matching and how readable it is...
pub fn gen_modrm(ins: &Ins, reg: Option<u8>, rm: Option<u8>, modrm_reg_is_dst: bool) -> u8 {
    let mod_: u8 = {
        match (ins.dst(), ins.src()) {
            (Some(&Op::Mem(Mem::SIB(_, _, _, _))), _)
            | (_, Some(&Op::Mem(Mem::SIB(_, _, _, _))))
            | (Some(&Op::Mem(Mem::Direct(_, _))), _)
            | (Some(&Op::Mem(Mem::Index(_, _, _))), _)
            | (_, Some(&Op::Mem(Mem::Index(_, _, _))))
            | (_, Some(&Op::Mem(Mem::Direct(_, _)))) => 0b00,

            (Some(&Op::Mem(Mem::SIBOffset(_, _, _, o, _) | Mem::Offset(_, o, _))), _)
            | (Some(&Op::Mem(Mem::IndexOffset(_, o, _, _))), _)
            | (_, Some(&Op::Mem(Mem::IndexOffset(_, o, _, _))))
            | (_, Some(&Op::Mem(Mem::SIBOffset(_, _, _, o, _) | Mem::Offset(_, o, _)))) => {
                match Number::squeeze_i64(o as i64) {
                    Number::Int8(_) => 0b01,
                    _ => 0b10,
                }
            }
            (Some(&Op::Segment(s)), _) | (_, Some(&Op::Segment(s))) => match s.address {
                Mem::SIB(_, _, _, _) | Mem::Index(_, _, _) | Mem::Direct(_, _) => 0b00,
                Mem::SIBOffset(_, _, _, o, _)
                | Mem::Offset(_, o, _)
                | Mem::IndexOffset(_, o, _, _) => match Number::squeeze_i64(o as i64) {
                    Number::Int8(_) => 0b01,
                    _ => 0b10,
                },
            },
            _ => 0b11,
        }
    };

    let mut modrm_reg_is_dst = modrm_reg_is_dst;

    let reg = if let Some(reg) = reg {
        reg
    } else {
        if matches!(
            ins.mnem,
            Mnemonic::PEXTRB | Mnemonic::PEXTRD | Mnemonic::PEXTRQ | Mnemonic::VINSERTF128
        ) {
            gen_rmreg(ins.src())
        } else if modrm_reg_is_dst {
            gen_rmreg(ins.dst())
        } else {
            if let Some(Op::Mem(_) | Op::Segment(_)) = ins.src() {
                modrm_reg_is_dst = true;
                gen_rmreg(ins.dst())
            } else if let Some(Op::Reg(r)) = ins.src() {
                let rp = r.purpose();
                if (rp == RPurpose::Mmx || rp == RPurpose::F128) && !ins.mnem.is_avx() {
                    modrm_reg_is_dst = true;
                    gen_rmreg(ins.dst())
                } else {
                    gen_rmreg(ins.src())
                }
            } else {
                gen_rmreg(ins.src())
            }
        }
    };
    let rm = {
        if ins.uses_sib() {
            0b100
        } else {
            if let Some(rm) = rm {
                rm
            } else {
                if matches!(
                    ins.mnem,
                    Mnemonic::PEXTRB | Mnemonic::PEXTRD | Mnemonic::PEXTRQ | Mnemonic::VINSERTF128
                ) {
                    gen_rmreg(ins.dst())
                } else if modrm_reg_is_dst {
                    gen_rmreg(ins.src())
                } else {
                    gen_rmreg(ins.dst())
                }
            }
        }
    };

    (mod_ << 6) + (reg << 3) + rm
}

fn gen_rmreg(op: Option<&Op>) -> u8 {
    if op.is_none() {
        return 0;
    };
    match op.unwrap() {
        Op::DbgReg(r)
        | Op::CtrReg(r)
        | Op::Reg(r)
        | Op::Mem(Mem::Direct(r, _) | Mem::Offset(r, _, _)) => r.to_byte(),
        Op::SegReg(r) => r.to_byte(),
        Op::Segment(s) => match s.address {
            Mem::Direct(r, _) => r.to_byte(),
            Mem::IndexOffset(_, _, _, _)
            | Mem::Index(_, _, _)
            | Mem::SIB(_, _, _, _)
            | Mem::SIBOffset(_, _, _, _, _) => 0b100,
            _ => 0,
        },
        _ => 0,
    }
}
