<div align=center>
    <h1>pasm</h1>
</div>

## about

`pasm` is modern and independent assembler for x86-64 architecture as one of key parts in (future project) `POLON` toolchain.

> [!NOTE]
> `pasm` is still in beta phase and should not be relied on for any serious work.

## features

> [!NOTE]
> This info is for `betao` release.

- Support for ~50%+ x86-64 ISA (~848 instructions)
- Support for `SSE*`, `AVX`, `AVX2`, (`AVX512` coming soon) `MMX` and more x86-64 extensions
- Support for 64-bit and 32-bit ELF as export target
- Support for multithreading to assemble your code

## getting started

Firstly you want to download/compile `pasm`'s binary.

Then you should read documentation (`docs`) and see examples (some can be currently found in `tests` directory).

## examples

go to [examples](examples)

## features

Precompiled `pasm` binary on default ships with following features: `timed`, `iinfo` and `target_all`.

You can also customize few parameters in `src/conf.rs`.

Here is exhaustive list of features you can use:

- `time`: measures time it took for assembling (as a whole)
- `vtime`: for benchmarking
- `iinfo` : instruction info (also stores Mnemonics as strings)
- `target_all`: every `target_*`
- `target_elf`: elf target handling

## dev roadmap

see [roadmap.md](roadmap.md)

## credits

`pasm` was brought to you by matissoss \<matissossgamedev@proton.me> under MPL 2.0 license.
