<div align=center>
    <h1>rasm</h1>
</div>

## about

rasmx86-64 (or just rasm) is assembler for x86-64 architecture.

> [!WARNING]
> rasm is still in early development and has [tests](tests), but there can be edge-cases not covered by them.

## roadmap

> [!NOTE]
> This roadmap is not final and may (will) change.

- alpha
    - [x] MVP
    - [x] Better variable support
    - [x] Support for 64-bit ELF
    - [x] Support for 32-bit (`protected`) and 16-bit (`real`) modes; `cr`, `dr`, `eflags` and `segments` (`cs`, `fs`, etc.)
    - [x] Support for: `SSE`, `SSE2`, `SSE3`, `SSSE3`, `SSE4_1`, `SSE4_2`, `MMX` x86(-64) extensions
    - [x] Support for `AVX` and `AVX2` extensions
    - [x] moving into beta phase (release of beta0)...
- beta
    - betaf (beta foundation)
        - [x] Support for most of "normal" (to norm-part6) x86-64 instructions
        - [x] Transforming `Mem` enum into struct
        - [x] Parser support for closures `()` other than memory address
        - [x] Support for "modifiers" that is: `base:mod1:mod2`
        - [x] Support for comptime mathematical evaluations (`$()` closure)
        - [x] Support for constant user defined mathematical values (that aren't symbols, but inline immediates)
        - [x] Improved segmentation (allow prefixing with `%` and free up `#` prefix)
        - [x] Support for includes 
        - [x] Support for label attributes (`#()` closure)
        - [x] Migration (from legacy `*gen_ins`) to new codegen API (`GenAPI` struct)
        - [x] Fix OSOP and ASOP prefixes (Operand/Address Size Override Prefix (for memory))
        - [x] Optimize `Instruction` struct
    - betao (beta obj)
        - [ ] Variables overhaul (labels as variables)
        - [ ] Extended Relocations/Symbols (`@()` closure + multiple relocation types; support for `section=name` attribute); relocation overhaul
        - [ ] Tests for relocations and other things
        - [ ] Better target handling (ELF reworked)
        - [ ] Basic support for DWARF
    - [ ] Create documentation
    - [ ] Overall polish
    - [ ] moving into beta-avx512 phase...
- beta-avx512
    - [ ] EVEX
    - [ ] AVX-512F
    - [ ] AVX-512VL, AVX-512DQ, AVX-512BW
    - [ ] AVX-512CD, AVX-512ER, AVX-512PF
    - [ ] VBMI2, VBMI, BITALG
    - [ ] GFNI, VAES, VPCLMULQDQ
    - [ ] IFMA, 4VNNIW, VNNI, 4FMAPS
    - [ ] AVX-FP16
    - [ ] other (VPOPCNTDQ, VPCLMULQDQ)
    - [ ] moving into beta-macro phase...
- beta-macro
    - [ ] Support for inline (or not) macros with C-like syntax
    - [ ] (idea - not certain) Support for pseudo functions (ability to use `.if`/`.loop`, etc.) (extended macros)
    - [ ] Support for custom opcodes (using assembler's API) (to support unsupported instructions)
    - [ ] Create documentation for macros
- beta-fpu
    - [ ] Support for x87 ISA (mostly instructions prefixed with `F`)
- stable
    - [ ] Stable Version `*-stable0`

## getting started

See [docs/syntax.md](docs/syntax.md)

## credits

made by matissoss [matissossgamedev@proton.me]

licensed under MPL 2.0
