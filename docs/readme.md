# rasm documentation

> [!WARNING]
> Documentation is under work

# development roadmap

- for users
    - `syntax.md` (related to `beta`)
        - [ ] Basic syntax
        - [ ] Closure explaination
        - [ ] Label attributes
    - `macro.md` (related to `beta-macro`)
- for developers
    - `extending-rasm.md` (short explaination of RASM's API and how to use it)
    - `std.md` (related to `beta`)
        - [ ] Introduction to x86-64 encoding
        - [ ] ModRM
        - [ ] REX
        - [ ] SIB and displacement
        - [ ] Offset
        - [ ] Different edge cases and how to handle them
        - [ ] Some examples (how I implemented it)
        - [ ] How to read opcode table?
    - `vex.md` (related to `beta`)
        - [ ] How to read opcode table with VEX?
        - [ ] Encoding VEX (when to use 2 byte VEX, when 3 byte one and more)
    - `avx512.md` (related to `beta-avx512`)
        - [ ] How to use it in RASM
        - [ ] How to read opcode table and encode EVEX

