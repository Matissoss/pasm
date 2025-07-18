<div align=center>
    <h1>perf</h1>
</div>

## info

- CPU: Intel Core i7-3770k
- RAM: 16GB
- ENV: Linux x86-64
- `pasm` compilation flags: `cargo build --release`

## benchmark 0

Source code for GAS:

```
.intel_syntax noprefix

_start:
    mov ax, bx
    ; [...] repeat until you get 100MB file
```

Source code for PASM:

```
format "elf64"
output "a.out"

section ".text" executable alloc

_start:
    mov ax, bx
    ; [...] same as before
```

Tests were done using `perf stat -e cycles,instructions,branches,branch-misses`. 

Following results were chosen as best of 5 iterations of each program.

### PASM

```
10 197 355 884      cycles:u
23 715 928 618      instructions:u                   #    2,33  insn per cycle
 5 102 555 528      branches:u
    29 623 422      branch-misses:u                  #    0,58% of all branches

2,917029825 seconds time elapsed

2,701643000 seconds user
0,204250000 seconds sys
```

### GAS

```
46 480 001 208      cycles:u                                                                (66,68%)
88 661 632 993      instructions:u                   #    1,91  insn per cycle              (83,34%)
18 184 929 314      branches:u                                                              (83,33%)
   116 183 976      branch-misses:u                  #    0,64% of all branches             (83,33%)

12,557571220 seconds time elapsed

12,292801000 seconds user
0,134142000 seconds sys
```

## benchmark 1

> [!NOTE]
> Following benchmark only has PASM variant

Following code was used:

```
format "bin"
bits 64

_start:
    jmp @_start
    ; [...] repeat until file reaches 100MB
```

Benchmark:

```
 9 340 119 980      cycles:u                         
20 354 620 308      instructions:u                   #    2,18  insn per cycle
 4 322 038 224      branches:u                       
    22 308 887      branch-misses:u                  #    0,52% of all branches

3,058543794 seconds time elapsed

2,502184000 seconds user
0,535834000 seconds sys
```

## benchmark 2

> [!NOTE]
> Following benchmark only has PASM variant

Following code was used:

```
format "bin"
bits 64

_start:
    vaddps xmm2, xmm3, xmm4
    ; [...] repeat until file reaches 100MB
```

Benchmark result:

```
 7 080 275 660      cycles:u
15 401 647 909      instructions:u                   #    2,18  insn per cycle
 3 195 457 138      branches:u
    35 821 104      branch-misses:u                  #    1,12% of all branches

2,007428115 seconds time elapsed

1,879298000 seconds user
0,117254000 seconds sys
```


## benchmark 3

> [!NOTE]
> Following benchmark only has PASM variant

Following code was used:

```
format "bin"
bits 64

_start:
    vaddps zmm21 {k1} {z}, zmm31, zword (rax + rcx * 4 + 10)
    ; [...] repeat until file reaches 100MB
```

Benchmark results:

```
 5 741 855 138      cycles:u
11 857 702 335      instructions:u                   #    2,07  insn per cycle
 2 546 956 139      branches:u
    37 318 864      branch-misses:u                  #    1,47% of all branches

1,630835000 seconds time elapsed

1,530515000 seconds user
0,093391000 seconds sys
```
