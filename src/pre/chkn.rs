// pasm - src/pre/chkn.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use std::mem::MaybeUninit;

use crate::core::apx::APXVariant;
use crate::shr::{
    instruction::{Instruction, Operand},
    atype::{AType, ToType, BCST_FLAG, K, VSIB_FLAG},
    booltable::BoolTable8 as Flags8,
    error::Error,
    mnemonic::Mnemonic,
    size::Size,
    stackvec::StackVec,
};
const REG_TYPE: u16 = 0b01;
const MEM_TYPE: u16 = 0b10;
const IMM_TYPE: u16 = 0b11;

const OPR_NEEDED: u8 = 0x0;
const HAS_IMM: u8 = 0x2;
const HAS_REG: u8 = 0x3;
const HAS_MEM: u8 = 0x4;

#[repr(u8)]
pub enum AVX10Modifier {
    None = 0b00,
    ER = 0b01,
    SAE = 0b10,
}

// metadata layout:
//  1-4th: prediction table:
//      - prediction table entry (NOTE: last bits are first bits):
//          0bXXXX: size of slice
//  5th and 6th: "prediction table" (PTable) metadata
//      - 0b0000_0000 0bXXYY_ZZAA:
//          - XX: operand type
//          - YY: operand type
//          - ZZ: operand type
//          - AA: operand type
//  7th: Booltable8
//  8th: reserved
// Example:
//  let's assert that we have in our operand set key for array (it must be ordered!): [R32, R64, M32, M64] and we want to search for R64.
//  Our checker will start by checking if we have registers.
//  We have HAS_REG set to 1, so we move on.
//  We then load PTable metadata and check for indexes.
//  In this case we search for Register Operand Type.
//  Let's assert that first 2 bits give a `Register` type.
//  Then we load next operand (in this case a `Memory` type).
//  It is different from what we search for, so slice is keys[0..2]
//  Now we iterate over slice and search for our type.
//  And we got it! we got R64, so it means our variant was correct.
pub struct OperandSet<'a> {
    ptable_data: u32,
    ptable_meta: u16,
    flags: Flags8,
    _reserved: u8,
    keys: &'a [AType],
}

