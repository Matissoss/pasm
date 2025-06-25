// pasm - src/core/rex.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::{Instruction, Operand},
    ins::Mnemonic as Mnm,
    size::Size,
};

fn needs_rex(ins: &Instruction) -> bool {
    if matches!(ins.mnem, Mnm::CMPXCHG16B) {
        return true;
    }
    let (size_d, size_s) = match (ins.dst(), ins.src()) {
        (Some(d), Some(s)) => (d.size(), s.size()),
        (Some(d), None) => (d.size(), Size::Unknown),
        (None, Some(s)) => (Size::Unknown, s.size()),
        _ => (Size::Unknown, Size::Unknown),
    };
    match (ins.dst(), ins.src()) {
        (Some(Operand::Reg(r)), Some(Operand::Reg(r1))) => {
            if r.needs_rex() || r1.needs_rex() {
                return true;
            }
        }
        (Some(Operand::Reg(r)), _) => {
            if r.needs_rex() {
                return true;
            }
        }
        (_, Some(Operand::Reg(r))) => {
            if r.needs_rex() {
                return true;
            }
        }
        _ => {}
    };
    match (size_d, size_s) {
        (Size::Qword, Size::Qword) | (Size::Qword, _) | (_, Size::Qword) => {}
        _ => return false,
    }
    match &ins.mnem {
        Mnm::XADD
        | Mnm::XRSTOR
        | Mnm::XRSTOR64
        | Mnm::XRSTORS
        | Mnm::XRSTORS64
        | Mnm::XSAVE
        | Mnm::XSAVE64
        | Mnm::XSAVEC
        | Mnm::XSAVEC64
        | Mnm::XSAVEOPT
        | Mnm::XSAVEOPT64
        | Mnm::XSAVES
        | Mnm::XSAVES64
        | Mnm::SHLD
        | Mnm::SHRD
        | Mnm::SMSW
        | Mnm::WRFSBASE
        | Mnm::WRGSBASE => true,

        Mnm::ROL
        | Mnm::RDRAND
        | Mnm::RDSEED
        | Mnm::RDSSPQ
        | Mnm::ROR
        | Mnm::RCL
        | Mnm::RCR
        | Mnm::BSF
        | Mnm::INVPCID
        | Mnm::ADCX
        | Mnm::ADOX
        | Mnm::BSWAP
        | Mnm::CMPXCHG
        | Mnm::BSR
        | Mnm::BT
        | Mnm::BTC
        | Mnm::BTS
        | Mnm::BTR
        | Mnm::CMOVA
        | Mnm::CMOVB
        | Mnm::CMOVC
        | Mnm::CMOVE
        | Mnm::CMOVG
        | Mnm::CMOVL
        | Mnm::CMOVO
        | Mnm::CMOVP
        | Mnm::CMOVS
        | Mnm::CMOVZ
        | Mnm::CMOVAE
        | Mnm::CMOVBE
        | Mnm::CMOVLE
        | Mnm::CMOVGE
        | Mnm::CMOVNA
        | Mnm::CMOVNB
        | Mnm::CMOVNC
        | Mnm::CMOVNE
        | Mnm::CMOVNG
        | Mnm::CMOVNL
        | Mnm::CMOVNO
        | Mnm::CMOVNP
        | Mnm::CMOVNS
        | Mnm::CMOVNZ
        | Mnm::CMOVPE
        | Mnm::CMOVPO
        | Mnm::CMOVNBE
        | Mnm::CMOVNLE
        | Mnm::CMOVNGE
        | Mnm::MOVZX
        | Mnm::MOVDIRI
        | Mnm::MOVBE
        | Mnm::LZCNT
        | Mnm::LSL
        | Mnm::CMOVNAE => true,
        Mnm::MOVMSKPD => true,
        Mnm::CVTSS2SI => true,
        Mnm::CVTSI2SS => true,
        Mnm::PINSRQ => true,
        Mnm::MOVQ => true,
        Mnm::PEXTRW | Mnm::PEXTRQ => true,
        Mnm::POPCNT => true,
        Mnm::EXTRACTPS => true,
        Mnm::MOV => {
            if let (Some(Operand::Reg(_)), Some(Operand::Reg(_)))
            | (Some(Operand::Mem(_)), _)
            | (_, Some(Operand::Mem(_))) = (ins.dst(), ins.src())
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
        | Mnm::ADC
        | Mnm::SBB
        | Mnm::XCHG
        | Mnm::XOR => {
            matches!(
                (ins.dst(), ins.src()),
                (_, Some(Operand::Reg(_)))
                    | (Some(Operand::Reg(_)), _)
                    | (Some(Operand::Mem(_)), _)
                    | (_, Some(Operand::Mem(_)))
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

fn get_wb(op: Option<&Operand>) -> bool {
    match op {
        Some(Operand::Reg(reg) | Operand::CtrReg(reg) | Operand::DbgReg(reg)) => reg.needs_rex(),
        _ => false,
    }
}

fn fix_rev(r: &mut bool, ins: &Instruction) {
    #[allow(clippy::single_match)]
    match ins.dst() {
        Some(Operand::Reg(reg)) => {
            if reg.size() == Size::Xword {
                *r = true;
            }
        }
        _ => {}
    }
    if matches!(ins.mnem, Mnm::UD1 | Mnm::UD2) {
        *r = true;
    }
}

fn calc_rex(ins: &Instruction, modrm_reg_is_dst: bool) -> u8 {
    let wbs = get_wb(ins.src());
    let wbd = get_wb(ins.dst());

    let sized = if let Some(o) = ins.dst() {
        o.size()
    } else {
        Size::Unknown
    };
    let sizes = if let Some(o) = ins.src() {
        o.size()
    } else {
        Size::Unknown
    };

    let mut modrm_reg_is_dst = modrm_reg_is_dst;
    fix_rev(&mut modrm_reg_is_dst, ins);

    let w = (!(ins.uses_cr() || ins.uses_dr() || ins.mnem.defaults_to_64bit())
        && (sized == Size::Qword || sizes == Size::Qword))
        || matches!(ins.mnem, Mnm::CMPXCHG16B);
    let mut r = if !modrm_reg_is_dst { wbs } else { wbd };
    let mut b = if !modrm_reg_is_dst { wbd } else { wbs };
    let mut x = false;

    match (ins.dst(), ins.src()) {
        (Some(Operand::Mem(m)), _) | (_, Some(Operand::Mem(m))) => (b, x) = m.needs_rex(),
        _ => {}
    }
    if let Some(Operand::Reg(reg)) = ins.dst() {
        if reg.needs_rex() {
            if modrm_reg_is_dst {
                r = true;
            } else {
                b = true;
            }
        }
    }

    rex(w, r, x, b)
}

pub fn gen_rex(ins: &Instruction, rev: bool) -> Option<u8> {
    if needs_rex(ins) {
        Some(calc_rex(ins, rev))
    } else {
        None
    }
}
#[rustfmt::skip]
#[inline(always)]
const fn btoi(b: bool) -> u8 { if b { 1 } else { 0 } }

#[rustfmt::skip]
#[inline(always)]
const fn rex(w: bool, r: bool, x: bool, b: bool) -> u8 { 0b0100_0000 | btoi(w) << 3 | btoi(r) << 2 | btoi(x) << 1 | btoi(b) }
