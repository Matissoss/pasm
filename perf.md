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

Tests were done using `perf`. Here are arguments for (GAS) `as` (version: `2.41-38.fc40`): `as <SOURCE_FILE>` and for `pasm` (compiled using `release` profile without anything else; for version `25.06-beta2` after patch from 07.07.2025): `pasm -i=<SOURCE_FILE>`.

Following results are the best one of 5 iterations.

### PASM

```
14 060 510 325      cycles:u                                                                (66,65%)
31 252 702 733      instructions:u                   #    2,22  insn per cycle              (83,32%)
 6 412 929 007      branches:u                                                              (83,34%)
    25 681 455      branch-misses:u                  #    0,40% of all branches             (83,33%)

4,569072173 seconds time elapsed

3,788768000 seconds user
0,728574000 seconds sys
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
