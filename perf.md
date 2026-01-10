<div align=center>
    <h1>perf</h1>
</div>

## info

- CPU: Intel Core i7-3770k
- RAM: 16GB
- ENV: Linux x86-64
- when: 10.01.2026
- `pasm` compilation flags: `cargo build --release` (ver 1.0.0)
- (GAS) `as` version: `2.45.1-1.fc43`
- `perf` version: `perf version 6.17.12-300.fc43.x86_64`
- `perf` arguments: `perf stat -e cycles,instructions,branches,branch-misses`

## benchmark 0

### Source Code for PASM

```
target elf64
bits 64
_start:
    mov rax, 1
    add rax, 2
    sub rax, 5
    push 0
    xor qword [rsp], rcx
    pop rax
    addps xmm0, xmm1
    vcvttpd2dq xmm7, xmm5
    addsubps xmm2, xword [rax]
    vfmaddsub132ps ymm9, ymm12, xmm0
    xor rdi, 10
    and rdi, 1
    shl rax, 14
    shr rax, 12
    syscall
    ; repeat these 14 lines until source code file gets 100MB
```

### Source Code for GAS
```
.intel_syntax noprefix
bits 64
_start:
    mov rax, 1
    add rax, 2
    sub rax, 5
    push 0
    xor qword [rsp], rcx
    pop rax
    addps xmm0, xmm1
    vcvttpd2dq xmm7, xmm5
    vaddsubps xmm2, oword [rax]
    vfmaddsub132ps ymm9, ymm12, xmm0
    xor rdi, 10
    and rdi, 1
    shl rax, 14
    shr rax, 12
    syscall
    ; repeat these 14 lines until source code file gets 100MB
```

### Results

#### PASM

```
     9 580 055 631      cycles:u
    14 773 197 936      instructions:u                   #    1,54  insn per cycle
     3 326 376 659      branches:u
        61 624 031      branch-misses:u                  #    1,85% of all branches

       2,648856174 seconds time elapsed

       2,552998000 seconds user
       0,072601000 seconds sys
```

#### GAS

```
    37 121 627 688      cycles:u
    52 318 531 365      instructions:u                   #    1,41  insn per cycle
    11 085 754 686      branches:u
       135 672 346      branch-misses:u                  #    1,22% of all branches

      10,238676801 seconds time elapsed

       9,914215000 seconds user
       0,255321000 seconds sys
```
