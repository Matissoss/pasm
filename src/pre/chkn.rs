// pasm - src/pre/chkn.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use std::mem::MaybeUninit;

use crate::shr::{
    ast::{Instruction, Operand},
    atype::{AType, ToType, BCST_FLAG, K, VSIB_FLAG},
    booltable::BoolTable8 as Flags8,
    error::Error,
    ins::Mnemonic,
    size::Size,
    smallvec::SmallVec,
};
const REG_TYPE: u16 = 0b01;
const MEM_TYPE: u16 = 0b10;
const IMM_TYPE: u16 = 0b11;

const OPR_NEEDED: u8 = 0x1;
const HAS_IMM: u8 = 0x2;
const HAS_REG: u8 = 0x3;
const HAS_MEM: u8 = 0x4;

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
        self.flags.get(OPR_NEEDED).unwrap_or(false)
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

pub struct CheckAPI<'a, const OPERAND_COUNT: usize> {
    allowed: SmallVec<OperandSet<'a>, OPERAND_COUNT>,

    forbidden: MaybeUninit<&'a [[AType; OPERAND_COUNT]]>,
    additional: MaybeUninit<&'a [Mnemonic]>,

    // layout:
    //  0bXY_00AM_ZZ:
    //  X - has additional mnemonic
    //  Y - has forbidden op combination
    //  00 - reserved
    //  A - is avx-512 instruction
    //  M - can have masks
    //  ZZ - check mode
    flags: u8,
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
            allowed: SmallVec::new(),
            flags: CheckMode::NONE as u8,
            additional: unsafe { MaybeUninit::uninit().assume_init() },
            forbidden: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
    const fn set_forb_flag(&mut self) {
        self.flags |= 0b0100_0000;
    }
    pub const fn has_forb(&self) -> bool {
        self.flags & 0b0100_0000 == 0b0100_0000
    }
    pub const fn get_avx512(&self) -> bool {
        self.flags & 0b00_0010_00 == 0b00_0010_00
    }
    pub const fn set_avx512(mut self) -> Self {
        self.flags |= 0b00_0010_00;
        self
    }
    pub const fn get_mask(&self) -> bool {
        self.flags & 0b00_0001_00 == 0b00_0001_00
    }
    pub const fn set_mask_perm(mut self) -> Self {
        self.flags |= 0b00_0001_00;
        self
    }
    pub const fn get_mode(&self) -> CheckMode {
        unsafe { std::mem::transmute::<u8, CheckMode>(self.flags & 0b11) }
    }
    pub const fn set_mode(mut self, mode: CheckMode) -> Self {
        self.flags |= mode as u8;
        self
    }
    pub const fn set_addt_flag(&mut self) {
        self.flags |= 0b1000_0000;
    }
    pub const fn has_addt(&self) -> bool {
        self.flags & 0b1000_0000 == 0b1000_0000
    }
    pub const fn set_forb(mut self, forb: &'a [[AType; OPERAND_COUNT]]) -> Self {
        self.set_forb_flag();
        self.forbidden = MaybeUninit::new(forb);
        self
    }
    pub const fn set_addt(mut self, addt: &'a [Mnemonic]) -> Self {
        self.set_addt_flag();
        self.additional = MaybeUninit::new(addt);
        self
    }
    pub const fn pushop<const N: usize>(mut self, types: &'a [AType; N], needed: bool) -> Self {
        self.allowed.push(OperandSet::new(needed, types));
        self
    }
    pub const fn getops(&self, idx: usize) -> Option<&OperandSet> {
        self.allowed.get(idx)
    }
    pub fn check_addt(&self, ins: &Instruction) -> Result<(), Error> {
        if let Some(found) = ins.addt() {
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
    pub fn check_forb(&self, ins: &Instruction) -> Result<(), Error> {
        if !self.has_forb() {
            return Ok(());
        }
        let mut smv: SmallVec<AType, OPERAND_COUNT> = SmallVec::new();
        for o in ins.iter() {
            smv.push(o.atype());
        }

        let mut at = 0;
        for f in unsafe { self.forbidden.assume_init_ref() }.iter() {
            for (i, k) in f.iter().enumerate() {
                if Some(k) == smv.get(i) {
                    at += 1;
                } else {
                    break;
                }
            }
            if at == smv.len() {
                let er = Error::new("you tried to use forbidden operand combination", 7);
                return Err(er);
            }
        }

        Ok(())
    }
    pub fn check(&self, ins: &Instruction) -> Result<(), Error> {
        self.check_addt(ins)?;
        if ins.get_mask().is_some() && !self.get_mask() {
            return Err(Error::new(
                "you tried to use mask on instruction that does not support it",
                16,
            ));
        }
        if (ins.get_evex()) && !self.get_avx512() {
            return Err(Error::new(
                "you tried to use AVX-512 modifiers on instruction that is not from AVX-512",
                16,
            ));
        }
        for (i, o) in self.allowed.iter().enumerate() {
            if let Some(s) = ins.get(i) {
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
        match self.get_mode() {
            CheckMode::NONE | CheckMode::AVX | CheckMode::NOSIZE => {}
            CheckMode::X86 => {
                let mut sz = Size::Unknown;
                for o in ins.iter() {
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
mod chkn_test {
    use super::*;
    use crate::shr::reg::Register;
    #[test]
    fn ops_test() {
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
    fn chk_test() {
        use crate::shr::ast::Operand;
        let t = Operand::Register(Register::RAX).atype();
        assert_eq!(t, AType::Register(Register::RAX, false));
    }
}
