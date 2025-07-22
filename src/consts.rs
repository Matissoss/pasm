// pasm - consts.rs
// ----------------
// made by matissoss
// licensed under MPL 2.0

// x86-64 conditions
pub const COND_O: u8 = 0b0000;
pub const COND_NO: u8 = 0b0001;
pub const COND_B: u8 = 0b0010;
pub const COND_C: u8 = 0b0010;
pub const COND_NAE: u8 = 0b0010;
pub const COND_NB: u8 = 0b0011;
pub const COND_NC: u8 = 0b0011;
pub const COND_AE: u8 = 0b0011;
pub const COND_E: u8 = 0b0100;
pub const COND_Z: u8 = 0b0100;
pub const COND_NE: u8 = 0b0101;
pub const COND_NZ: u8 = 0b0101;
pub const COND_BE: u8 = 0b0110;
pub const COND_NA: u8 = 0b0110;
pub const COND_NBE: u8 = 0b0111;
pub const COND_A: u8 = 0b0111;
pub const COND_S: u8 = 0b1000;
pub const COND_NS: u8 = 0b1001;
pub const COND_P: u8 = 0b1010;
pub const COND_PE: u8 = 0b1010;
pub const COND_NP: u8 = 0b1011;
pub const COND_PO: u8 = 0b1011;
pub const COND_L: u8 = 0b1100;
pub const COND_NGE: u8 = 0b1100;
pub const COND_NL: u8 = 0b1101;
pub const COND_GE: u8 = 0b1101;
pub const COND_LE: u8 = 0b1110;
pub const COND_NG: u8 = 0b1110;
pub const COND_G: u8 = 0b1111;
pub const COND_NLE: u8 = 0b1111;
