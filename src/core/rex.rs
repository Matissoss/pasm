// pasm - src/core/rex.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use crate::shr::{
    ast::{Instruction, Operand},
    ins::Mnemonic,
    size::Size,
};

pub fn needs_rex(ins: &Instruction) -> bool {
    if matches!(ins.mnem, Mnemonic::CMPXCHG16B) {
        return true;
    }
    let (size_d, size_s) = match (ins.dst(), ins.src()) {
        (Some(d), Some(s)) => (d.size(), s.size()),
        (Some(d), None) => (d.size(), Size::Unknown),
        (None, Some(s)) => (Size::Unknown, s.size()),
        _ => (Size::Unknown, Size::Unknown),
    };
    for o in ins.operands.iter() {
        if let Some(r) = o.get_reg() {
            if r.get_ext_bits()[1] {
                return true;
            }
        }
    }
    match (size_d, size_s) {
        (Size::Qword, Size::Qword) | (Size::Qword, _) | (_, Size::Qword) => {}
        _ => return false,
    }
    match &ins.mnem {
        Mnemonic::XADD
        | Mnemonic::XRSTOR
        | Mnemonic::XRSTOR64
        | Mnemonic::XRSTORS
        | Mnemonic::XRSTORS64
        | Mnemonic::XSAVE
        | Mnemonic::XSAVE64
        | Mnemonic::XSAVEC
        | Mnemonic::XSAVEC64
        | Mnemonic::XSAVEOPT
        | Mnemonic::XSAVEOPT64
        | Mnemonic::XSAVES
        | Mnemonic::XSAVES64
        | Mnemonic::SHLD
        | Mnemonic::SHRD
        | Mnemonic::SMSW
        | Mnemonic::WRFSBASE
        | Mnemonic::WRGSBASE => true,

        Mnemonic::ROL
        | Mnemonic::RDRAND
        | Mnemonic::RDSEED
        | Mnemonic::RDSSPQ
        | Mnemonic::ROR
        | Mnemonic::RCL
        | Mnemonic::RCR
        | Mnemonic::BSF
        | Mnemonic::INVPCID
        | Mnemonic::ADCX
        | Mnemonic::ADOX
        | Mnemonic::BSWAP
        | Mnemonic::CMPXCHG
        | Mnemonic::BSR
        | Mnemonic::BT
        | Mnemonic::BTC
        | Mnemonic::BTS
        | Mnemonic::BTR
        | Mnemonic::CMOVA
        | Mnemonic::CMOVB
        | Mnemonic::CMOVC
        | Mnemonic::CMOVE
        | Mnemonic::CMOVG
        | Mnemonic::CMOVL
        | Mnemonic::CMOVO
        | Mnemonic::CMOVP
        | Mnemonic::CMOVS
        | Mnemonic::CMOVZ
        | Mnemonic::CMOVAE
        | Mnemonic::CMOVBE
        | Mnemonic::CMOVLE
        | Mnemonic::CMOVGE
        | Mnemonic::CMOVNA
        | Mnemonic::CMOVNB
        | Mnemonic::CMOVNC
        | Mnemonic::CMOVNE
        | Mnemonic::CMOVNG
        | Mnemonic::CMOVNL
        | Mnemonic::CMOVNO
        | Mnemonic::CMOVNP
        | Mnemonic::CMOVNS
        | Mnemonic::CMOVNZ
        | Mnemonic::CMOVPE
        | Mnemonic::CMOVPO
        | Mnemonic::CMOVNBE
        | Mnemonic::CMOVNLE
        | Mnemonic::CMOVNGE
        | Mnemonic::MOVZX
        | Mnemonic::MOVDIRI
        | Mnemonic::MOVBE
        | Mnemonic::LZCNT
        | Mnemonic::LSL
        | Mnemonic::CMOVNAE => true,
        Mnemonic::MOVMSKPD => true,
        Mnemonic::CVTSS2SI => true,
        Mnemonic::CVTSI2SS => true,
        Mnemonic::PINSRQ => true,
        Mnemonic::MOVQ => true,
        Mnemonic::PEXTRW | Mnemonic::PEXTRQ => true,
        Mnemonic::POPCNT => true,
        Mnemonic::EXTRACTPS => true,
        Mnemonic::MOV => {
            if let (Some(Operand::Register(r0)), Some(Operand::Register(r1))) =
                (ins.dst(), ins.src())
            {
                if r0.is_ctrl_reg()
                    || r0.is_dbg_reg()
                    || r1.is_ctrl_reg()
                    || r1.is_dbg_reg()
                    || r0.is_sgmnt()
                    || r1.is_sgmnt()
                {
                    return r0.get_ext_bits()[1] || r1.get_ext_bits()[1];
                } else {
                    return true;
                }
            }
            if let (Some(Operand::Mem(_)), _) | (_, Some(Operand::Mem(_))) = (ins.dst(), ins.src())
            {
                return true;
            }
            false
        }
        Mnemonic::SUB
        | Mnemonic::ADD
        | Mnemonic::IMUL
        | Mnemonic::CMP
        | Mnemonic::TEST
        | Mnemonic::DEC
        | Mnemonic::INC
        | Mnemonic::OR
        | Mnemonic::AND
        | Mnemonic::NOT
        | Mnemonic::NEG
        | Mnemonic::ADC
        | Mnemonic::SBB
        | Mnemonic::XCHG
        | Mnemonic::XOR => {
            matches!(
                (ins.dst(), ins.src()),
                (_, Some(Operand::Register(_)))
                    | (Some(Operand::Register(_)), _)
                    | (Some(Operand::Mem(_)), _)
                    | (_, Some(Operand::Mem(_)))
            )
        }
        Mnemonic::SAR | Mnemonic::SAL | Mnemonic::SHL | Mnemonic::SHR | Mnemonic::LEA => true,
        _ => {
            if let Some(Operand::Register(dst)) = ins.dst() {
                if dst.get_ext_bits()[1] {
                    return true;
                }
            }
            if let Some(Operand::Register(src)) = ins.src() {
                if src.get_ext_bits()[1] {
                    return true;
                }
            }
            false
        }
    }
}

fn get_wb(op: Option<&Operand>) -> bool {
    match op {
        Some(Operand::Register(reg)) => reg.get_ext_bits()[1],
        _ => false,
    }
}

fn fix_rev(r: &mut bool, ins: &Instruction) {
    #[allow(clippy::single_match)]
    match ins.dst() {
        Some(Operand::Register(reg)) => {
            if reg.size() == Size::Xword {
                *r = true;
            }
        }
        _ => {}
    }
    if matches!(ins.mnem, Mnemonic::UD1 | Mnemonic::UD2) {
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
        || matches!(ins.mnem, Mnemonic::CMPXCHG16B);
    let mut r = if !modrm_reg_is_dst { wbs } else { wbd };
    let mut b = if !modrm_reg_is_dst { wbd } else { wbs };
    let mut x = false;

    match (ins.dst(), ins.src()) {
        (Some(Operand::Mem(m)), _) | (_, Some(Operand::Mem(m))) => (b, x) = m.needs_rex(),
        _ => {}
    }
    if let Some(Operand::Register(reg)) = ins.dst() {
        if reg.get_ext_bits()[1] {
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
