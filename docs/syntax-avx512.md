<div align=center>
    <h1>syntax-avx512.md</h1>
</div>

## before we start

When I mention EVEX then I mean AVX-512.

## mnemonics

Mnemonics that use EVEX are prefixed with letter `E` (not `V` like other AVX mnemonics).

The reason for this is "logical": if we assert that `V` prefix means `VEX`, then `E` means `EVEX`. 

Another reason is that it is explicit: you exactly know, if our assembler will encode EVEX or VEX.

## using masks

Masks have same naming as in every other assembler: `k0-7`.

To use mask you will need to use modifier on mnemonic:

```
emnemonic:k0 ; [...]
```

## using {sae}, `{er}` and {z}

To use `{sae}` (suppress all exceptions), `{er}` and `{z}` you will have to use another modifier on mnemonic:

```
emnemonic:sae/er:z ; [...]
```

To use them with masks we have modifer of 4 elements: `mnemonic:mask:z:sae/er`.

## using mbcst

Use modifier: `.size:bcst`

```
.qword:bcst (%rax + %rcx)
```

## example instruction

We will use `vaddph` instruction as example:

```
eaddph:k2:z %xmm2, .word:bcst (%rax)
```
