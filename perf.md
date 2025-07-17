<div align=center>
    <h1>perf</h1>
</div>

## test no. 0 - 100MB source file

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

Tests were done using `perf stat -e cycles,instructions,branches,branch-misses`. `pasm` version `beta-avx` (`release` profile) and `as` version `2.41-38.fc40`.

Following results are the best of 5 iterations.

### PASM

```
10 668 754 630      cycles:u                                                                (83,34%)
24 637 897 305      instructions:u                   #    2,31  insn per cycle              (83,34%)
 5 252 531 119      branches:u                                                              (83,34%)
    29 857 183      branch-misses:u                  #    0,57% of all branches             (83,33%)

3,037352654 seconds time elapsed

2,830512000 seconds user
0,175963000 seconds sys
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
