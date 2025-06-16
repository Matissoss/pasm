<div align=center>
    <h1>docs</h1>
</div>

> [!WARNING]
> Documentation is under work

- Do you want to use `rasm` assembler?

If yes, see [syntax-std.md](syntax-std.md) for casual and [syntax-avx.md](syntax-avx.md) for AVX-512.

- Do you want to extend `rasm`?

If yes, see [extending-rasm.md](extending-rasm.md)

- Are you developer of your own assembler (or just looking for x86 machine code encoding)?

See:

- [encoding-x86.md](encoding-x86.md) : for standard encoding (MODRM, REX, etc.)
- [encoding-vex.md](encoding-vex.md) : for encoding VEX (for AVX)
- [encoding-avx.md](encoding-avx.md) : for encoding EVEX (for AVX-512)
- [target-elf.md](target-elf.md) : for ELF target support (most essential things)

## dev roadmap

- for users
    - [x] `syntax-std.md` (related to `beta`)
    - [ ] `syntax-avx.md` (related to `beta-avx`)
    - [ ] `macro.md` (related to `beta-macro`)
- for developers
    - [x] `extending-rasm.md` (short explaination of RASM's API and how to use it)
    - `encoding-x86.md` (related to `beta`)
        - [ ] Introduction to x86-64 encoding
        - [ ] ModRM
        - [ ] REX
        - [ ] SIB and displacement
        - [ ] Offset
        - [ ] Different edge cases and how to handle them
        - [ ] Some examples (how I implemented it)
        - [ ] How to read opcode table?
    - `encoding-vex.md` (related to `beta`)
        - [ ] How to read opcode table with VEX?
        - [ ] Encoding VEX (when to use 2 byte VEX, when 3 byte one and more)
    - `encoding-avx.md` (related to `beta-avx`)
        - [ ] How to use AVX-512 (or simillar) in RASM
        - [ ] How to read opcode table and encode EVEX

