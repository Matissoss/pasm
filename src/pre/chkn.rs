// rasmx86_64 - src/pre/chkn.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(unused)]

use crate::conf::Shared;

use crate::shr::{
    booltable::BoolTable8 as Flags8,
    ins::Mnemonic,
    reg::{Purpose as RPurpose, Register},
    size::Size,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AType {
    //                fixed register
    Register(Register, bool),
    //     size|address size  (registers used)
    Memory(Size, Size),
    //              is_string
    Immediate(Size, bool),
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
#[repr(transparent)]
pub struct Key {
    key: u16,
}

const REG_TYPE: u16 = 0b01;
const MEM_TYPE: u16 = 0b10;
const IMM_TYPE: u16 = 0b11;

impl Key {
    pub const fn enc(at: AType) -> Self {
        let mut toret = Self { key: 0 };
        toret.set_opt(match at {
            AType::Memory(_, _) => MEM_TYPE,
            AType::Register(_, _) => REG_TYPE,
            AType::Immediate(_, _) => IMM_TYPE,
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
    pub const fn dec(&self) -> Option<AType> {
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
                if rpr == 0b0000 {
                    Some(AType::Register(Register::__ANY, is_fixed))
                } else {
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
            Some(match self.key | (0b111 << 5) {
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
        match (self.key & 0b00_1111_00 << 8) >> 2 {
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
    pub fn has(&self, rhs: AType) -> bool {
        let (sz, tp) = match rhs {
            AType::Register(r, _) => {
                if self.has_reg() {
                    (r.size(), REG_TYPE)
                } else {
                    return false;
                }
            }
            AType::Immediate(i, _) => {
                if self.has_imm() {
                    (i, IMM_TYPE)
                } else {
                    return false;
                }
            }
            AType::Memory(m, _) => {
                if self.has_mem() {
                    (m, MEM_TYPE)
                } else {
                    return false;
                }
            }
        };
        let mut idx = 0;
        let mut off = 0;
        let mut lsz = 0;
        loop {
            let crt = (self.ptable_meta & (0b11 << (idx << 1))) >> (idx << 1);
            if crt == 0 {
                break;
            } else if crt == tp {
                let lix = ((idx + 1) << 2);
                let and = 0b1111 << lix;
                lsz = (self.ptable_data & and) >> lix;
                lsz += 1;
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
            match (aty, rhs) {
                (AType::Register(lr, lf), AType::Register(rr, rf)) => {
                    if lf || rf {
                        if lr == rr {
                            return true;
                        }
                    } else {
                        if lr.is_any() || rr.is_any() {
                            if lr.size() == rr.size() {
                                return true;
                            }
                        } else if lr.purpose() == rr.purpose() && lr.size() == rr.size() {
                            return true;
                        }
                    }
                }
                (AType::Memory(lsz, laddr), AType::Memory(rsz, raddr)) => {
                    if lsz == rsz && laddr == raddr {
                        return true;
                    }
                }
                (AType::Immediate(lsz, ls), AType::Immediate(rsz, rs)) => {
                    if ls && rs {
                        return true;
                    } else {
                        if rsz < lsz {
                            return true;
                        }
                    }
                }
                _ => continue,
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
    allowed: [OperandSet; OPERAND_COUNT],
    // TODO:
    // forbidden: [],
    // addtional: [Mnemonic]
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
            Some(AType::Register(Register::AX, false))
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
                AType::Memory(Size::Dword, Size::Qword),
            ],
        );
        assert_eq!(o.ptable_meta, 0b0000_0000_0000_1001);
        assert_eq!(o.ptable_data, 0b0001_0010);
        assert_eq!(o.has(AType::Register(Register::BX, false)), true);
    }
}
