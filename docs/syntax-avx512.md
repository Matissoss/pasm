<div align=center>
    <h1>syntax-avx512.md</h1>
</div>

## getting started

You can use AVX-512 in my assembler like in other assemblers.

Example of `vaddph`:

```
vaddph %xmm20 {k2}, %xmm21, %xmm22
```

## mbcst

To use broadcast on memory use `.size:bcst` modifier:

```
.word:bcst (%rax + %rcx)
```

## disclosure

I'm very lazy programmer, so instructions encoded with EVEX derived from AVX(1/2) don't check if they can use EVEX. So if anything is wrong (if CPU throws #UD, etc.), 
check AVX instructions used in code first and then if something is still wrong (and you get wrong output regardless) then report it as bug.
