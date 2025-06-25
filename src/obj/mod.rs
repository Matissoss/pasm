// pasm - src/obj/mod.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0

#[cfg(feature = "target_elf")]
pub mod elf;
#[cfg(feature = "target_elf")]
pub use elf::*;
