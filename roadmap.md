<div align=center>
    <h1>roadmap.md</h1>
</div>

## alpha

- [x] MVP
- [x] Better variable support
- [x] Support for 64-bit ELF
- [x] Support for 32-bit (protected) and 16-bit (real) modes; cr, dr, eflags and segments (cs, fs, etc.)
- [x] Support for: SSE, SSE2, SSE3, SSSE3, SSE4_1, SSE4_2, MMX x86(-64) extensions
- [x] Support for AVX and AVX2 extensions
- [x] moving into beta phase (release of beta0)...

## beta

### betaf (beta foundation)

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

### betao (beta obj)

- [x] Variables overhaul (labels as variables)
- [x] Switch `!` prefix for keywords for `.`
- [x] Relocation/Symbol overhaul (use of `@()` closure)
- [x] Tests for relocations and other things
- [x] Better target handling (ELF rewritten from zero)
- [x] Support for custom sections (with `.section` keyword)

### beta

- [x] Create documentation (including better readme ;))
- Overall polish:
    - [x] Allow for `type` in labels attributes.
    - [x] Make sections also symbols.
    - [x] Allow for use of `PREFIX_VAL` in `ExtSymbolRef`.
    - [x] Move some of logic in `src/main.rs` to separate file.
    - [x] Mem support for (R)IP (addresing like `($10)`)
    - [x] Allow for different size relocations (`relXX` and `absXX`)
    - [x] Fix `in`, `lea`, `wrf/gsbase`, `loopXX` and `out` instructions.
    - [x] Add missing `lgdt` and `lidt` instructions.
    - [x] Allow using `lock`, `repXX` mnemonics as prefix
    - [x] Allow for using symbols/relocations in more than one place (as address or value under address; `.deref`/`.ref` directives)
    - [x] Remove `.assemble()` from `match` statement in `src/core/comp.rs:compile_instruction` and move inside `compile_label`
    - [x] Create `.debug_assemble()` (`--debug`)
    - [x] Limit lifetimes, where they are unnecessary (use owned values)
    - [x] Replace `String`s with `Arc<str>`/`Rc<str>`.
    - [x] Allow for usage of multi threads to compile labels/sections
    - [x] Revamp `AType` and `src/pre/chk.rs` in favor of `CheckAPI` (like is with `GenAPI`)
    - [x] Revamp errors (better readability) with explainations
    - [x] Create `src/docs/errors.md` and `--explain=[ECD]`
- [x] moving into beta-avx phase...

## beta-avx

Goal: implement most of AVX based (E)VEX instructions

- [x] Prepare `pasm`'s syntax for AVX-512
- [x] Prepare backend for AVX-512 (add `IS_BCST` flag to Mem, add support for masks, etc. in parser)
- [x] implement EVEX support
- ISA implementation (divided in 15 parts):
    - [x] avx-ext-0
    - [x] avx-ext-1
    - [x] avx-ext-2
    - [x] avx-ext-3
    - [x] avx-ext-4
    - [x] avx-ext-5
    - [x] avx-ext-6
    - [x] avx-ext-7
    - [x] avx-ext-8
    - [x] avx-ext-9
    - [x] avx-ext-a
    - [x] avx-ext-b
    - [x] avx-ext-c
    - [x] avx-ext-d
    - [x] avx-ext-e (vsib)
- [x] Move to beta-intel-apx

## beta-intel-apx

Goal: implement support for Intel APX

- [x] cleanup `Instruction` in `src/shr/ast.rs` (it is barely readable) and prepare it for Intel APX
- [ ] cleanup `CheckAPI` in `src/pre/chkn.rs` for better readability, performance and preparations for Intel APX
- [ ] implement support for APX in syntax
- [ ] implement EEVEX (extended EVEX; all 4 variants) and REX2 prefixes support
- ISA implementantation (divided in 8 parts; legacy instructions included):
    - [ ] intel-apx-0 (legacy instructions without setcc)
    - [ ] intel-apx-1 (ccmpcc)
    - [ ] intel-apx-2 (cfcmovcc)
    - [ ] intel-apx-3 (ctestcc)
    - [ ] intel-apx-4 (setcc)
    - [ ] intel-apx-6 (cmpccxadd)
    - [ ] intel-apx-5 (push2/pop2)
    - [ ] intel-apx-7 (evex extended evex instructions and evex extended vex instructions)
- [ ] Move to beta-min

## beta-min

Goal: support for smaller x86-64 ISA extensions

- [ ] CET_SS (Shadow Stack)
- [ ] VMX
- [ ] SMX
- [ ] SGX
- [ ] GFNI
- [ ] BMI*
- [ ] x87 ISA
- [ ] missing x86-64 instructions
- [ ] move to rc

## rc

Goal: extensive testing, polish and optimizations of assembler, less updates/commits

- [ ] Support for 16-bit addressing
- [ ] Add `type` directive for `section`s (allows to have `.bss` sections)
- [ ] Allow for `protected public function label_name:`
- [ ] Support for `offset` (aka `ORG`) directive
- [ ] Allow for long jumps (`jmp ptrXX:YY` and `jmp m16:XX`)
- [ ] Rework `src/shr/math.rs` (it is not effective currently) and allow for symbol referencing inside `$()` closure
- [ ] Allow for usage of memory addressing without size directive
- [ ] Better support of `include` directive (test it)
- [ ] Support for anonymous labels (like in FASM `@@:`) and references like `@previous`, `@next` (reserved relocation names)
- [ ] Allow for symbol referencing with section specifier
- [ ] Allow for `gotpcrel` relocation type
- [ ] Support for `repeatX` instruction (custom one; aka `times` in other assemblers)
- [ ] Support for `align` instruction
- [ ] Support for `fixedsize` directive (throws an error, if buffer len is gt fixedsize)
- [ ] move to stable

## stable

- [ ] release of `stable` version
