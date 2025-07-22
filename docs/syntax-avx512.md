<div align=center>
    <h1>syntax-avx512.md</h1>
</div>

## getting started

You can use AVX-512 in `pasm` like in other assemblers.

Example of `vaddph`:

```
vaddph xmm20 {k2}, xmm21, xmm22
```

## mbcst

To use broadcast on memory use `*MBCST* = *Modifier* where: PREFIX = None, VALUE = <*SIZE*>:bcst`:

`pasm` will automatically choose the "right" broadcast size (by setting `EVEX.w` - no need for `{1to8}` or `{1to16}`)

```
word:bcst (rax + rcx)
```
