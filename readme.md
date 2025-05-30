<div align=center>
    <h1>rasm</h1>
</div>

## about

rasmx86-64 (or just rasm) is assembler for x86-64 architecture.

> [!WARNING]
> rasm is still in early development and has [tests](tests), but there can be edge-cases not covered by them.

## roadmap

> [!NOTE]
> This roadmap is not final and may (and will) change.

- alpha
    - [x] MVP
    - [x] Better variable support
    - [x] Support for 64-bit ELF
    - [x] Support for 32-bit (`protected`) and 16-bit (`real`) modes; `cr`, `dr`, `eflags` and `segments` (`cs`, `fs`, etc.)
    - [x] Support for: `SSE`, `SSE2`, `SSE3`, `SSSE3`, `SSE4_1`, `SSE4_2`, `MMX` x86(-64) extensions
    - [x] Support for `AVX` and `AVX2` extensions
- beta
    - [x] Support for most of "normal" (to norm-part6) x86-64 instructions
    - [ ] Extended Relocations/Symbols, Support for comptime mathematical evaluations, imports/includes and label attributes
    - [ ] Tests for relocations and other things
    - [ ] Improved segmentation
    - [ ] Overall polish
- beta-avx512
    - [ ] AVX-512F
    - [ ] AVX-512VL, AVX-512DQ, AVX-512BW
    - [ ] AVX-512CD, AVX-512ER, AVX-512PF
    - [ ] VBMI2, VBMI, BITALG
    - [ ] GFNI, VAES, VPCLMULQDQ
    - [ ] IFMA, 4VNNIW, VNNI, 4FMAPS
    - [ ] AVX-FP16
    - [ ] other (VPOPCNTDQ, VPCLMULQDQ)
- beta-macro
    - [ ] Support for inline (or not) macros with C-like syntax
    - [ ] Support for pseudo functions (ability to use `.if`/`.loop`, etc.) (extended macros)
    - [ ] Support for local (and global) scope aliases
    - [ ] Support for custom opcodes (using assembler's API) (to support unsupported instructions)
- beta-fpu
    - [ ] Support for x87 ISA (mostly instructions prefixed with `F`)
- stable
    - [ ] Stable Version `*-stable0`

## getting started

See [docs/syntax.md](docs/syntax.md)

## credits

made by matissoss [matissossgamedev@proton.me]

licensed under MPL 2.0
