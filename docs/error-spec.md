<div align=center>
    <h1>error-spec.md</h1>
</div>

### e000

#### description

This error is provoked by using unclosed delimeter.

### e001

Currently reserved.

### e002

Currently reserved.

### e003

#### description

This error is provoked, when you provided one token for operand, but it couldn't be converted into operand.

#### example

```
_start:
    dword ; e[003]. directive dword cannot be converted into operand
```

### e004

#### description

This error is provoked, when you redeclare a symbol/directive, when they can only be unique.


#### example

```
_start:
    jmp @_start1 ; e[004]
_start: ; e[004]
    ; [...]
```

### e005

#### description

> [!NOTE]
> This error is specific for `bin` target

This error is provoked, when assembler tried to write too large relocation address on too small buffer.

#### example

```
some_label:
    bytele 0
#(align=257)
_start:
    jmp @_start1:rel8 ; e[005]
```

### e006

#### description

This error is provoked, when you tried to use prefix mnemonic (like `LOCK`), when instruction does not allow for it.

#### example

```
_start:
    add mov rax, rbx ; e[006]
```

### e007

#### description

This error is provoked, when you try to use forbidden operand combination on instruction.

#### example

```
_start:
    mov dr0, dword (rax) ; e[007]
```

### e008

#### description

This error is provoked, when instruction expects other operand type than one that was provided and when you provide merger/parser with invalid combination

#### example

```
section ".text"
    bits ; e[008]
    align "invalid_input" ; e[008]
_start:
    mov xmm0, rax ; e[008]
```

### e009

#### description

This error is provoked, when instruction expects operand number larger (or other number) than what was found.

#### example

```
_start:
    mov rax ; e[009]
```

### e010

#### description

This error is provoked, when instruction requires REX or EVEX prefix, when bits parameter is not set to 64.

#### example

```
#(bits=32)
_start:
    mov r8d, 10 ; e[010]
    mov rax, 10 ; e[010]
    stosq ; e[010]
```

### e011

#### description

This error is provoked on errors that occur, when invalid addressing (`()` closure) is used.

#### example

```
(rax + ecx) ; e[011]
() ; e[011]
```

### e012

#### description

This error is provoked, when something goes wrong while extending `AST` with `AST::extend()` (when using `include` directive).

#### example

main.asm:

```
include lib.asm

_label:
    call @_exit
```

lib.asm:

```
; e[012] - symbol is redeclared twice
_label: 
    ; [...]
```

### e013

#### description

This error is provoked if something wents wrong, while reading/writing a file.

### e014

#### description

This error is provoked if invalid target (format) is used.

### e015

#### description

This error is provoked, when you try to use root node, when you are not inside root (or vice versa).

#### example

Let's take for an example: `define` directive.

```
align 10 ; invalid, align cannot be used in root
define name 10 ; correct
_start:
    define name 10 ; e[015]
```

### e016

#### description

This error is provoked, when mask is being used, but instruction does not support that. This error is provoked by checks with `CheckAPI`.

#### example

```
mov {k2} ; e[016]
```

### e017

#### description

This error is provoked, when you use directives in wrong way (provide wrong arguments, etc.).

#### example

```
align 65536 ; e[017] - align accepts 16-bit unsigned integer
bits 63 ; e[017] - bits only accept values: 16, 32 or 64
define name 10 some_garbage ; e[017]: expected name and 10, but not some_garbage
```

### e018

#### description

This error is provoked, when you tried to use unknown subexpression.

#### example

```
mov rax, eax {unknown-subexpression} ; e[018]
```

### e019

#### description

This error is provoked, when you tried too many mnemonics in one instruction.

#### example

```
lock lock add rax, eax ; e[019]
```

### e020

#### description

This error is provoked, when you try to use unknown attribute on a label or when you provide inline attributes without label.

#### example

```
// e[020]
#(unknown)
unknown label: ; [...]

public ; e[020]
```

### e021

#### description

This error is provoked, when you try to redeclare one thing multiple times.

#### example

```
format elf64 ; declaration here
format bin   ; redeclaration here
```

### e101

#### description

This error is provoked by creating symbol reference without name.

### e102

#### description

This error is provoked by using unknown mathematical operation inside `$()` closure.

### e103

#### description

This error is provoked by using mathematical operation that requires two operands, but only 1 (or 0) were found.

### e104

#### description

This error is provoked by using two numbers without an operation symbol inside `$()` closure.

#### example

```
$(10 20) ; this will throw an error
; instead use something like:
; $(10 + 20)
```

### e105

#### description

This error is provoked by using string literal that could not be parsed into number inside `$()` closure and for values prefixed with `$`.
