<div align=center>
    <h1>syntax-std.md</h1>
</div>

## quick introduction

`pasm`'s syntax is Intel-like and is very familliar to FASM's syntax.

Format used to explain syntax is [psyn v1](https://github.com/Matissoss/psyn)

## syntax

### size

`*SIZE* = (byte/word/dword/qword/xword/yword/zword)` 

where:

- `byte`: 8-bit
- `word`: 16-bit
- `dword`: 32-bit
- `qword`: 64-bit
- `xword`: 128-bit
- `yword`: 256-bit
- `zword`: 512-bit

### closures

`*Closure* = [PREFIX](<VALUE>)`

### modifiers

`*Modifier* = [PREFIX]<VALUE>:[...]`

### file structure

```
ROOT

<section 0>
    <label 0>
    [...]
[...]
```

## comments

Comments in `pasm` can be either `//` or `;`:

```
// this is a comment
; this is also a comment
```

## registers

Register naming conventions are same as in every other x86 assembler and do not require prefixing. This will be later referenced as `*Register*`.

```
rax
rcx
r8
r8d
r9w
r10b
xmm10
ymm2
```

## memory addressing

Memory addressing is `<*SIZE*> <*MEMORY*>`

`*Memory* = *Closure* where: PREFIX = None` and `VALUE = [?BASE:Register] [+ <?INDEX:Register>] [* <?SCALE:(1/2/4/8)>] [+ <?OFFSET:(u32/i32)>]`

```
byte (rax + rcx * 4 + 10)
qword (rax + rcx)
xword (rax)
```

To use RIP-relative addressing use `*RIP-relative* = *Memory* where: BASE = None, INDEX = None, SCALE = None, OFFSET = (u32/i32)`

```
word (10) ; RIP + 10
```

And for symbols see `symbol referencing` header.

## immediate

For immediates you can use hexadecimal (`0x` prefix), octal (`0o` prefix), binary (`0b` prefix) and float/double number formats.

```
0xFF
128
0o76
-129.0
3.14
```

## symbol referencing

`*SYMBOLREF* = *Modifier* where: PREFIX = @, VALUE = <SYMBOL_NAME>[:<?reltype:*RELTYPE*>][:<?ADDEND:int>]`

Here is the `*RELTYPE*` table along with their size and mapping in ELF x86-64 relocations:

| `RELTYPE` | Size | ELF Mapping   |
|:---------:|:----:|:-------------:|
|   abs32   | dword|`R_X86_64_32S` |
|   rel32   | dword|`R_X86_64_PC32`|
|   rel16   | word |`R_X86_64_PC16`|
|   rel8    | byte |`R_X86_64_PC8` |
|   [NONE]  | (d/q)word|`R_X86_64PC32` or `R_X86_64PC16` depending on `bits`|

Example:

```
@symbol:rel32:-0xFF
@symbol:rel32:10
```

You can also dereference a symbol (it will be treated as RIP-relative addressing) using `<*SIZE*> <*SYMBOLREF*>`:

```
qword @symbol:rel32:-0xFF
xword @symbol:rel32:10
```

## labels

Label can be declared with `*LABEL* = <NAME>:` (like in other assemblers)
```
label:
```

