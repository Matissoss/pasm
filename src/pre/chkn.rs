// rasmx86_64 - src/pre/chkn.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

use std::mem::MaybeUninit;

use crate::conf::Shared;

use crate::shr::{
    booltable::BoolTable8 as Flags8,
    ins::Mnemonic,
};

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
//              - X1: is extended register
//              - YYYY: purpose
//              - ZZZ: code of register
//      if memory:
//          0bXXX_YYYYY:
//              - XXX: address size
//              - YYYYY: reserved
//      if immediate:
//          0bXXXX_XXXX
//              - XXXX_XXXX: reserved
#[repr(transparent)]
pub struct Key {
    key: u16,
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
    keys: Shared<[u16]>,
}

pub struct CheckAPI {}
