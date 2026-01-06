<div align=center>
    <h1>syntax-avx512.md</h1>
</div>

## subexpressions

Subexpressions are written just like in other assemblers, but they must be separated with `,` from operands and each other.

Example:
```
xmm0,{k1},{z},xmm2,xmm3
```

## bcst

To use memory broadcasting we use `{bcst}` subexpression.
