<div align=center>
    <h1>pasm</h1>
</div>

## about

`pasm` is modern and independent assembler for x86-64 architecture as one of key parts in [polon](https://github.com/Matissoss/polon) toolchain.

> [!NOTE]
> `pasm` will not receive new content updates from now on as development on it is finished.
>
> Only bugfixes and minor changes will be submitted.
> - matissoss, 30.07.2025

## features

- Wide coverage of x86-64 ISA (1795 unique mnemonics)
- Support for `SSE*`, `AVX`, `AVX2`, `AVX-512`/`AVX-10`, `Intel APX`, `x87`, `MMX` and more x86-64 extensions
- Support for bin and 32/64-bit ELF as export target
- Very performant and optimized (~3,4x faster than GAS in [some cases](perf.md))

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
- `iinfo` : instruction info (stores mnemonics as strings)
- `target_all`: every `target_*`
- `target_elf`: elf target handling

## dev roadmap

see [roadmap.md](roadmap.md)

## credits

`pasm` was brought to you by matissoss \<matissossgamedev@proton.me> under MPL 2.0 license.