impl<'a> OperandSet<'a> {
    pub fn is_optional(&self) -> bool {
        !self.flags.get(OPR_NEEDED).unwrap_or(false)
    }
    pub fn has(&self, rhs: AType) -> bool {
        let tp = match rhs {
            AType::Register(_, _) => {
                if self.has_reg() {
                    REG_TYPE
                } else {
                    return false;
                }
            }
            AType::Immediate(_, _) => {
                if self.has_imm() {
                    IMM_TYPE
                } else {
                    return false;
                }
            }
            AType::Memory(_, _, _) => {
                if self.has_mem() {
                    MEM_TYPE
                } else {
                    return false;
                }
            }
            AType::Any | AType::NoType => return false,
        };
        let mut idx = 0;
        let mut off = 0;
        let mut lsz = 0;
        loop {
            let crt = (self.ptable_meta & (0b11 << (idx << 1))) >> (idx << 1);
            if crt == 0 {
                break;
            } else if crt == tp {
                let lix = idx << 2;
                let and = 0b1111 << lix;
                lsz = (self.ptable_data & and) >> lix;
                break;
            }
            off += (self.ptable_data & (0b1111 << (idx << 2))) >> (idx << 2);
            idx += 1;
        }
        let slice_start = off as usize;
        let slice_end = (off + lsz) as usize;
        let slice = &self.keys[slice_start..slice_end];
        for key in slice {
            if key == &rhs {
                return true;
            }
        }
        false
    }
    pub fn has_imm(&self) -> bool {
        self.flags.get(HAS_IMM).unwrap_or_default()
    }
    pub fn has_mem(&self) -> bool {
        self.flags.get(HAS_MEM).unwrap_or_default()
    }
    pub fn has_reg(&self) -> bool {
        self.flags.get(HAS_REG).unwrap_or_default()
    }
    #[inline(always)]
    pub const fn new<const N: usize>(ndd: bool, ats: &'a [AType; N]) -> Self {
        let mut flags = Flags8::new();
        flags.set(OPR_NEEDED, ndd);
        let mut ptable_data = 0;
        let mut ptable_meta = 0;
        let _reserved = 0;

        let mut pvtype = 0;

        let mut lttype = 0;

        let mut ptindx = 0;
        let mut slsize = 0;

        let mut idx = 0;
        while idx < N {
            let crtype = match ats[idx] {
                AType::Register(_, _) => {
                    flags.set(HAS_REG, true);
                    REG_TYPE
                }
                AType::Memory(_, _, _) => {
                    flags.set(HAS_MEM, true);
                    MEM_TYPE
                }
                AType::Immediate(_, _) => {
                    flags.set(HAS_IMM, true);
                    IMM_TYPE
                }
                AType::Any | AType::NoType => 0b00,
            };
            if pvtype == 0 {
                pvtype = crtype;
                ptindx = 0;
                slsize = 1;
            } else if crtype != pvtype {
                ptable_meta |= pvtype << (ptindx << 1);
                ptable_data |= slsize << (ptindx << 2);
                lttype = pvtype;
                pvtype = crtype;
                ptindx += 1;
                slsize = 1;
            } else {
                slsize += 1;
            }
            idx += 1;
        }
        if lttype != pvtype {
            ptable_meta |= pvtype << (ptindx << 1);
            ptable_data |= slsize << (ptindx << 2);
        }

        Self {
            ptable_data,
            ptable_meta,
            flags,
            _reserved,
            keys: ats,
        }
    }
}

pub const EVEX: u8 = 0b0001;
pub const APX: u8 = 0b0010;

pub struct CheckAPI<'a, const OPERAND_COUNT: usize> {
    allowed: StackVec<OperandSet<'a>, OPERAND_COUNT>,

    forbidden: MaybeUninit<&'a [[AType; OPERAND_COUNT]]>,
    additional: MaybeUninit<&'a [Mnemonic]>,

    // Also: TODO: make CheckAPI actually respect if EVEX: EE and SSSS.
    // layout:
    // 0bXXXX_MFRR_ZZZZ_ZZZZ:
    //  XXXX: prefix
    //      0000 - None
    //      0001 - EVEX
    //      0010 - APX
    //  M: additional mnemonic
    //  F: forbidden operand combination
    //  RR: check mode
    //  ZZZZ_ZZZZ: specific
    //      if prefix == EVEX:
    //          EEKZ_SSSS:
    //              EE: AVX10 modifier
    //                  00 - None
    //                  01 - {er}
    //                  10 - {sae}
    //                  11 - reseved
    //              K: can have masks
    //              Z: can have {z}
    //              SSSS:
    //                  if EE != 0:
    //                      Size where you can use EE modifier
    //      if prefix == FPFX_APX:
    //          AAAF_0000
    //              AAA - EEvexVariant
    //              F - can use NF
    //
    flags: u16,
}

#[repr(u8)]
pub enum CheckMode {
    NONE = 0b00,   // don't check for size diff
    AVX = 0b01,    // don't check for size diff
    X86 = 0b10,    // check for size
    NOSIZE = 0b11, // don't check for size diff
}

