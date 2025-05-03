// rasmx86_64 - src/core/modrm.rs
// ------------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::{Instruction as Ins, Operand as Op},
    mem::Mem,
    num::Number,
};

// man, i love pattern matching and how readable it is...
pub fn gen_modrm(ins: &Ins, reg: Option<u8>, rm: Option<u8>, rev: bool) -> u8 {
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

    let mut rev = rev;

    let reg = if let Some(reg) = reg {
        reg
    } else {
        if rev {
            gen_rmreg(ins.dst())
        } else {
            if let Some(Op::Mem(_) | Op::Segment(_)) = ins.src() {
                rev = true;
                gen_rmreg(ins.dst())
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
                if rev {
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
        Op::DbgReg(r) | Op::CtrReg(r) | Op::Reg(r) | Op::Mem(Mem::Direct(r, _)) => r.to_byte(),
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
