//  rasmx86_64 - src/conf.rs
//  --------------------------
//  made by matissoss
//  licensed under MPL 2.0

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

// MEM_START and MEM_CLOSE defines character that will begin/end memory address
// p.e. (if MEM_START == '(' and MEM_CLOSE == ')'): mov %rax, !dword (%rsi-$4)
// means that: destination = %rsi - $4
// -------------------------------------
// default : MEM_START = '(', MEM_CLOSE = ')'
pub const MEM_START: char = '(';
pub const MEM_CLOSE: char = ')';

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

//  PREFIX_REF defines prefix for constant values (.bss and .data references)
//  ---------------------------------------
//  default = '@'
pub const PREFIX_REF: char = '@';

//  PREFIX_KWD defines prefix for keywords like `!global`
//  ---------------------------------------
//  default = '!'
pub const PREFIX_KWD: char = '!';
