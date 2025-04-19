<div align=center>
    <h1>rasm</h1>
</div>

## about

rasmx86-64 (or just rasm) is assembler for x86-64 architecture.

> [!WARNING]
> rasm is still in early development and is not well tested (***there still may be edge cases, but basic usage is functional***). 
>
> ***Use with caution!***

## roadmap

- [x] Frontend for assembler (tokenizer, lexer, parser)
- [x] AST
- [x] Instruction encoding support (with REX, ModRM, registers, SIB, displacement and immediates) for most essential instructions
- [x] Basic support for relocations (for flat binaries)
- [x] Most basic support for `.o` output file.
- [ ] Supporting `.data`, `.bss`, `.rodata` and `.rel.*` sections in `.o` file
- [ ] Releasing MVP version
- [ ] Supporting basic optimisations (dead code elimination)
- [ ] Supporting atleast 10% of x86-64 "normal" instructions
- [ ] Full support for `real` (16-bit) and `protected` (32-bit) mode; cr, dr, eflags and segments (cs, fs, etc.) registers
- [ ] Support for AVX and SSE x86-64 extensions with VEX prefix (xmm0-15, ymm0-15 registers and legacy FPU).
- [ ] Supporting atleast 70% of x86-64 instructions

## error messages

`rasm` wants to made sure that error messages are as clear to end developer as possible. 
If some error message is, then open an issue, share context and I'll try to fix that :)

## credits

made by matissoss [matissossgamedev@proton.me]

licensed under MPL 2.0
