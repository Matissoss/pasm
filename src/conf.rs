// pasm - src/conf.rs
// ------------------
// made by matissoss
// licensed under MPL 2.0

pub type RString = Shared<str>;

#[cfg(feature = "mthread")]
pub type Shared<T> = std::sync::Arc<T>;
#[cfg(not(feature = "mthread"))]
pub type Shared<T> = std::rc::Rc<T>;

// CORE_LB_GROUP groups CORE_LB_GROUP lines to make assembling
// labels faster in multithreading.
// -----------
// default = 2
#[cfg(feature = "mthread")]
pub const CORE_LB_GROUP: usize = 2;

// TOK_LN_GROUP groups (TOK_LN_GROUP) lines to make tokenizer
// faster in multithreading.
// -----------
// default = 512
#[cfg(feature = "mthread")]
pub const TOK_LN_GROUP: usize = 512;

// default = 8
#[cfg(feature = "mthread")]
pub const THREAD_LIMIT: u8 = 8;

// default = 5
#[cfg(feature = "mthread")]
pub const RETRY_TIME_MS: u64 = 5;

pub const SMALLVEC_TOKENS_LEN: usize = 16;

// LINE_WIDTH is used in src/main.rs:print_supported_instructions
// --------------------------------------------------------------
// default = 25
pub const LINE_WIDTH: usize = 25;

// FAST_MODE means that assembler will skip
// the *_ie part and return the value immediately;
// useful for scenarios where you don't need 100% accuracy
// and can allow for some heurestics for efficiency (like compilers)
// --------------------------------------------------------
// default = false
pub const FAST_MODE: bool = false;

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

// metadata for help

pub const BIN: &str = "pasm";
pub const VER: &str = "25.06-beta0";
