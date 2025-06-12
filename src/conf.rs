//  rasmx86_64 - src/conf.rs
//  --------------------------
//  made by matissoss
//  licensed under MPL 2.0

// LINE_WIDTH is used in src/main.rs:print_supported_instructions
// --------------------------------------------------------------
// default = 25
pub const LINE_WIDTH: usize = 25;

// TIME defines if assembler will measure how long did it take to assemble a file.
// -------------------------------------
// default = true
pub const TIME: bool = true;

// FAST_MODE means that assembler will skip
// the *_ie part and return the value immediately;
// useful for scenarios where you don't need 100% accuracy
// and can allow for some heurestics for efficiency (like compilers)
// --------------------------------------------------------
// default = false
pub const FAST_MODE: bool = false;

// CLOSURE_START and CLOSURE_END defines character that will begin/end closures
// means that: destination = %rsi - $4
// -------------------------------------
// default : CLOSURE_START = '(', CLOSURE_END = ')'
pub const CLOSURE_START: char = '(';
pub const CLOSURE_END: char = ')';

// COMMENT_S or in other words: comment start
// ------------------------------------------
// default = ';'
pub const COMMENT_S: char = ';';

//  ---------------------------------------------------------------------------
//                              PREFIXES WARNING:
//      assembler is programmed to recognize different values, so do not
//      be shocked if you set every PREFIX* to same value and it does not work.
//  ---------------------------------------------------------------------------

//  PREFIX_REG defines prefix for segments
//  ---------------------------------------
//  default = '#'
pub const PREFIX_SEG: char = '#';

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

//  PREFIX_KWD defines prefix for keywords like `.global`
//  ---------------------------------------
//  default = '.'
pub const PREFIX_KWD: char = '.';
