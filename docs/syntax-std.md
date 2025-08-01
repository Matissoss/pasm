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

### $() closure

You can utilize the `$()` closure for compilation-type evaluations. It is then later inlined into immediate. 

> [!NOTE]
> You cannot reference other symbols inside this closure.

It supports following operations:

| Operator | Operation |
|:--------:|:---------:|
| + (add)  | lhs + rhs |
| - (sub)  | lhs - rhs |
| * (mul)  | lhs * rhs |
| / (div)  | lhs / rhs |
| % (mod)  | lhs % rhs |
| ! (not)  | !lhs      |
| ~ (neg)  | -lhs      |
| ^ (xor)  | lhs ^ rhs |
| & (and)  | lhs & rhs |
| | (or)   | lhs | rhs |
| << (lsh) | lhs << rhs|
| >> (rsh) | lhs >> rhs|

You can isolate different parts of evaluation by using `()`.

Example of `$()` closure:

```
mov rax, $((2 << 4) >> (1 << 4))
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

## advanced syntax

Now that we covered basics, we can go to more complex things like sections, labels and ROOT directives.

### labels

Labels are defined using `<LABEL_NAME>:`.

```
label:
_start:
main:
```

#### attributes

##### inline

Following attributes can be used as inline attributes:

- visibility (only one at once):
    - `public`
    - `weak`
    - `protected`
    - `anonymous`
    - `local`
- type (only one at once):
    - object
    - function

Here is the syntax formula:

```
[VISIBILITY] [TYPE] <LABEL_NAME>:
```

Example:

```
public function main:
public object hello_world:
_start:
```

##### external

External attributes are basically Closure `#()`. Syntax formula:

```
#(<ATTRIBUTE>[=VALUE],[...])
```

| Attribute | Accepted values |
|:---------:|:---------------:|
| bits      | 16, 32 or 64    |
| align     | uint16          |
| public    |        -        |
| protected | -               |
| local     | -               |
| weak      | -               |

External attributes can be chained across multiple lines.

---

Both of these attribute types can be used at once:

```
#(bits=64)
public function _start:
```

### sections

Sections can be defined using `section "<SECTION_NAME>" [ATTR] [...]`

List of `[ATTR]`:

- `executable` - sets executable flag (`X` in ELF)
- `alloc` - sets alloc flag (`A` in ELF)
- `writeable` - sets write flag (`W` in ELF)
- `nobits` - treats section like `.bss` in ELF

```
section ".text" alloc executable
```

You can also specify section's align (`sh_addralign` in ELF):

```
section ".text" alloc executable
    align 16
```

### ROOT

In ROOT you can define most of `pasm`'s default settings.

Here are directives you can use inside ROOT:

#### define

You can define constant inline values (they are not translated into ELF symbols):

```
define <NAME> <VALUE>
```

Symbol defined with `define` can be referenced using `@` prefix. `<VALUE>` can only be 64-bit immediate that cannot reference other symbols.

There are also builtins `defines` that we discuss in Appendix A

#### extern

You can make extern symbol (they are ignored in `bin` target) using `extern` directive:

```
extern <NAME>
```

#### output

You can specify default output path using `output` directive:

```
output <PATH>
```

#### format

You can specify output's format using `format` directive:

```
format elf64/elf32/bin
```

> [!NOTE]
> `elf*` targets are only Little-Endian variants

## data instructions

- `bytele <IMM>`/`bytebe <IMM>`: 8-bit value
- `wordle <IMM>`: 16-bit LE value
- `wordbe <IMM>`: 16-bit BE value
- `dwordle <IMM>`: 32-bit LE value
- `dwordbe <IMM>`: 32-bit BE value
- `qwordle <IMM>`: 64-bit LE value
- `qwordbe <IMM>`: 64-bit BE value
- `empty <IMM>`: fills buffer with 0 `<IMM>` times
- `string <STRING>`: string value

## instructions with changed name

- `MOVSD` (for strings): `MOVSTRD`
- `MOVSB`: `MOVSTRB`
- `MOVSW`: `MOVSTRW`
- `MOVSQ`: `MOVSTRQ`
- `CMPSB`: `CMPSTRB`
- `CMPSW`: `CMPSTRW`
- `CMPSD`: `CMPSTRD`
- `CMPSQ`: `CMPSTRQ`

