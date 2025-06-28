<div align=center>
    <h1>error-spec.md</h1>
</div>

### e000

#### description

This error is provoked by using unclosed delimeter.

### e001

#### description

This error is provoked by using signed number that is too large to be signed.

#### example

```
; for 16-bit value
$-0xFFFF ; 0xFFFF is max for unsigned 16-bit value and it is not possible to set sign
```

### e002

#### description

This error is provoked by using invalid character in binary, octal or hexadecimal number formats.

#### example

```
0b200 ; invalid
0xGG  ; invalid
0o88  ; invalid
```

### e003

#### description

This error is provoked, when you provided one token for operand, but it couldn't be converted into operand.

#### example

```
_start:
    .dword ; e[003]. directive .dword cannot be converted into operand
```

### e004

#### description

This error is occured, when an undefined symbol is being used or symbol is redeclared multiple times.

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
    byte $0
#(align=257)
_start:
    jmp @(_start1:rel8) ; e[005]
```

### e006

#### description

This error is provoked, when you tried to use prefix mnemonic (like `LOCK`), when instruction does not allow for it.

#### example

```
_start:
    add mov %rax, %rbx ; e[006]
```

### e007

#### description

This error is provoked, when you try to use forbidden operand combination on instruction.

#### example

```
_start:
    mov %dr0, .dword (%rax) ; e[007]
```

### e008

#### description

This error is provoked, when instruction expects other operand type than one that was provided and when you provide merger/parser with invalid combination

#### example

```
.section ".text"
    .bits ; e[008]
    .align "invalid_input" ; e[008]
_start:
    mov %xmm0, %rax ; e[008]
```

### e009

#### description

This error is provoked, when instruction expects operand number larger (or other number) than what was found.

#### example

```
_start:
    mov %rax ; e[009]
```

### e010

#### description

This error is provoked, when instruction requires REX or EVEX prefix, when bits parameter is not set to 64.

#### example

```
#(bits = 32)
_start:
    mov %r8d, $10 ; e[010]
    mov %rax, $10 ; e[010]
    stosq ; e[010]
```

### e011

#### description

This error is provoked on errors that occur, when invalid addressing (`()` closure) is used.

#### example

```
(%rax + %ecx) ; e[011]
() ; e[011]
```

### e012

#### description

This error is provoked, when something goes wrong while extending `AST` with `AST::extend()` (when using `include` directive).

#### example

main.asm:

```
.include lib.asm

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

This error is provoked when using "avx-512 mnemonic modifier" on instruction that does not support that.

#### example

```
add:k2 ; e[015]
```

### e016

#### description

This error is provoked, when mask is being used, but instruction does not support that. This error is provoked by checks with `CheckAPI`.

#### example

```
mov:k2 ; e[016]
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

### e106

#### description

This error is provoked by using invalid escape character (`\<CHAR>`).

## internal errors (e500)

These errors should not be in finished product. If you encounter one of them, it is certainly a bug/edge case.
