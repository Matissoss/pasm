<div align=center>
    <h1>syntax-std.md</h1>
</div>

## file layout

```
ROOT - before sections/labels
section 0
    label 0
    [...]
    label n
[...]
section n
    label 0
    [...]
    label n
```

## quick introduction

`pasm`'s syntax is very simillar to what will you find in Intel-like assemblers (mostly FASM), but as everything: it does a lot of things on its own.

Here are they...

## comments

In `pasm` comments can start with `;` or `//`

## memory addressing

In contrast to Intel-like syntax, `pasm` uses `()` instead of `[]`. Almost everything else is the same.

Hovewer, you will need to specify size of addressed memory with size directive before:

```
qword (rax)
```

## labels

Labels are the same as in every other x86 assembler, but you can use external and inline attributes.

```
#(bits=64) ; this is external attribute
public main: ; "public" part is inline attribute
    ; [...]
protected main:
    ; [...]
```

## symbol referencing

Symbol referencing requires to use `@` prefix. 

```
mov rax, @symbol
```

`pasm` also allows for "extended" relocations by using modifier: `@symbol_name:reltype:/addend`

```
mov rax, @symbol:rel32:10
mov rax, @symbol:abs32:-10
```

Hovewer, if you want to "dereference" a symbol (treat it as memory addressing), then you have to use size specifier just like in memory.

```
qword @symbol
```

## sections

You can define sections just like in FASM:

```
section ".text" alloc executable
```

You can also specify section's address align.

```
section ".text"
    align 16
    alloc       // this way is also fine
    executable  // same
```

## define

To define variable use `define` directive along with name and value (uint).

It is refrenced same as symbol (with `@` prefix) and it is inlined (and isn't exported even in `elf` target):

```
define pi 3.14

main:
    mov rax, @pi ; pasm will inline value of pi here
```

> [!NOTE]
> This directive can only be used inside `ROOT`.

## format

You can also specify output format:

```
format "elf64" ; only Little-Endian for now
```

Accepted values are:
- "elf64"
- "elf32"
- "bin"

> [!NOTE]
> This directive can only be used inside `ROOT`.

## output

You can specify default output path:

```
output "a.out"
```

> [!NOTE]
> This directive can only be used inside `ROOT`.

## size directives

| Size (bits) | directive  |
|:-----------:|:----------:|
|      8      |    byte    |
|     16      |    word    |
|     32      |   dword    |
|     64      |   qword    |
|    128      |   xword    |
|    256      |   yword    |
|    512      |   zword    |

## prefixes

`pasm` also allows for prefixing (because originally it was supposed to work only with prefixes).

Here is table of prefixes along with expected type:

| prefix (default) |   type    |
|:----------------:|:---------:|
| %                | register  |
| $                | immediate |
| @                | symbol ref|
| .                | directive |