## appendix

### appendix a

Here is table of builtins defines:

|   Name            | Value                         |
|:-----------------|:-----------------------------|
|`__TRUE`           | 1                             |
|`__FALSE`          | 0                             |
|`__DOUBLE_MIN`     | min. value of double          |
|`__DOUBLE_MAX`     | max. value of double          |
|`__DOUBLE_INF`     | infinity as double            |
|`__DOUBLE_NEG_INF` | negative infinity as double   |
|`__DOUBLE_EXP_MIN` | minimal exponent of double    |
|`__DOUBLE_EXP_MAX` | maximal exponent of double    |
|`__DOUBLE_PI`      | PI                            |
|`__DOUBLE_SQRT2`   | `√2`                          |
|`__DOUBLE_LN2`     | log(2)                        |
|`__DOUBLE_LN10`    | log(10)                       |
|`__FLOAT_MIN`      | min. value of float           |
|`__FLOAT_MAX`      | max. value of float           |
|`__FLOAT_INF`      | infinity as float             |
|`__FLOAT_NEG_INF`  | negative infinity as float    |
|`__FLOAT_EXP_MIN`  | minimal exponent of float     |
|`__FLOAT_EXP_MAX`  | maximal exponent of float     |
|`__FLOAT_PI`       | PI                            |
|`__FLOAT_SQRT2`    | `√2`                          |
|`__FLOAT_LN2`      | log(2)                        |
|`__FLOAT_LN10`     | log(10)                       |
|`__COND_O`         | Can be used in some instructions; Condition used in `jo`        |
|`__COND_NO`        | Can be used in some instructions; Condition used in `jno`       |
|`__COND_O`         | Can be used in some instructions; Condition used in `jo`        |
|`__COND_B`         | Can be used in some instructions; Condition used in `jb`        |
|`__COND_C`         | Can be used in some instructions; Condition used in `jc`        |
|`__COND_NAE`       | Can be used in some instructions; Condition used in `jnae`      |
|`__COND_NB`        | Can be used in some instructions; Condition used in `jnb`       |
|`__COND_NC`        | Can be used in some instructions; Condition used in `jnc`       |
|`__COND_AE`        | Can be used in some instructions; Condition used in `jae`       |
|`__COND_E`         | Can be used in some instructions; Condition used in `je`        |
|`__COND_Z`         | Can be used in some instructions; Condition used in `jz`        |
|`__COND_NE`        | Can be used in some instructions; Condition used in `jne`       |
|`__COND_NZ`        | Can be used in some instructions; Condition used in `jnz`       |
|`__COND_BE`        | Can be used in some instructions; Condition used in `jbe`       |
|`__COND_NA`        | Can be used in some instructions; Condition used in `jna`       |
|`__COND_NBE`       | Can be used in some instructions; Condition used in `jnbe`      |
|`__COND_A`         | Can be used in some instructions; Condition used in `ja`        |
|`__COND_S`         | Can be used in some instructions; Condition used in `js`        |
|`__COND_NS`        | Can be used in some instructions; Condition used in `jns`       |
|`__COND_P`         | Can be used in some instructions; Condition used in `jp`        |
|`__COND_PE`        | Can be used in some instructions; Condition used in `jpe`       |
|`__COND_NP`        | Can be used in some instructions; Condition used in `jnp`       |
|`__COND_PO`        | Can be used in some instructions; Condition used in `jpo`       |
|`__COND_L`         | Can be used in some instructions; Condition used in `jl`        |
|`__COND_NGE`       | Can be used in some instructions; Condition used in `jnge`      |
|`__COND_NL`        | Can be used in some instructions; Condition used in `jnl`       |
|`__COND_GE`        | Can be used in some instructions; Condition used in `jge`       |
|`__COND_LE`        | Can be used in some instructions; Condition used in `jle`       |
|`__COND_NG`        | Can be used in some instructions; Condition used in `jng`       |
|`__COND_G`         | Can be used in some instructions; Condition used in `jg`        |
|`__COND_NLE`       | Can be used in some instructions; Condition used in `jnle`      |
