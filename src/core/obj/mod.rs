// TODO: fix this bug
// I literally don't have idea why this doesn't work
// why tf mold doesn't work on generated *.o files, but ld linker (throws warning that entry symbol
// is not defined) does?
// and tf is wrong with entry and why is it 0x401000 on default in ld?
pub mod elf32;
pub mod elf64;
