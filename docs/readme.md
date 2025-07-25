<div align=center>
    <h1>docs</h1>
</div>

- Do you want to use `pasm` assembler?

If yes, see [syntax-std.md](syntax-std.md) for casual and [syntax-avx512.md](syntax-avx512.md) for AVX-512.

- Do you want to extend `pasm`?

If yes, see [extending-pasm.md](extending-pasm.md)

- Are you developer of your own assembler (or just looking for x86 machine code encoding)?

See:

- [encoding-x86.md](encoding-x86.md) : for standard encoding (MODRM, REX, etc.)
- [encoding-vex.md](encoding-vex.md) : for encoding VEX (for AVX and VEX-based extensions)
- [encoding-evex.md](encoding-evex.md): for encoding EVEX (for AVX-512)

## dev roadmap

- for users
    - [x] `syntax-std.md` (related to `beta`)
    - [x] `syntax-avx512.md` (related to `beta-avx`)
    - [x] `syntax-apx.md` (related to `beta-intel-apx`)
- for developers
    - [x] `extending-pasm.md` (short explaination of PASM's API and how to use it)
    - [x] `encoding-x86.md` (related to `beta`)
    - [x] `encoding-vex.md` (related to `beta`)
    - [x] `encoding-evex.md` (related to `beta-avx`)
