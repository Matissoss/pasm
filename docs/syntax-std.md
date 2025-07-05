<div align=center>
    <h1>syntax-std.md</h1>
</div>

## quick introduction

`pasm`'s syntax is Intel-like and is very familliar to FASM's syntax.

Format used to explain syntax is [psyn v1](https://github.com/Matissoss/psyn)

## terminology

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

#### extern

You can make extern symbol (they are ignored in `bin` target) using `extern` directive:

```
extern <NAME>
```

#### include

You can include other `pasm` source files using `include` directive:

```
include <PATH>
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
