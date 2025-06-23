// rasmx86_64 - src/pre/chkn.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(unused)]

use crate::conf::Shared;

use crate::shr::{
    booltable::BoolTable8 as Flags8,
    ins::Mnemonic,
    reg::{Register, Purpose as RPurpose},
    size::Size
};

pub enum AType {
    //                fixed register
    Register(Register, bool),
    //     size|address size  (registers used)
    Memory(Size, Size),
    //              is_string
    Immediate(Size, bool),
}

impl Into<Key> for AType {
    fn into(self) -> Key {
        Key::new(self)
    }
}

// key:
// 1 byte: 0bXX_YYYY_ZZ
// XX   - operand type:
//          00 - Invalid
//          01 - Register
//          10 - Memory
//          11 - Immediate
// YYY  - size:
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
    pub const fn new(at: AType) -> Self{
        let mut toret = Self {key: 0};
        toret.set_opt(match at {
            AType::Memory(_, _) => MEM_TYPE,
            AType::Register(_,_)=> REG_TYPE,
            AType::Immediate(_,_) => IMM_TYPE,
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
                Size::Word  => 0b001,
                Size::Dword => 0b010,
                Size::Qword => 0b011,
                _           => 0b000,
            };
            toret.key |= addrsz << 5;
        } else if let AType::Immediate(sz, is_string) = at {
            toret.set_sz(sz);
            toret.set_zz((is_string as u16) << 1);
        }
        toret
    }
    pub const fn de(&self) -> Option<AType> {
        match self.get_opt() {
            MEM_TYPE => {
                let asz = self.get_addrsz().expect("Address size should be Some in memory!");
                let msz = self.get_sz();
                Some(AType::Memory(msz, asz))
            },
            REG_TYPE => {
                let rpr = self.get_rpurpose().expect("Register purpose should be Some in registers!");
                let rsz = self.get_sz();
                let is_fixed = self.get_zz() & 0b10 == 0b10;
                let rcd = if is_fixed {
                    self.key & 0b0000_0111
                } else {
                    0
                };
                if rpr.is_any() {
                    Some(AType::Register(Register::__ANY, true))
                } else {
                    // TODO:
                    // replace with Register::de()
                    Some(AType::Register(Register::__ANY, true))
                }
            },
            IMM_TYPE => {
                Some(AType::Immediate(self.get_sz(), self.get_zz() & 0b10 == 0b10))
            },
            _ => None,
        }
    }
    const fn get_zz(&self) -> u16 {
        (self.key & (0b00_0000_11 << 8)) >> 8
    }
    const fn get_rpurpose(&self) -> Option<RPurpose> {
        if self.get_opt() == REG_TYPE {
            // scary :D
            Some(unsafe { std::mem::transmute::<u8, RPurpose>((self.key & 0x0F) as u8) })
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
                _     => Size::Unknown,
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
    pub const fn set_opt(&mut self, v: u16) {
        self.key |= v << 6 << 8;
    }
    // get operand type
    pub const fn get_opt(&self) -> u16 {
        self.key & (0b11 << 6 << 8)
    }
    pub const fn set_sz(&mut self, sz: Size) {
        self.key |= match sz {
            Size::Any | Size::Unknown => 0b0000,
            Size::Byte  => 0b0001,
            Size::Word  => 0b0010,
            Size::Dword => 0b0011,
            Size::Qword => 0b0100,
            Size::Xword => 0b0101,
            Size::Yword => 0b0111,
        } << 2 << 8;
    }
    pub const fn get_sz(&self) -> Size {
        match (self.key & 0b00_1111_00) >> 2 {
            0b0000 => Size::Any,
            0b0001 => Size::Byte,
            0b0010 => Size::Word,
            0b0011 => Size::Dword,
            0b0100 => Size::Qword,
            0b0101 => Size::Xword,
            0b0111 => Size::Yword,
            _ => Size::Unknown
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
//      - prediction table entry:
//          0bXXX: size of slice
//  5th: "prediction table" (PTable) metadata
//      - 0bXXYY_ZZAA:
//          - XX: operand type
//          - YY: operand type
//          - ZZ: operand type
//          - AA: operand type
//  6th: reserved
//  7th: Booltable8
//  8th: reserved
// Example:
//  let's assert that we have in our operand set key for array (it must be ordered!): [R32, R64, M32, M64].
//  Our checker will start by checking if we have registers.
//  We have HAS_REG set to 1, so we move on.
//  We then load PTable metadata and check for indexes.
//  In this case we search for Register Operand Type.
//  Let's assert that first 2 bits give a `Register` type.
//  Then we load next operand (in this case a `Memory` type).
//  Then we try to calculate slice size for prediction (in worst case we have to iterate over [u16; 6]).
//  Then we try to predict our index based on which key we are on. In this case we get `R32`.
//  Because of that we try to look for next operand,
//  we assert that it goes like: Nothing (or Other Type) -> R32 -> R64 -> Other Type.
//  That's why we use a heurestic and try to access next element (after R32: R64).
//  And we got it! we got R64, so it means our variant was correct.
//
//  If for some reason we couldn't find our variant using prediction, then we iterate over that
//  slice. If we still didn't found our operand, it means that combination was invalid.
pub struct OperandSet {
    ptable_data: u32,
    ptable_meta: u16,
    flags: Flags8,
    _reserved: u8,
    keys: Shared<[Key]>,
}

pub struct CheckAPI<const OPERAND_COUNT: usize> {
    allowed: [OperandSet; OPERAND_COUNT],
    // TODO:
    // forbidden: [],
    // addtional: [Mnemonic]
}
