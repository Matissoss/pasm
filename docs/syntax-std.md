<div align=center>
    <h1>syntax-std.md</h1>
</div>

## Instruction syntax

Instruction syntax is almost same as Intel-like syntax in NASM/FASM, but symbol referencing and sections are distinct.

> [!NOTE]
> Memory addressing must always be prefixed using size prefix such as `qword`, `byte`, etc. Full list of size prefixes can be found below this section.

Example:
```
mov rax, rbx
mov rax, qword [rax + rcx * 4 + 20]
```

To use RIP-relative addressing we'll use another method of memory addressing:

```
mov rax, qword [10] ; (RIP + 10)
```

## Immediate syntax

You can store number using 4 methods:
- **decimal** like `18`
- **binary** like `0b10110`
- **octal** like `0o167`
- **hex** like `0xFF`

You can also use `_` like in Rust to save numbers.

Example:
```
0xFFFF_FFFF
0b1101_1010_1110
0o1743_1243_1365
```

## Size prefixes
Full list of supported size prefixes:

| Name | Size (in bits) |
|:----:|:--------------:|
| byte | 8              |
| word | 16             |
|dword | 32             |
|qword | 64             |
|xword | 128            |
|yword | 256            |
|zword | 512            |

## Symbol Referencing

To reference symbol we'll use `@[]` closure. Inside it we'll provide 3 parameters: symbol name, relocation type (optional) and addend (optional) separated by a comma (`,`).

Example:
```
mov rax, @[strlen]
mov rax, @[strlen, 10]
mov rax, @[strlen, rel32]
mov rax, @[strlen, rel32, -10]
```

To get what is under address of the symbol you'll have to use size prefix just like in memory addressing.

Example:
```
mov rax, qword @[strlen]
mov rax, qword @[strlen, 10]
mov rax, qword @[strlen, rel32]
mov rax, qword @[strlen, rel32, -10]
```

> [!NOTE]
> Full list of supported relocation types can be found in Appendix C

## Supported Relocation types

| Name | ELF equivalent                                                 |
|:----:|----------------------------------------------------------------|
|abs32 | `R_X86_64_32S`                                                 |
|rel32 | `R_X86_64_PC32`                                                |
|rel16 | `R_X86_64_PC16`                                                |
|rel8  | `R_X86_64_PC8`                                                 |
|NONE  | `R_X86_64PC32` or `R_X86_64PC16` depending on `bits` directive |

## Global Directives

Global directives can be used anywhere and they change globally.

Here is full list of them:
| Name    | Parameters + Types      | Behaviour                                                      |
|:-------:|:-----------------------:|----------------------------------------------------------------|
|bits     | value: uint8 (16/32/64) | Sets assembler's bits parameter to `value` (derived from NASM) |
|extern   | value: string           | Adds an external symbol to symbol list                         |
|public   | value: string           | Sets symbol's visibility to global/public                      |
|protected| value: string           | Sets symbol's visibility to protected                          |
|private  | value: string           | Sets symbol's visibility to local/private                      |
|weak     | value: string           | Sets symbol's visibility to weak                               |
|function | value: string           | Sets symbol's type to function                                 |
|object   | value: string           | Sets symbol's type to object                                   |

## Section-related syntax

### Section declaration

To declare a section we'll use following syntax:
```
section <section_name>
```

Example:
```
section cool_section
```

### Section Attributes

To give a section attributes we'll use directives such as:
- `writeable`
- `executable`
- `nobits` (behaviour same as in `.bss` section)
- `alloc`
- `align <UINT16>`

We can use these directives after section declaration, one per line.

Example for `".text"` section:
```
section .text
    executable
    alloc
```

Example for `".data"` section:
```
section .data
    alloc
    writeable
    align 16
```

Example for `".bss"` section:
```
section .bss
    nobits
```

### Labels

Labels like in other assemblers contain instructions. They belong to sections and are written under them.
They are considered symbols in PASM and as such they can be manipulated using directives.

To declare a label we use `<LABEL_NAME>:` just like in other x86-64 assembler syntaxes you may know.

Example:
```
section .text
    alloc
    executable
    public _start
    _start:
        mov rax, 60
        mov rdi, 0
        syscall
```


## Appendixes

### Appendix A
Full list of supported data instructions.

|    Instruction   | Size (in bits) | Byte Order |
|:----------------:|:--------------:|:----------:|
|   bytele/bytebe  |       8        |  LE/BE     |
| wordle           |       16       |  LE        |
| wordbe           |       16       |  BE        |
| dwordle          |       32       |  LE        |
| dwordbe          |       32       |  BE        |
| qwordle          |       64       |  LE        |
| qwordbe          |       64       |  BE        |
| empty            | N (variable)   |  -         |
| string           | N (variable)   |  -         |

### Appendix B
Full list of supported relocation types

### Appendix C
Full list of instructions with changed names.

| Original Name | Changed Name |
|:-------------:|:------------:|
| MOVSD (string)| MOVSTRD      |
| MOVSB (string)| MOVSTRB      |
| MOVSW (string)| MOVSTRW      |
| MOVSQ (string)| MOVSTRQ      |
| CMPSD (string)| CMPSTRD      |
| CMPSB (string)| CMPSTRB      |
| CMPSW (string)| CMPSTRW      |
| CMPSQ (string)| CMPSTRQ      |

### Appendix D
Full list of known edge cases.

#### No Section Provided
For source code like following:

```
_start:
    mov rax, 60
    mov rdi, 0
    syscall
```

Section will be `.text`.
