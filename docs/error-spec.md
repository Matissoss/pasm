<div align=center>
    <h1>error-spec.md</h1>
</div>

## a0000 - internal error

If you see this error, it is most probably a bug.

## a0001 - unclosed delimeter

Provokes when a `(`, `"`, `'` delimeter is unclosed.

Example:

```
mov rax, (rcx    ; a0001: unclosed ( delimeter
mov rax, "string ; a0001: unclosed " delimeter
mov rax, 'c      ; a0001: unclosed ' delimeter
```

## a0002 - too many closing delimeters

Provokes, when there are more `)` in line than `(`.

Example:

```
mov rax, (rcx)) ; a0002
```

## a0003 - parsing error

Provokes, when parser encounters unexpected input, most probably by what it interprets as directive without value.

Example:

```
this-will-give-you-error-code ; a0003
```

## a0004 - invalid subexpression

Provokes, when you try to use invalid subexpression.

Example:
```
mov rax, {invalid-subexpression} ; a0004
```

## a0005 - operand parsing error

Provokes, when parser tries parsing an operand and you provide it an invalid input.

Example:
```
cword [rax] ; a0005
[rax] ; a0005
```

## a0006 - file I/O error

Provokes, when assembler wasn't able to open, read or write to/from a file.

## a0007 - invalid target name

Provokes, when assembler does not recognize provided's target's name.
