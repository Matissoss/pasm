// pasm - src/pre/chkn.rs
// ----------------------
// made by matissoss
// licensed under MPL 2.0

use std::{
    fmt::{Debug, Formatter},
    mem::MaybeUninit,
};

use crate::conf::Shared;

use crate::shr::{
    ast::{Instruction, Operand},
    booltable::BoolTable8 as Flags8,
    error::RASMError as Error,
    ins::Mnemonic,
    reg::Register,
    size::Size,
    smallvec::SmallVec,
};

pub const SR: AType = AType::Register(Register::CS, false);
pub const CR: AType = AType::Register(Register::CR0, false);
pub const DR: AType = AType::Register(Register::DR0, false);

pub const CL: AType = AType::Register(Register::CL, true);
pub const AL: AType = AType::Register(Register::AL, true);
pub const AX: AType = AType::Register(Register::AX, true);
pub const EAX: AType = AType::Register(Register::EAX, true);
pub const DX: AType = AType::Register(Register::DX, true);

pub const RA: AType = AType::Register(Register::__ANY, false);
pub const R8: AType = AType::Register(Register::AL, false);
pub const R16: AType = AType::Register(Register::AX, false);
pub const R32: AType = AType::Register(Register::EAX, false);
pub const R64: AType = AType::Register(Register::RAX, false);
pub const XMM: AType = AType::Register(Register::XMM0, false);
pub const YMM: AType = AType::Register(Register::YMM0, false);

pub const MA: AType = AType::Memory(Size::Any, Size::Any);
pub const M8: AType = AType::Memory(Size::Byte, Size::Any);
pub const M16: AType = AType::Memory(Size::Word, Size::Any);
pub const M32: AType = AType::Memory(Size::Dword, Size::Any);
pub const M64: AType = AType::Memory(Size::Qword, Size::Any);
pub const M128: AType = AType::Memory(Size::Xword, Size::Any);
pub const M256: AType = AType::Memory(Size::Yword, Size::Any);

pub const IA: AType = AType::Immediate(Size::Any, false);
pub const I8: AType = AType::Immediate(Size::Byte, false);
pub const I16: AType = AType::Immediate(Size::Word, false);
pub const I32: AType = AType::Immediate(Size::Dword, false);
pub const I64: AType = AType::Immediate(Size::Qword, false);
pub const STRING: AType = AType::Immediate(Size::Unknown, true);

#[derive(Debug, Clone, Copy)]
pub enum AType {
    None,

    //                fixed register
    Register(Register, bool),
    //     size|address size  (registers used)
    Memory(Size, Size),
    //              is_string
    Immediate(Size, bool),
}

trait ToType {
    fn atypen(&self) -> AType;
}

impl ToType for Operand {
    fn atypen(&self) -> AType {
        match self {
            Self::SegReg(r) | Self::CtrReg(r) | Self::DbgReg(r) | Self::Reg(r) => {
                AType::Register(*r, false)
            }
            Self::Mem(m) => AType::Memory(
                m.size().unwrap_or(Size::Unknown),
                m.addrsize().unwrap_or(Size::Unknown),
            ),
            Self::SymbolRef(s) => {
                if s.is_deref() {
                    AType::Memory(s.size().unwrap_or(Size::Unknown), Size::Any)
                } else {
                    AType::Immediate(Size::Dword, false)
                }
            }
            Self::Imm(i) => AType::Immediate(i.size(), false),
            Self::String(_) => AType::Immediate(Size::Unknown, true),
        }
    }
}

impl PartialEq for AType {
    fn eq(&self, rhs: &Self) -> bool {
        match (*self, *rhs) {
            (AType::Register(lr, lf), AType::Register(rr, rf)) => {
                if lf || rf {
                    lr == rr
                } else {
                    if lr.is_any() || rr.is_any() {
                        lr.size() == rr.size()
                    } else {
                        lr.purpose() == rr.purpose() && lr.size() == rr.size()
                    }
                }
            }
            (AType::Memory(lsz, laddr), AType::Memory(rsz, raddr)) => {
                lsz == rsz && (laddr == raddr || (laddr.is_any() || raddr.is_any()))
            }
            (AType::Immediate(lsz, ls), AType::Immediate(rsz, rs)) => {
                if ls && rs {
                    true
                } else {
                    rsz <= lsz
                }
            }
            _ => false,
        }
    }
}

impl From<AType> for Key {
    fn from(t: AType) -> Key {
        Key::enc(t)
    }
}

