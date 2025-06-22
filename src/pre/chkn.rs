// rasmx86_64 - src/pre/chkn.rs
// ----------------------------
// made by matissoss
// licensed under MPL 2.0

#![allow(unused)]

pub enum AType {
    Register(),
    Memory(),
    Immediate(),
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

const NEEDED: u8 = 0x0;
const HAS_IMM: u8 = 0x1;
const HAS_REG: u8 = 0x2;
const HAS_MEM: u8 = 0x3;

// metadata layout:
//  1 byte: Booltable8
//  2 byte: "prediction table" index
//      - 0bXXYY_ZZAA:
//          - XX: operand type
//          - YY: operand type
//          - ZZ: operand type
//          - AA: operand type
//  3 byte: reserved for future
pub struct OperandSet<const N: usize> {
    metadata: u64,
    keys: [u16; N],
}

pub struct CheckAPI {}
