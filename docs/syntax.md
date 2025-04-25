<h1 align=center>rasm syntax</h1>

## before we get started...

This tutorial asserts that you have basic knowledge of x86(-64) assembly.

Don't have x86(-64) assembly knowledge? Too bad.

## instructions

Instructions follow Intel-like syntax:
```
mnemonic destination, source
```

## immediates

immediates can be saved in `float` (32-bit), `double` (64-bit), `binary`, `decimal` or `hex` formats prefixed with `$` (this applies to all numbers).

### floats

Floats are saved as bytes and treated like byte casted 32/64-bit (depending if it's `float` or `double`) unsigned integer, 
if used in context that is not float related (like `xmm` registers).

### hex and binary

```
$0xFF ; 255 in hexadecimal
$0b10 ; 2 in binary
$-0b10 ; -2 in binary
$-0xFF ; -255 in hexadecimal
```

## registers

Registers are prefixed with `%` using known convention (rax, rbx, rcx).

## memory

Memory is a bit more complex. It follows this syntax:
```
(memory) !size
```

where `!size` can be one of these: `!byte` (for 8-bit value), `!word` (for 16-bit value), `!dword` (for 32-bit value) or `!qword` (for 64-bit value)

### base

```
(%register +- $displacement)
```

### index * scale

```
(%index * $scale +- displacement)
```

> [!WARNING]
> `*` must be used, otherwise you will get error

### base + index * scale

```
(%base, %index * $scale +- $displacement)
```

## labels

Labels are created like this `name:`

## variables

Depending on type of variable you want to declare, declaring variable is very simmilar for all cases.

### .data/.rodata - initialized constant/readonly

```
!const/!ronly var_name !size content
```

where `!size` is either: `!byte`, `!word`, `!dword`, `!qword` (***or in future***: `!xword`, `!yword`).

`content` can be either: a number (prefix `$`) or a string (***IF YOU WANT YOUR VARIABLE TO BE STRING, PREFIX IT'S NAME WITH*** `str_`***!***)

### .bss - uninitialized buffers

```
!uninit var_name !size/$size
```

`!size/$size` means that size specifier can either be a classical size specifier or immediate size specifier (number like `$10`).

## globals

Globals can be declared using `!global` keyword, followed by symbol name (not prefixed with anything).

Example:
```
; makes symbol `_start` global
!global _start
!global var

; works on variables
!const var !byte $10

; this symbol is global now
_start: 
    ; [...]
```

## externs

This feature is planned, but currently isn't implemented in RASM assembler.

Basically tells assembler that symbol used in this file cannot be found in this file.

## entry

File entry (ussually `_start` in UNIX-like systems) ***MUST BE GLOBAL***. 

Option to change default entry will be added soon in RASM Assembler
