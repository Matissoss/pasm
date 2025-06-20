<div align=center>
    <h1>syntax-std.md</h1>
</div>

## quick introduction

`rasm`'s syntax is a custom mix of Intel and AT&T x86-64 assembly syntax with a bit of custom "improvements".

`rasm` follows following operand order (our naming):

```
dst. (destination) -> src. (source) -> ssrc. (or src2; second source)
```

You can make comments in `rasm` using `;`.

## operands

### immediate

Immediates are prefixed with `$`. They can be saved in: hex, binary, decimal, float or double number formats.

```
$10
$0xFF
$0b10
$1.00
$3.14
```

### register

Registers are prefixed with `%` and their naming is same as in every other x86-64 assembler.

```
%eax
%fs
%dr0
%xmm0
%ymm1
%cr1
```

### memory addressing

Memory addressing is almost same as in Intel syntax, but with difference that we use `()` instead of `[]`. 

Operands used inside memory must be also prefixed. Note that you have to use size directive after memory addressing.

```
(%rax) .qword
(%rax + %rcx) .xword
(%rax + %rcx * $4) .dword
(%rax + $10) .byte
```

You can also refer to segments, but then you will need to use modifier `%segment:memory_address`. It may sound complicated, but it is not.

```
%fs:(%rax + %rcx)
%cs:(%rax)
```

### symbol references

Symbol referenced are prefixed with `@`.

```
@_entry
@_start
```

If you want to use "extended" version of them (specify addend, relocation type (rel/abs)) then you want to use `@(symbol:reltype:addend)` closure.

Immediate there can be prefixed or not.

```
@(symbol:rel:10)
@(symbol:-10)
@(symbol:rel)
@(symbol)
```

## labels

Label is basically storage for instructions with name and attributes like: `align` (only `bin` target!), `bits` (like `bits` directive in `NASM`), `visibility` (either `local` (default) or `global`).

```
_label:
```

### label attributes

Label attributes are inside `#()` closure and split using `,`. They also can be chained.

```
#(bits=64,visibility=global)
#(align=16)
_label:
```

> [!NOTE]
> If `bits` attributes is not set, then one used in section is used.

## sections

You can declare section using `.section` directive followed by section's name (`.section section_name`).

You can specify section's: `bits` parameter (used for OSOP/ASOP and checks), `align` (ELF's `sh_addralign` or smth like that), `write`/`alloc`/`exec` permissions (ELF's `SHF_WRITE`, `SHF_ALLOC` and `SHF_EXECINSTR`).

```
.section mysection
    .bits $64
    .align $16
    .write ; sets write flag (W)
    .alloc ; sets alloc flag (A)
    .exec  ; sets exec flag  (X)
```

Sections basically store labels.

> [!NOTE]
> Be aware that due to `.` being prefix for directives, for sections like `.text` you will have to use `".text"`

## data

To preserve simplicity data is stored in labels.

For this purpose few "instructions" were added. `be` suffix means big endian and `le` (and no) suffix means little endian.

Here is list of them:

Following "instructions" need immediate at destination operand.

- `byte`/`bytele`/`bytebe` : 8-bit value
- `word`/`wordle`/`wordbe` : 16-bit value
- `dword`/`dwordle`/`dwordbe`: 32-bit value
- `qword`/`qwordle`/`qwordbe`: 64-bit value
- `empty`: makes zero'ed buffer

Following "instructions" need string at destination operand.
- `string`/`ascii`: string without null terminator

```
_data:
    string "Hello, World!"
    byte $0
    wordbe $10
    qword $10
    dwordle $10
    empty $16
```

## symbols

Symbol is basically every label/section you declare.

## closures

### consteval

Constant time evaluations are basically immediates. They can be used with `$()` closures.

Note that referencing other symbols inside `$()` is forbidden.

```
$(10 ^ 20)  ; 10 XOR 20
$(10 & 20)  ; 10 AND 20
$(10 | 20)  ; 10 OR  20
$(~10)      ; NEG 10
$(!10)      ; NOT 10
$(10 << 20) ; LSH 10 20
$(10 >> 20) ; RSH 20 10
```

You can use `.math` directive to declare mathematical symbol and reference with `@` (it is inlined).

```
.math PI $3.14

; [...]
_start:
    mov %rax, @PI
```

> [!NOTE]
> `rasm`'s constevals don't use classical mathematic expression solver (like RPN), but custom one.

### attributes

See `label` section

### symbols references

See `symbols references` section

## directives

Here is a list of availiable directives (also known as keywords) with their explaination:

- `.extern <SYMBOL_NAME>`: forbidden in `bin` target
- `.include <FILE_NAME>`
- `.section <SECTION_NAME>`
- `.align <UINT16_T>`
- `.bits <UINT8_T>`
- `.alloc` - used in ELF targets
- `.write` - used in ELF targets
- `.exec` - used in ELF targets
- `.math <MATHSYMBOL> $<IMMEDIATE>`

Here is list of size directives:

- `.byte` : 8-bit
- `.word` : 16-bit
- `.dword`: 32-bit
- `.qword`: 64-bit
- `.xword`: 128-bit
- `.yword`: 256-bit
