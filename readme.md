<div align=center>
    <h1>pasm-x86</h1>
</div>

## about

pasm-x86 is an assembler for x86-64 architecture.

## features

- Wide coverage of x86-64 ISA (1795 unique mnemonics)
- Support for `SSE*`, `AVX`, `AVX2`, `AVX-512`/`AVX-10`, `Intel APX`, `x87`, `MMX` and more x86-64 extensions
- Support for bin and 32/64-bit ELF as export target
- Very performant and optimized (~3-4x faster than GAS in [our cherry picked benchmarks](perf.md) 😊)

## getting started

Just run: `cargo build --release` or `cargo install --path .`

> [!NOTE]
> If you want to test if source code from any commit is valid to use:
> ```sh
> # this requires that NASM binary is installed in $PATH
> $ just test 
> ```

## documentation

pasm's documentation can be found [here](docs/readme.md).

It contains information on pasm's syntax as well as full x86-64 encoding notes.

## credits

`pasm` was brought to you by matissoss \<matissossgamedev@proton.me> under MPL 2.0 license.
