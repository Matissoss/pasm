<div align=center>
    <h1>pasm</h1>
</div>

## about

> [!WARNING]
> This is development branch of PASM assembler.

`pasm` is modern and independent assembler for x86-64 architecture as one of key parts in [polon](https://github.com/Matissoss/polon) toolchain.

Current focuses/goals are:
- Rewriting entire front-end to be streaming model only for better memory optimizations, may reduce it up to only `Source Code Size + Output Size` with minor offset (unless you're planning to put 1GB long string, then maybe more than that ;D)
- Rewrite most of modules in `shr`
- Remove most of "high-level" features (defines, const time evaluations) in favor of future polon preprocessor
- Adapt existing back-end to new front-end
- Fully migrate to new `CheckAPI` (or some derivative)
- Write tests and document every module
- Make `pasm`'s syntax compatible with most of Intel Syntax (so I don't have to write two different tests).
- Write tests for rest of x86-64 instructions
- Adapt new version naming system

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
