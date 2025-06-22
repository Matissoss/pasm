<div align=center>
    <h1>rasm</h1>
</div>

## about

`rasm` is modern and independent assembler for x86-64 architecture as one of key parts in (future project) `RAD` toolchain.

> [!NOTE]
> `rasm` is still in beta phase and should not be relied on for any serious work.

## features

> [!NOTE]
> This info is for `betao` release.

- Support for ~50%+ x86-64 ISA (~848 instructions)
- Support for `SSE*`, `AVX`, `AVX2`, (`AVX512` coming soon) `MMX` and more x86-64 extensions
- Support for 64-bit and 32-bit ELF as export target
- Support for multithreading to assemble your code

## getting started

Firstly you want to download/compile `rasm`'s binary.

Then you should read documentation (`docs`) and see examples (some can be currently found in `tests` directory).

## features

Precompiled `rasm` binary on default ships with following features: `mthread`, `timed`, `iinfo` and `target_all`.

You can also customize few parameters in `src/conf.rs`.

Here is exhaustive list of features you can use:

- `mthread`: multi-threading
- `timed`: measures time it took for assembling (as a whole)
- `vtimed`: for benchmarking
- `iinfo` : instruction info (also stores Mnemonics as strings)
- `target_all`: every `target_*`
- `target_elf`: elf target handling

## dev roadmap

see [roadmap.md](roadmap.md)

## credits

`rasm` was brought to you by matissoss \<matissossgamedev@proton.me> under MPL 2.0 license.
