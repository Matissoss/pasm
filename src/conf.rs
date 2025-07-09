// pasm - src/conf.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

// LINE_WIDTH is used in src/main.rs:print_supported_instructions
// --------------------------------------------------------------
// default = 25
pub const LINE_WIDTH: usize = 25;

// CLOSURE_START and CLOSURE_END defines character that will begin/end closures
// -------------------------------------
// default : CLOSURE_START = '(', CLOSURE_END = ')'
pub const CLOSURE_START: char = '(';
pub const CLOSURE_END: char = ')';

// subexpressions like in Intel documents:
// {subexpr}
// default: SUBEXPR_START = '{'; SUBEXPR_CLOSE = '}';
pub const SUBEXPR_START: char = '{';
pub const SUBEXPR_CLOSE: char = '}';

//  ---------------------------------------------------------------------------
//                              PREFIXES WARNING:
//      assembler is programmed to recognize different values, so do not
//      be shocked if you set every PREFIX* to same value and it does not work.
//  ---------------------------------------------------------------------------

//  PREFIX_REG defines prefix for registers
//  ---------------------------------------
//  default = '%'
pub const PREFIX_REG: char = '%';

//  PREFIX_VAL defines prefix for constant/immediate values (like 10, 0xFF, 0b1001, 'h')
//  ---------------------------------------
//  default = '$'
pub const PREFIX_VAL: char = '$';

//  PREFIX_REF defines prefix for referencing symbol
//  ---------------------------------------
//  default = '@'
pub const PREFIX_REF: char = '@';

// metadata for help

pub const BIN: &str = "pasm";
pub const VER: &str = "25.06-beta2";
