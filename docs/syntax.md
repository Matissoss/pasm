<h1 align=center>rasm syntax</h1>

>[!WARN]
> This section will be updated frequently.

## Registers and Immediates

Registers are prefixed with `%` and immediates prefixed with `$`.

Immediates can be saved in hexadecimal, decimal or binary format.

They need to start with `0x` or `-0x` for hex and `0b` or `-0b` for binary.

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

General rule is same as in Register-base, but we add offset at beginning: `(base_register+-offset) !scale`

Example:
```
(%rsp-4) !dword   -   pointing at dword (4 bytes) in address `%rsp - 4`
(%rsp+4) !dword   -   pointing at dword (4 bytes) in address `%rsp + 4`
```

### SIB

General rule is the same as in two previous examples, but we add `index` register. Now it only depends if we have displacement or not:

- With:     `(%base_register, %index * $scale) !size`
- Without:  `(%base_register, %index * $scale +- $displacement) !size`

### SIB without Base

Same as in SIB, but you remove base register.

> [!NOTE]
> if this method is used, then index must be followed by scale with asterisk prefix. Otherwise assembler will treat
> scale as displacement.

## Keywords

Keywords are prefixed with `!` char.

Keyword list:
- `!qword` - scale = 8
- `!dword` - scale = 4
- `!word`  - scale = 2
- `!byte`  - scale = 1
- `!bss <NAME> <SIZE>`      - declares variable in `.bss` section
- `!data <NAME> <CONTENT>`  - declares variable in `.data` section
- `!bits <BITS>`            - specifies to which mode assembler should stick (def: 64)
- `!global <NAME>`          - makes symbol `<NAME>` global like in other assemblers

## Labels

Labels define sequence of instructions and are started with `<NAME>:`.

Labels can be referred in code to with `&` prefix.

```
_start:
    jmp &label

label:
    mov %rax, $60
    mov %rdi, $1
    syscall
```

Naming convention is following for labels:
- Normal Labels: `label`
- Helper Labels: `_parent-function_label`

Example:
```
strlen: ; normal label
    ; code here
    jmp _strlen_loop
_strlen_loop: ; helper label
    ; code goes here
```

## Sections

You cannot directly modify sections, but you can use it with `!bss`, `!data`, `!rodata` keywords at start of statement.
Symbols declared that way are referenced using `@` prefix.

```
!bss buffer $20 ; 20B long buffer in bss section
!data msg "Hello, World!", $13, $10, $0 ; can be used for strings. Note that you need to prefix numbers with '$', because
                                        ; otherwise number will be treated as string that should be concatted.
!data mynum $10 ; ... or for numbers

label:
    ; [...]
    lea %rax, @msg
    ; error will happen here
    mov %rax, @msg
    ; [...]
```

## Source Code Example

### Strlen

```
!bss buffer $20
!global _start
!bits 64

_start:
    ; read(0)
    mov %rax, $0
    mov %rdi, $0
    lea %rsi, @buffer
    mov %rdx, $20
    syscall

    lea %rdi, @buffer
    call &strlen

    ; write(1)
    mov %rdx, %rax
    mov %rax, $1
    mov %rdi, $1
    lea %rsi, @buffer
    syscall

    ; exit(0)
    mov %rax, $60
    mov %rdi, $0
    syscall

strlen:
    mov %rcx, $0
    jmp &_strlen_loop
_strlen_loop:
    mov %al, (%rdi) !byte
    cmp %al, $0
    je  &_strlen_end
    inc %rcx
    inc %rdi
    jmp &_strlen_loop
_strlen_end:
    mov %rax, %rcx
    ret
```
