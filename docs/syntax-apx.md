<div align=center>
    <h1>syntax-apx.md</h1>
</div>

## about

This file describes how to use `Intel APX` in `pasm` assembler.

> [!WARNING]
> Intel APX support is untested and probably will contain bugs. Use with caution and don't rely on it for serious projects.
> Maybe someday I'll try to actually test it.
> - matissoss

## mnemonics

Legacy mnemonics that were extended with `EVEX` (like `ADD`) are prefixed `A`.

## conditional test and cmp

`pasm` does not have the `{dfv}` subexpression.

Instead you have to use: `{of}`, `{cf}`, `{sf}` and `{zf}` subexpressions.

## {nf}

NF indicator is defined by `{nf}` subexpression. These are checked.

Hovewer to use `{nf}` on extended VEX instruction you have to use subexpression `{vex-nf}` (this will be later fixed, don't worry; WILL BE FIXED IN DEV BRANCH).

## {eevex} and {rex2}

These subexpressions are used to specified to force instruction into either using `{rex2}` or `{eevex}` (if it isn't able to encode `{rex2}`, then it will encode `{eevex}` instead) - these are only hints for the assembler that assembler can (but does not need to) utilize.

## {nd=zu}

`zu` suffix is added to mnemonic as suffix (like `aimulzu` and `aimul`).