impl<'a, const OPERAND_COUNT: usize> CheckAPI<'a, OPERAND_COUNT> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            allowed: StackVec::new(),
            flags: 0,
            additional: unsafe { MaybeUninit::uninit().assume_init() },
            forbidden: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    // Forbidden operand combinations go here
    const fn allow_forbidden(&mut self) {
        self.flags &= !0b0100_0000_0000;
        self.flags |= 0b0100_0000_0000;
    }
    const fn allows_forbidden(&self) -> bool {
        self.flags & 0b0100_0000_0000 == 0b0100_0000_0000
    }

    // Forced Prefix-related methods go here
    const fn fpfx_set(&mut self, v: u8) {
        self.flags &= !0b1111_0000_0000_0000;
        self.flags |= (v as u16 & 0b1111) << 12;
    }
    const fn fpfx_get(&self) -> u8 {
        ((self.flags & 0b1111_0000_0000_0000) >> 12) as u8
    }
    // Intel APX related methods go here
    pub const fn is_apx(&self) -> bool {
        self.fpfx_get() == APX
    }
    pub const fn apx(mut self, apx: APXVariant, nf: bool) -> Self {
        self.fpfx_set(APX);
        self.flags |= (apx as u16) << 5;
        self.flags |= (nf as u16) << 4;
        self
    }
    pub const fn apx_get(&self) -> (APXVariant, bool) {
        (
            unsafe { std::mem::transmute(((self.flags & 0b1110_0000) >> 5) as u8) },
            self.flags & 1 << 4 == 1 << 4,
        )
    }

    // AVX-512 (including AVX-10 and APX EEVEX EVEX Extension)
    pub const fn get_evex(&self) -> bool {
        self.fpfx_get() == EVEX
    }
    pub const fn set_evex(mut self) -> Self {
        self.fpfx_set(EVEX);
        self
    }
    pub const fn get_mask(&self) -> bool {
        if !self.get_evex() {
            return false;
        }
        self.flags & 0b0010_0000 == 0b0010_0000
    }
    pub const fn allow_masks(mut self) -> Self {
        self = self.set_evex();
        self.flags |= 0b0010_0000;
        self
    }

    pub const fn avx10_modifier(mut self, kind: AVX10Modifier, for_size: Size) -> Self {
        self = self.set_evex();
        self.flags &= !0b1100_1111;
        self.flags |= (kind as u16) << 6;
        self.flags |= Size::se(&for_size) as u16;
        self
    }
    const fn get_avx10_modifier(&self) -> Option<(AVX10Modifier, Size)> {
        if !self.get_evex() {
            return None;
        }
        let modf = unsafe {
            std::mem::transmute::<u8, AVX10Modifier>(((self.flags & 0b1100_0000) >> 6) as u8)
        };
        let size = Size::de((self.flags & 0b1111) as u8);
        Some((modf, size))
    }

    // Check Mode related
    pub const fn get_mode(&self) -> CheckMode {
        unsafe {
            std::mem::transmute::<u8, CheckMode>(((self.flags & 0b0000_0011_0000_0000) >> 8) as u8)
        }
    }
    pub const fn set_mode(mut self, mode: CheckMode) -> Self {
        self.flags |= (mode as u16 & 0b11) << 8;
        self
    }

    // Additional Mnemonic related
    pub const fn set_addt_flag(&mut self) {
        self.flags |= 0b0000_1000_0000_0000;
    }
    pub const fn has_addt(&self) -> bool {
        self.flags & 0b1000_0000_0000 == 0b1000_0000_0000
    }

    // Builder methods
    pub const fn forbidden(mut self, forb: &'a [[AType; OPERAND_COUNT]]) -> Self {
        self.allow_forbidden();
        self.forbidden = MaybeUninit::new(forb);
        self
    }
    pub const fn additional_mnemonics(mut self, addt: &'a [Mnemonic]) -> Self {
        self.set_addt_flag();
        self.additional = MaybeUninit::new(addt);
        self
    }

    pub const fn push<const N: usize>(mut self, types: &'a [AType; N], needed: bool) -> Self {
        self.allowed.push(OperandSet::new(needed, types));
        self
    }

    pub fn check_addt(&self, ins: &Instruction) -> Result<(), Error> {
        if let Some(found) = ins.get_addt() {
            if self.has_addt() {
                let mut f = false;
                for allowed in &**unsafe { self.additional.assume_init_ref() } {
                    if allowed == &found {
                        f = true;
                        break;
                    }
                }
                if !f {
                    let er = Error::new("you tried to use prefix mnemonic, but primary mnemonic does not allow for this one", 6);
                    return Err(er);
                }
            } else {
                let er = Error::new(
                    "you tried to use prefix mnemonic, but primary mnemonic does not allow for one",
                    6,
                );
                return Err(er);
            }
        }
        Ok(())
    }
    pub fn check_forb(&self, ops: &StackVec<Operand, 4>) -> Result<(), Error> {
        if !self.allows_forbidden() {
            return Ok(());
        }
        let mut smv: StackVec<AType, OPERAND_COUNT> = StackVec::new();
        for o in ops.iter() {
            smv.push(o.atype());
        }

        for f in unsafe { self.forbidden.assume_init_ref() }.iter() {
            let mut at = 0;
            for (i, lhs) in f.iter().enumerate() {
                if let Some(rhs) = smv.get(i) {
                    if lhs == rhs {
                        at += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            if at == OPERAND_COUNT {
                let er = Error::new("you tried to use forbidden operand combination", 7);
                return Err(er);
            }
        }

        Ok(())
    }
    pub fn check(&self, ins: &Instruction) -> Result<(), Error> {
        let mut smv: StackVec<Operand, 4> = StackVec::new();

        for o in ins.iter() {
            smv.push(o);
        }

        self.check_addt(ins)?;

        for (i, o) in self.allowed.iter().enumerate() {
            if let Some(s) = smv.get(i) {
                if !o.has(s.atype()) {
                    let er = Error::new(
                        format!("operand at index {i} has invalid type: {}", s.atype()),
                        8,
                    );
                    return Err(er);
                }
            } else if o.is_optional() {
                break;
            } else {
                let er = Error::new("you didn't provide valid amount of operands", 9);
                return Err(er);
            }
        }
        if self.allows_forbidden() {
            self.check_forb(&smv)?;
        }

        // APX specific checks
        if self.is_apx() {
            let (avr, nf) = self.apx_get();
            if let Some(true) = ins.apx_get_leg_nf() {
                if !nf {
                    return Err(Error::new(
                        "you tried to use {nf} on instruction that does not support it",
                        22,
                    ));
                }
            } else if let Some(true) = ins.apx_eevex_vex_get_nf() {
                if !nf {
                    return Err(Error::new(
                        "you tried to use {vex-nf} on instruction that does not support it",
                        22,
                    ));
                }
            }
            if let Some(ins_apx) = ins.apx_get_eevex_mode() {
                if avr != ins_apx && ins_apx != APXVariant::Rex2 {
                    return Err(Error::new(
                        format!(
                            "you tried to use {} on instruction that does not support it",
                            match avr {
                                APXVariant::Rex2 => "rex2 extension",
                                APXVariant::EvexExtension => "EEVEX EVEX Extension",
                                APXVariant::VexExtension => "EEVEX VEX Extension",
                                APXVariant::LegacyExtension => "EEVEX Legacy Extension",
                                APXVariant::CondTestCmpExtension =>
                                    "EEVEX for conditional tests and compares",
                                APXVariant::Auto => "EEVEX",
                            }
                        ),
                        22,
                    ));
                }
            }
        }
        // EVEX specific checks
        else if self.get_evex() {
            match ins.evex_mask() {
                None | Some(0) => {}
                Some(_) => {
                    if !self.get_mask() {
                        return Err(Error::new(
                            "you tried to use mask on instruction that does not support it",
                            16,
                        ));
                    }
                }
            }

            let (modf, size) = self
                .get_avx10_modifier()
                .expect("self.get_avx10_modifier() with self.get_evex() SHOULD ALWAYS BE SOME");

            match modf {
                AVX10Modifier::None => {}
                AVX10Modifier::ER => {
                    if ins.evex_er().is_some() && ins.size_full_gt() != size {
                        return Err(Error::new("you tried to use {er} subexpression on instruction using wrong variant", 16));
                    } else if let Some(true) = ins.evex_sae() {
                        return Err(Error::new("you tried to use {sae} subexpression on instruction that does not allow it", 16));
                    }
                }
                AVX10Modifier::SAE => {
                    if let Some(true) = ins.evex_sae() {
                        if ins.size_full_gt() != size {
                            return Err(Error::new("you tried to use {sae} subexpression on instruction using wrong variant", 16));
                        }
                    } else if ins.evex_er().is_some() {
                        return Err(Error::new("you tried to use {er} subexpression on instruction that does not allow it", 16));
                    }
                }
            }
        } else if ins.is_evex() {
            return Err(Error::new(
                "you tried to use AVX-512 modifiers on instruction that is not from AVX-512",
                16,
            ));
        }

        match self.get_mode() {
            CheckMode::NONE | CheckMode::AVX | CheckMode::NOSIZE => {}
            CheckMode::X86 => {
                let mut sz = Size::Unknown;
                for o in smv.into_iter() {
                    if let AType::Memory(Size::Word | Size::Dword | Size::Qword, _, fl) = o.atype()
                    {
                        if fl.get(BCST_FLAG).unwrap() || fl.get(VSIB_FLAG).unwrap() {
                            continue;
                        }
                    } else if o.atype() == K {
                        continue;
                    }
                    if sz == Size::Unknown {
                        if let Some(r) = o.get_reg() {
                            if r.is_dbg_reg() || r.is_ctrl_reg() || r.is_sgmnt() {
                                continue;
                            }
                        }
                        sz = o.size();
                        continue;
                    }
                    if o.is_imm() && sz < o.size() {
                        let er = Error::new(
                            "you provided immediate which size was larger than other operands",
                            8,
                        );
                        return Err(er);
                    }
                    match o {
                        Operand::Register(r) => {
                            if sz != r.size()
                                && !r.is_dbg_reg()
                                && !r.is_ctrl_reg()
                                && !r.is_sgmnt()
                            {
                                let er = Error::new(
                                    "you tried to use invalid operand size in this instruction",
                                    8,
                                );
                                return Err(er);
                            }
                        }
                        Operand::Mem(m) => {
                            if sz != m.size() {
                                let er = Error::new(
                                    "you tried to use invalid operand size in this instruction",
                                    8,
                                );
                                return Err(er);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::shr::reg::Register;
    #[test]
    fn tops_0() {
        use crate::shr::atype::*;
        assert_eq!(1 << 1, 2);
        assert_eq!(0 << 1, 0);
        assert_eq!(REG_TYPE, 0b01);
        assert_eq!(MEM_TYPE << (1 << 1), 0b1000);
        assert_eq!((MEM_TYPE << (1 << 1)) | REG_TYPE, 0b1001);
        let o = OperandSet::new(
            true,
            &[
                AType::Register(Register::AL, false),
                AType::Register(Register::BX, false),
                M16,
                I8,
            ],
        );
        assert_eq!(
            o.keys,
            vec![
                AType::Register(Register::AL, false),
                AType::Register(Register::BX, false),
                M16,
                I8,
            ]
        );
        assert_eq!(o.ptable_meta, 0b0000_0000_0011_1001);
        assert_eq!(o.ptable_data, 0b0001_0001_0010);
        assert!(o.has(AType::Register(Register::BX, false)));
        assert!(o.has(M16));
        assert!(o.has(I8));
    }
    #[test]
    fn tchk_1() {
        use crate::shr::instruction::Operand;
        let t = Operand::Register(Register::RAX).atype();
        assert_eq!(t, AType::Register(Register::RAX, false));
        assert!(Register::EAX != Register::K0);
        assert_eq!(size_of::<AType>(), 4);
    }
}
