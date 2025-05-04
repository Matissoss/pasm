// rasmx86_64 - src/core/rex.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::{Instruction, Operand},
    ins::Mnemonic as Mnm,
    mem::Mem,
    size::Size,
};

fn needs_rex(ins: &Instruction) -> bool {
    let (size_d, size_s) = match (ins.dst(), ins.src()) {
        (Some(d), Some(s)) => (d.size(), s.size()),
        (Some(d), None) => (d.size(), Size::Unknown),
        (None, Some(s)) => (Size::Unknown, s.size()),
        _ => (Size::Unknown, Size::Unknown),
    };
    match (size_d, size_s) {
        (Size::Qword, Size::Qword) | (Size::Qword, _) | (_, Size::Qword) => {}
        _ => return false,
    }
    match &ins.mnem {
        Mnm::MOVMSKPD => true,
        Mnm::CVTSS2SI => true,
        Mnm::CVTSI2SS => true,
        Mnm::MOVQ => true,
        Mnm::MOV => {
            if let (Some(Operand::Reg(_)), Some(Operand::Reg(_)))
            | (Some(Operand::Mem(_) | Operand::Segment(_)), _)
            | (_, Some(Operand::Mem(_) | Operand::Segment(_))) = (ins.dst(), ins.src())
            {
                return true;
            }
            if let (Some(Operand::CtrReg(r) | Operand::DbgReg(r)), _)
            | (_, Some(Operand::CtrReg(r) | Operand::DbgReg(r))) = (ins.dst(), ins.src())
            {
                return r.needs_rex();
            }
            false
        }
        Mnm::SUB
        | Mnm::ADD
        | Mnm::IMUL
        | Mnm::CMP
        | Mnm::TEST
        | Mnm::DEC
        | Mnm::INC
        | Mnm::OR
        | Mnm::AND
        | Mnm::NOT
        | Mnm::NEG
        | Mnm::XOR => {
            matches!(
                (ins.dst(), ins.src()),
                (_, Some(Operand::Reg(_)))
                    | (Some(Operand::Reg(_)), _)
                    | (Some(Operand::Mem(_) | Operand::Segment(_)), _)
                    | (_, Some(Operand::Mem(_) | Operand::Segment(_)))
            )
        }
        Mnm::SAR | Mnm::SAL | Mnm::SHL | Mnm::SHR | Mnm::LEA => true,
        _ => {
            if let Some(Operand::Reg(dst)) = ins.dst() {
                if dst.needs_rex() {
                    return true;
                }
            }
            if let Some(Operand::Reg(src)) = ins.src() {
                if src.needs_rex() {
                    return true;
                }
            }
            false
        }
    }
}

fn get_wb(op: Option<&Operand>) -> u8 {
    match op {
        Some(Operand::Reg(reg) | Operand::CtrReg(reg) | Operand::DbgReg(reg)) => {
            if reg.needs_rex() {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn calc_rex(ins: &Instruction, rev: bool) -> u8 {
    // fixed pattern
    let base = 0b0100_0000;

    let w: u8 = if ins.uses_cr() || ins.uses_dr() || ins.mnem.defaults_to_64bit() {
        0
    } else {
        1
    };

    let r = if !rev {
        get_wb(ins.src())
    } else {
        get_wb(ins.dst())
    };
    let mut b = if !rev {
        get_wb(ins.dst())
    } else {
        get_wb(ins.src())
    };

    let mut x: u8 = 0;
    if let (_, Some(Operand::Segment(m))) | (Some(Operand::Segment(m)), _) = (ins.dst(), ins.src())
    {
        match m.address {
            Mem::SIB(_, i, _, _)
            | Mem::SIBOffset(_, i, _, _, _)
            | Mem::Index(i, _, _)
            | Mem::IndexOffset(i, _, _, _) => {
                if i.needs_rex() {
                    x = 1;
                }
            }
            _ => {}
        }
    }
    if let (_, Some(Operand::Mem(m))) | (Some(Operand::Mem(m)), _) = (ins.dst(), ins.src()) {
        match m {
            Mem::SIB(_, i, _, _)
            | Mem::SIBOffset(_, i, _, _, _)
            | Mem::Index(i, _, _)
            | Mem::IndexOffset(i, _, _, _) => {
                if i.needs_rex() {
                    x = 1;
                }
            }
            _ => {}
        }
    }

    if let (_, Some(Operand::Mem(m))) | (Some(Operand::Mem(m)), _) = (ins.dst(), ins.src()) {
        match m {
            Mem::SIB(base, _, _, _) | Mem::SIBOffset(base, _, _, _, _) => {
                if base.needs_rex() {
                    b = 1;
                }
            }
            _ => {}
        }
    }
    if let (_, Some(Operand::Segment(m))) | (Some(Operand::Segment(m)), _) = (ins.dst(), ins.src())
    {
        match m.address {
            Mem::SIB(base, _, _, _) | Mem::SIBOffset(base, _, _, _, _) => {
                if base.needs_rex() {
                    b = 1;
                }
            }
            _ => {}
        }
    }

    base + (w << 3) + (r << 2) + (x << 1) + b
}

pub fn gen_rex(ins: &Instruction, rev: bool) -> Option<u8> {
    if needs_rex(ins) {
        Some(calc_rex(ins, rev))
    } else {
        None
    }
}
