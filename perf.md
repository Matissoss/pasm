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

Tests were done using `perf`. Here are arguments for (GAS) `as` (version: `2.41-38.fc40`): `as <SOURCE_FILE>` and for `pasm` (compiled using `release` profile without anything else; for version `25.06-beta2` after patch from 08.07.2025): `pasm -i=<SOURCE_FILE>`.

Following results are the best one of 5 iterations.

### PASM

```
13 340 703 854      cycles:u                                                   (66,71%)
31 133 614 154      instructions:u      #    2,33  insn per cycle              (83,37%)
 6 178 595 883      branches:u                                                 (83,32%)
    29 335 888      branch-misses:u     #    0,47% of all branches             (83,32%)

    4,005931614 seconds time elapsed

    3,545263000 seconds user
    0,399712000 seconds sys
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
