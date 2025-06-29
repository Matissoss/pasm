<div align=center>
    <h1>syntax-avx512.md</h1>
</div>

## abbreviations

`AVX-512` = `AVX-512` and/or `AVX-10`

## mnemonics

Mnemonics that are derived from AVX-512 and are encoded using `EVEX` prefix are prefixed with `E` and not `V` like other AVX mnemonics.

Here are two reasons:
- Explicitness: you exactly know if assembler will encode `EVEX` or `VEX`
- "Logical": we can assert that `V` means usage of `VEX` prefix and `E` of `EVEX` prefix.

## using masks

Masks have same naming as in Intel documentations: `k0-7`; You will have to utilize mnemonic modifier.

```
emnemonic:mask ; [...]
```

## `{sae}`, `{er}`, `{z}`

You will have to use mnemonic modifier:

```
emnemonic:z:er:k0
emnemonic:z:sae:k0
```

## using mbcst

Use modifier `.size:bcst`.


```
.qword:bcst (%rax + %rcx)
```

## example instruction

We will use `vaddph` instruction as example:

```
eaddph:k2:z %xmm2, .word:bcst (%rax)
```
