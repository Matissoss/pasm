<div align=center>
    <h1>rasm</h1>
</div>

## about

rasmx86-64 (or just rasm) is assembler for x86-64 architecture.

> [!WARNING]
> rasm is still in early development and has [tests](tests), but there can be edge-cases not covered by them.

## roadmap

- alpha
    - [x] MVP
    - [x] Better variable support
    - [x] Support for 64-bit ELF
    - [x] Support for 32-bit (`protected`) and 16-bit (`real`) modes; `cr`, `dr`, `eflags` and `segments` (`cs`, `fs`, etc.)
    - [x] Support for: `SSE`, `SSE2`, `SSE3`, `SSSE3`, `SSE4_1`, `SSE4_2`, `MMX` x86(-64) extensions
    - [x] Support for `AVX` and `AVX2` extensions
- beta
    - [ ] Support for most of (atleast 50%) "normal" x86-64 instructions
    - [ ] Extended Relocations/Symbols, Support for comptime mathematical evaluations and label attributes
- beta-avx512
    - [ ] AVX-512F
    - [ ] AVX-512VL, AVX-512DQ, AVX-512BW
    - [ ] AVX-512CD, AVX-512ER, AVX-512PF
    - [ ] VBMI2, VBMI, BITALG
    - [ ] GFNI, VAES, VPCLMULQDQ
    - [ ] IFMA, 4VNNIW, VNNI, 4FMAPS
    - [ ] AVX-FP16
    - [ ] other (VPOPCNTDQ, VPCLMULQDQ)
- stable
    - [ ] Stable Version `*-stable0`

## getting started

See [docs/syntax.md](docs/syntax.md)

## credits

made by matissoss [matissossgamedev@proton.me]

licensed under MPL 2.0
