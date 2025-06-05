<h1 align=center>rasm syntax</h1>

This is short documentation for RASM assembler syntax.

## Source File Structure

```
ROOT

; code
label:
    ; [...]
label2:
    ; [...]
```

## Instruction format

All instructions follow following order:

```
mnemonic [additional mnemonic] operand_1, operand_2, operand_3, [...]
```

Operand order is **same as in Intel-like Syntax** (destination, then source, then second source (used in `AVX`), etc.).

## Operand format

Operands can either be: a register, a memory address, a memory address within segment, symbol or immediate. 

### Registers

Registers are prefixed with `%`.

```
%eax
%rax
%rcx
%r8
%r9
%xmm0
```

### Immediate
    
Immediates are prefixed with `$`. They can be saved in: hexadecimal, binary, decimal
single precision (`float`) or double precision (`double`) formats.

```
$0xFF
$0b10
$-0xFF
$1.00
$-3.14
```

### Memory

Memory is same as in Intel-like syntax, but is started with `(` and ended with `)` (not `[`, `]`).
Values there also must be prefixed (immediates can be non-prefixed there).

A size specifier must be either after or before memory address (both variants are supported, because why not?).

RASM assembler supports: SIB, SIB with offset, Base, Index-only memory formats.

```
; SIB
!byte (%rax + %rcx * 1 + 200)
; Base
!dword (%rax)
; Index-only
(%rcx * 4) !qword
```

Memory can also relate to segments like `cs`. It must be prefixed with `#` and split using `:`

```
#cs:(%rax)
#fs:(%rax + %rcx * 4 + 20)
```

### Symbols

What is a symbol? Symbol is every label and every variable you declare. It is prefixed (when referenced) with `@` prefix.

```
@symbol_name
```

### Size Specifiers

Size specifiers are prefixed with `!` (like other keywords)

| NASM Size Specifier | RASM Size Specifier | Size (in bits) |
|:-------------------:|:-------------------:|:--------------:|
|       byte          |         byte        |        8       |
|       word          |         word        |        16      |
|      dword          |        dword        |        32      |
|      qword          |        qword        |        64      |
|      oword          |        xword        |       128      |
|      yword          |        yword        |       256      |

### Keywords

Keywords are prefixed with `!`.
Here is list of all keywords (according to `src/shr/kwd.rs`) with their arguments:
- `!bits $IMM8` : Specifies `bits` parameter (same as NASM's `bits` parameter).  Must be declared in `ROOT`
- `!qword [MEM]`
- `!byte [MEM]`
- `!word [MEM]`
- `!xword [MEM]`
- `!math [NAME] [VAL]`
- `!ronly [VAR DECLARATION]`
- `!const [VAR DECLARATION]`
- `!uninit [VAR DECLARATION]`
- `!entry [LABEL NAME]`: Specifies entry point for relocatable file. (basically is a swap; see `src/shr/ast.rs:AST::fix_entry`)
- `!global [SYMBOL NAME]`: Specifies if symbol `[SYMBOL NAME]` is global or not. Must be declared in `ROOT`.
- `!extern [SYMBOL NAME]`: Specifies that symbol `[SYMBOL NAME]` is in this file or not. Must be declared in `ROOT`.

### Mathematics

Mathematical closures (`$()`) can be used to evaluate complex mathematical evaluations.

To use them you can either:
- Use as immediate (like `mov %rax, $(...)`)
- Or defined as constant in `ROOT` (like `!math name $(...)` - must be referenced with `@`)

### Variables

> [!NOTE]
> Variables must be declared in `ROOT`.

#### Constant (.data) & Readonly (.rodata)

Constant/Readonly variable must have following things: name, size specifier (keyword size specifier) 
and content (either string or number).

```
!const name !byte "Hello, World!", $13, $0
!ronly eman !word $10
```

> [!WARNING]
> Constant/Readonly value in `bin` format cannot be strings as it is "inline" - use manual `push`'es

#### Uninitialized (.bss)

Uninitialized variable must have a name and size specifer (can be a number).

```
!uninit name $13
!uninit eman !word
```

> [!WARNING]
> Uninitialized values cannot be used in `bin` format - use manual `push`'es
