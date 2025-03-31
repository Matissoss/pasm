<h1 align=center>rasm syntax</h1>

## Status

Working on implementing

## TLDR

- Instruction format: `instr dest, source`
- Register prefix: `%`
- Comment Start: `;`
- Label prefix: `&`
- Immediate prefix: `$`; supports binary, decimal and hex format.
- Memory brackets: `(` and `)`
- Size specificators (aka `!scale`): `!qword` (8 byte), `!dword` (4 bytes), `!word` (2 bytes), `!byte` (1 byte)
- SIB: `disp(%base,%index) !scale`
- Offset: `offset(%base) !scale`
- Base: `(%base) !size`
- Section: `.section_name`
- .bss  variable: `name !scale $size`
- .data variable: `name !scale content`
- .text global labels: `global label`
- Labels and Sections end with `!end`

## Registers and Immediates

Registers are prefixed with `%` and immediates prefixed with `$`.

Immediates can be saved in hexadecimal, decimal or binary format.

They need to start with `0x` or `-0x` for hex, `0b` or `-0b` for binary.

Example:
```
%rax    - RAX register
%esp    - ESP register
%cx     - CX  register
%dl     - DL  register

$0xFF        - 255 in hexadecimal format
$0x11111111  - 255 in binary format
$255         - 255 in decimal format
```

## Memory Addresation

Memory address should be found in beetwen '(' and ')' (AT&T-like).

### Register-base

General rule is following `(base_register) !scale`, where `!scale` is either: 
`!byte` (1 byte), `!word` (2 bytes), `!dword` (4 bytes) or `!qword`(8 bytes)

Example:
```
(%rsp) !dword   -   pointing at dword (4 bytes) in address `%rsp`
```

### With Offset

General rule is same as in Register-base, but we add offset at beginning: `offset(base_register) !scale`

Example:
```
-0x4(%rsp) !dword   -   pointing at dword (4 bytes) in address `%rsp - 4`
0x4(%rsp)  !dword   -   pointing at dword (4 bytes) in address `%rsp + 4`
```

### SIB

General rule is the same as in two previous examples, but we add `index` register. Now it only depends if we have offset (referred to as `displacement`) or not:

- With:     `(%base_register, %index) !scale`
- Without:  `displacement(%base_register, %index) !scale`

## Keywords

Keywords are prefixed with `!` char.

Keyword list:
- `!qword` - scale = 8
- `!dword` - scale = 4
- `!word`  - scale = 2
- `!byte`  - scale = 1
- `!end`   - ends label/section declaration

## Labels

Labels define sequence of instructions and are started with `name:` and ended with `!end`.

Labels can be referred in code to with `&` prefix.

```
_start:
    jmp &label
!end

label:
    mov %rax, $60
    mov %rdi, $1
    syscall
!end
```

Naming convention is following for labels:
- Normal Labels: `label`
- Helper Labels: `_parent-function_label`

Example:
```
strlen: ; normal label
    ; code here
    jmp _strlen_loop
!end
_strlen_loop: ; helper label
    ; code goes here
!end
```

## Sections

Sections are declared with `.` prefix without need for `section` keyword and ended with `!end` keyword.

There are few unique/reserved `sections` that being: `.rodata`, `.text`, `.data`, `.bss`.

Example:
```
.bss
    buffer !byte $12    ;   bss variable named `buffer` with len = 12
!end
.data
    msg !byte "Hello, World!", $13, $10, $0  ; data variable named `msg` with content: "Hello, World!\r\n\0"
    msg1 !byte "Hello, World!", 13, 10, 0    ; NOTE that without `$` prefix values will be treated as strings and it will return content: "Hello, World!13100"
!end
.text
    global _start       ;   global label named `_start`
!end
```

To reference value declared in `.bss` and `.data` sections we can use `@` prefix.

Example:
```
mov %rsi, @msg
```

## Source Code Example

### Strlen

```
.text
    global _start
!end

.bss
    buffer !byte $20
!end

_start:
    ; read(0)
    mov %rax, $0
    mov %rdi, $0
    mov %rsi, @buffer
    mov %rdx, $20
    syscall

    mov %rdi, @buffer
    call &strlen

    ; write(1)
    mov %rdx, %rax
    mov %rax, $1
    mov %rdi, $1
    mov %rsi, @buffer
    syscall

    ; exit(0)
    mov %rax, $60
    mov %rdi, $0
    syscall
!end

strlen:
    mov %rcx, $0
    jmp _strlen_loop
!end
_strlen_loop:
    mov %al, (%rdi) !byte
    cmp %al, $0
    je  &_strlen_end
    inc %rcx
    inc %rdi
    jmp &_strlen_loop
!end
_strlen_end:
    mov %rax, %rcx
    ret
!end
```

## Inspirations

rasm's syntax is inspired by AT&T-like and Intel-like syntax.