// key:
// 1 byte: 0bXX_YYYY_ZZ
// XX   - operand type:
//          00 - Invalid
//          01 - Register
//          10 - Memory
//          11 - Immediate
// YYYY  - size:
//          0000 - Unknown/Any
//          0001 - Byte
//          0010 - Word
//          0011 - Dword
//          0100 - Qword
//          0101 - Xword
//          0111 - Yword
//          1000..1111 - reserved
// ZZ - depends on operand type:
//      if register:
//          - Z1: is register fixed (we know its value)
//          - Z2: reserved
//      if memory:
//          - Z1 and Z2: reserved
//      if immediate:
//          - Z1: is string
//          - Z2: reserved
//  2 byte: depends on operand type
//      if register (not fixed):
//          0bXXXX_YYYY:
//              - XXXX: reserved
//              - YYYY: purpose
//      if register (fixed):
//          0bX_YYYY_ZZZ:
//              - X: is extended register
//              - YYYY: purpose
//              - ZZZ: code of register
//      if memory:
//          0bXXX_YYYYY:
//              - XXX: address size:
//                  0b000: unknown/any
//                  0b001: word
//                  0b010: dword
//                  0b011: qword
//                  100..111: reserved
//              - YYYYY: reserved
//      if immediate:
//          0bXXXX_XXXX
//              - XXXX_XXXX: reserved
#[derive(PartialEq, Clone, Copy)]
#[repr(transparent)]
pub struct Key {
    key: u16,
}

impl Debug for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.dec())?;
        Ok(())
    }
}

const REG_TYPE: u16 = 0b01;
const MEM_TYPE: u16 = 0b10;
const IMM_TYPE: u16 = 0b11;

impl Key {
    const fn blank() -> Self {
        Self { key: 0 }
    }
    pub const fn enc(at: AType) -> Self {
        let mut toret = Self { key: 0 };
        toret.set_opt(match at {
            AType::Memory(_, _) => MEM_TYPE,
            AType::Register(_, _) => REG_TYPE,
            AType::Immediate(_, _) => IMM_TYPE,
            AType::None => 0,
        });
        if let AType::Register(r, b) = at {
            toret.set_sz(r.size());
            toret.set_zz((b as u16) << 1);
            if b {
                // X
                toret.key |= (r.needs_rex() as u16) << 7;
                // YYYY
                toret.key |= (r.purpose() as u16) << 3;
                // ZZZ
                toret.key |= r.to_byte() as u16;
            } else {
                toret.key |= r.purpose() as u16;
            }
        } else if let AType::Memory(sz, addr) = at {
            toret.set_sz(sz);
            let addrsz = match addr {
                Size::Word => 0b001,
                Size::Dword => 0b010,
                Size::Qword => 0b011,
                _ => 0b000,
            };
            toret.key |= addrsz << 5;
        } else if let AType::Immediate(sz, is_string) = at {
            toret.set_sz(sz);
            toret.set_zz((is_string as u16) << 1);
        }
        toret
    }
    pub fn dec(&self) -> Option<AType> {
        match self.get_opt() {
            MEM_TYPE => {
                let asz = self
                    .get_addrsz()
                    .expect("Address size should be Some in memory!");
                let msz = self.get_sz();
                Some(AType::Memory(msz, asz))
            }
            REG_TYPE => {
                let rpr = self
                    .get_rpurpose()
                    .expect("Register purpose should be Some in registers!");
                let rsz = self.get_sz();
                let is_fixed = self.get_zz() & 0b10 == 0b10;
                let rcd = if is_fixed { self.key & 0b0000_0111 } else { 0 };
                if is_fixed {
                    let ext = (self.key & 0b1 << 7) == 0b1 << 7;
                    let en = Register::mksek(ext, rsz as u16, rpr as u16, rcd);
                    let de = Register::de(en);
                    Some(AType::Register(de, true))
                } else {
                    let en = Register::mksek(false, rsz as u16, rpr as u16, 0b000);
                    Some(AType::Register(Register::de(en), false))
                }
            }
            IMM_TYPE => Some(AType::Immediate(
                self.get_sz(),
                self.get_zz() & 0b10 == 0b10,
            )),
            _ => None,
        }
    }
    const fn get_zz(&self) -> u16 {
        (self.key & (0b00_0000_11 << 8)) >> 8
    }
    const fn get_rpurpose(&self) -> Option<u8> {
        if self.get_opt() == REG_TYPE {
            // scary :D
            if self.get_zz() & 0b10 == 0b10 {
                Some(((self.key & 0b1111_000) >> 3) as u8)
            } else {
                Some((self.key & 0x0F) as u8)
            }
        } else {
            None
        }
    }
    const fn get_addrsz(&self) -> Option<Size> {
        if MEM_TYPE == self.get_opt() {
            Some(match self.key & (0b111 << 5) {
                0b000 => Size::Any,
                0b001 => Size::Word,
                0b010 => Size::Dword,
                0b011 => Size::Qword,
                _ => Size::Unknown,
            })
        } else {
            None
        }
    }
    // set zz bits in first byte
    const fn set_zz(&mut self, v: u16) {
        self.key |= v << 8;
    }
    // set operand type
    const fn set_opt(&mut self, v: u16) {
        self.key |= v << 14;
    }
    // get operand type
    const fn get_opt(&self) -> u16 {
        (self.key & (0b11 << 14)) >> 14
    }
    const fn set_sz(&mut self, sz: Size) {
        self.key |= match sz {
            Size::Any | Size::Unknown => 0b0000,
            Size::Byte => 0b0001,
            Size::Word => 0b0010,
            Size::Dword => 0b0011,
            Size::Qword => 0b0100,
            Size::Xword => 0b0101,
            Size::Yword => 0b0111,
        } << 2
            << 8;
    }
    const fn get_sz(&self) -> Size {
        match (self.key & 0b00_1111_00 << 8) >> 8 >> 2 {
            0b0000 => Size::Any,
            0b0001 => Size::Byte,
            0b0010 => Size::Word,
            0b0011 => Size::Dword,
            0b0100 => Size::Qword,
            0b0101 => Size::Xword,
            0b0111 => Size::Yword,
            _ => Size::Unknown,
        }
    }
}

