//  rasmx86_64  -  conf.rs
//  ----------------------
//  made by matissoss
//  licensed under MPL 2.0


// FAST_MODE means that assembler will skip 
// the *_ie part and return the value immediately;
// useful for scenarios where you don't need 100% accuracy 
// and can allow for some heurestics for efficiency (like compilers)
// --------------------------------------------------------
// default = false
pub const FAST_MODE : bool = false;

// MEM_START and MEM_CLOSE defines character that will begin/end memory address
// p.e. (if MEM_START == '(' and MEM_CLOSE == ')'): movd %rax, dword (%rsi-$4)
// means that: destination = %rsi - $4 
// -------------------------------------
// default : MEM_START = '(', MEM_CLOSE = ')'
pub const MEM_START : char = '(';
pub const MEM_CLOSE : char = ')';

// COMMENT_S or in other words: comment start
// ------------------------------------------
// default = ';'
pub const COMMENT_S : char = ';';

//  ---------------------------------------------------------------------------
//                              PREFIXES WARNING:
//      assembler is programmed to recognize different values, so do not
//      be shocked if you set every PREFIX* to same value and it does not work.
//  ---------------------------------------------------------------------------

//  PREFIX_REG defines prefix for registers
//  ---------------------------------------
//  default = '%'
pub const PREFIX_REG : char = '%';

//  PREFIX_VAL defines prefix for constant/immediate values (like 10, 0xFF, 0b1001, 'h')
//  ---------------------------------------
//  default = '$'
pub const PREFIX_VAL : char = '$';

//  PREFIX_REF defines prefix for constant values (.bss and .data references)
//  ---------------------------------------
//  default = '@'
pub const PREFIX_REF : char = '@';

//  PREFIX_LAB defines prefix for labels/functions
//  ---------------------------------------
//  default = '&'
pub const PREFIX_LAB : char = '&';

pub const PREFIX_KWD : char = '!';
