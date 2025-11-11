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