const IS_CORRECT: u8 = 0x0;
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
pub struct OperandSet {
    ptable_data: u32,
    ptable_meta: u16,
    flags: Flags8,
    _reserved: u8,
    keys: Shared<[Key]>,
}

impl OperandSet {
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
            AType::Memory(_, _) => {
                if self.has_mem() {
                    MEM_TYPE
                } else {
                    return false;
                }
            }
            AType::None => return false,
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
            let aty = key.dec().expect("Key should be correct!");
            if aty == rhs {
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
    pub fn new(ndd: bool, ats: &[AType]) -> Self {
        let mut flags = Flags8::new();
        flags.set(IS_CORRECT, true);
        flags.set(OPR_NEEDED, ndd);
        let mut keys = Vec::with_capacity(ats.len());

        let mut ptable_data = 0;
        let mut ptable_meta = 0;
        let _reserved = 0;

        let mut pvtype = 0;

        let mut lttype = 0;

        let mut ptindx = 0;
        let mut slsize = 0;

        for key in ats {
            let crtype = match key {
                AType::Register(_, _) => {
                    flags.set(HAS_REG, true);
                    REG_TYPE
                }
                AType::Memory(_, _) => {
                    flags.set(HAS_MEM, true);
                    MEM_TYPE
                }
                AType::Immediate(_, _) => {
                    flags.set(HAS_IMM, true);
                    IMM_TYPE
                }
                AType::None => 0b00,
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
            keys.push(Key::enc(*key));
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
            keys: keys.into(),
        }
    }
}

pub struct CheckAPI<const OPERAND_COUNT: usize> {
    allowed: SmallVec<OperandSet, OPERAND_COUNT>,

    // less commonly used, so behind a pointer
    forbidden: MaybeUninit<Shared<[[Key; OPERAND_COUNT]]>>,
    additional: MaybeUninit<Shared<[Mnemonic]>>,

    // first bit is if has additional mnemonics
    // second bit is if has forbidden operand combo
    // lower 2 bits are check mode
    flags: u8,
}

#[repr(u8)]
pub enum CheckMode {
    NONE = 0b00,   // don't check for size diff
    AVX = 0b01,    // don't check for size diff
    X86 = 0b10,    // check for size
    NOSIZE = 0b11, // don't check for size diff
}

impl<const OPERAND_COUNT: usize> CheckAPI<OPERAND_COUNT> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            allowed: SmallVec::new(),
            flags: CheckMode::X86 as u8,
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
    pub fn set_forb(mut self, forb: &[[AType; OPERAND_COUNT]]) -> Self {
        self.set_forb_flag();
        let mut vec: Vec<[Key; OPERAND_COUNT]> = Vec::with_capacity(forb.len());
        let mut smv: SmallVec<Key, OPERAND_COUNT> = SmallVec::new();
        for f in forb {
            for t in f {
                smv.push(Key::enc(*t));
            }
            while smv.len() < OPERAND_COUNT {
                smv.push(Key::blank());
            }
            let mut slc = [Key::blank(); OPERAND_COUNT];
            for (slp, k) in smv.iter().enumerate() {
                slc[slp] = *k;
            }
            vec.push(slc);
            smv.clear();
        }
        self.forbidden = MaybeUninit::new(Shared::from(vec));
        self
    }
    pub fn set_addt(mut self, addt: &[Mnemonic]) -> Self {
        self.set_addt_flag();
        self.additional = MaybeUninit::new(addt.into());
        self
    }
    pub fn pushop(mut self, types: &[AType], needed: bool) -> Self {
        self.allowed.push(OperandSet::new(needed, types));
        self
    }
    pub fn getops(&self, idx: usize) -> Option<&OperandSet> {
        self.allowed.get(idx)
    }
    pub fn check_addt(&self, ins: &Instruction) -> Result<(), Error> {
        if let Some(found) = ins.addt {
            if self.has_addt() {
                let mut f = false;
                for allowed in &**unsafe { self.additional.assume_init_ref() } {
                    if allowed == &found {
                        f = true;
                        break;
                    }
                }
                if !f {
                    return Err(Error::no_tip(Some(ins.line), Some("Tried to use prefix mnemonic, but used instruction does not allow for this one")));
                }
            } else {
                return Err(Error::no_tip(
                    Some(ins.line),
                    Some("Tried to use prefix mnemonic, but used instruction does not permit it"),
                ));
            }
        }
        Ok(())
    }
    pub fn check_forb(&self, ins: &Instruction) -> Result<(), Error> {
        if !self.has_forb() {
            return Ok(());
        }
        let mut smv: SmallVec<AType, OPERAND_COUNT> = SmallVec::new();
        for o in ins.oprs.iter() {
            smv.push(o.atypen());
        }

        let mut at = 0;
        for f in unsafe { self.forbidden.assume_init_ref() }.iter() {
            for (i, k) in f.iter().enumerate() {
                if let Some(k) = k.dec() {
                    if Some(&k) == smv.get(i) {
                        at += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            if at == smv.len() {
                return Err(Error::no_tip(
                    Some(ins.line),
                    Some("Tried to use forbidden operand combination!"),
                ));
            }
        }

        Ok(())
    }
    pub fn check(&self, ins: &Instruction) -> Result<(), Error> {
        self.check_addt(ins)?;
        for (i, o) in self.allowed.iter().enumerate() {
            if let Some(s) = ins.get_opr(i) {
                if !o.has(s.atypen()) {
                    return Err(Error::no_tip(
                        Some(ins.line),
                        Some(format!("Invalid operand type at index {i}")),
                    ));
                }
            } else {
                if o.is_optional() {
                    break;
                } else {
                    return Err(Error::no_tip(
                        Some(ins.line),
                        Some(format!(
                            "Internal error: expected operand set at index {i}, found none"
                        )),
                    ));
                }
            }
        }
        match self.get_mode() {
            CheckMode::NONE | CheckMode::AVX | CheckMode::NOSIZE => {}
            CheckMode::X86 => {
                let mut sz = Size::Unknown;
                for o in ins.oprs.iter() {
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
                        return Err(Error::no_tip(
                            Some(ins.line),
                            Some("Tried to use immediate that is too large for other operands!"),
                        ));
                    }
                    if let Some(r) = o.get_reg() {
                        if sz != r.size() && !r.is_dbg_reg() && !r.is_ctrl_reg() && !r.is_sgmnt() {
                            return Err(Error::no_tip(
                                Some(ins.line),
                                Some("Invalid operand size was used in this instruction!"),
                            ));
                        }
                    }
                }
            }
        }
        if ins.oprs.len() < OPERAND_COUNT {
            return Err(Error::no_tip(
                    Some(ins.line),
                    Some(format!("Provided too little operands for instruction: found {}, expected {OPERAND_COUNT} max", ins.oprs.len())))
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod chkn_test {
    use super::*;
    #[test]
    fn key_test() {
        let k = AType::Register(Register::EDX, true);
        assert_eq!(
            Key::enc(k).dec(),
            Some(AType::Register(Register::EDX, true))
        );
        let k = AType::Register(Register::BL, false);
        assert_eq!(
            Key::enc(k).dec(),
            Some(AType::Register(Register::BL, false))
        );
    }
    #[test]
    fn ops_test() {
        assert_eq!(1 << 1, 1 * 2);
        assert_eq!(0 << 1, 0);
        assert_eq!(REG_TYPE << (0 << 1), 0b01);
        assert_eq!(MEM_TYPE << (1 << 1), 0b1000);
        assert_eq!((MEM_TYPE << (1 << 1)) | (REG_TYPE << (0 << 1)), 0b1001);
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
            Shared::from([
                Key::enc(AType::Register(Register::AL, false)),
                Key::enc(AType::Register(Register::BX, false)),
                Key::enc(M16),
                Key::enc(I8),
            ])
        );
        assert_eq!(o.ptable_meta, 0b0000_0000_0011_1001);
        assert_eq!(o.ptable_data, 0b0001_0001_0010);
        assert_eq!(o.has(AType::Register(Register::BX, false)), true);
        assert_eq!(o.has(M16), true);
        println!("{:?}", o.keys);
        assert_eq!(o.has(I8), true);
    }
    #[test]
    fn chk_test() {
        let t = Key::enc(Operand::Reg(Register::RAX).atypen())
            .dec()
            .unwrap();
        assert_eq!(t, AType::Register(Register::RAX, false));
    }
}
