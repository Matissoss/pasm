<div align=center>
    <h1>rasm</h1>
</div>

## about

rasmx86-64 (or just rasm) is assembler for x86-64 architecture.

## roadmap

- [x] Frontend for assembler (tokenizer, lexer, parser)
- [x] AST
- [ ] Instruction encoding support (with REX, ModRM, registers, SIB, displacement and immediates) for most essential instructions
- [ ] Most basic support for `.o` output file.
- [ ] Releasing MVP version
- [ ] Improving syntax (if needed; optional)
- [ ] Supporting basic optimisations (dead code elimination)
- [ ] Supporting atleast 10% of x86-64 instructions
- [ ] Support for `real` (16-bit) and `protected` (32-bit) mode; cr, dr, eflags and segments (cs, fs, etc.) registers
- [ ] Support for AVX and SSE x86-64 extensions with VEX prefix.
- [ ] Supporting atleast 70% of x86-64 instructions

## credits

made by matissoss [matissossgamedev@proton.me]

licensed under MPL 2.0
