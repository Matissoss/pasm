<div align=center>
    <h1>encoding-evex.md</h1>
</div>

## introduction

EVEX as you may know was introduced with `AVX-512`. It is 4B prefix that allows for encoding instructions like: `vaddps xmm21 {k2}{z}, xmm22, xmm31`.

Before we get started hovewer you want to know that extension fields (like `R1`, `R0`) are treated like following:

```
+---+---+---+---+---+
| R1| R0|    ccc    |
+---+---+---+---+---+
```

## encoding

```
+---+---+---+---+---+---+---+---+
| 0   1   1   0   0   0   1   0 |
+---+---+---+---+---+---+---+---+
| R1| X0| B0| R0| 0 | m | m | m |
+---+---+---+---+---+---+---+---+
| W | v   v   v   v | 1 | p   p |
+---+---+---+---+---+---+---+---+
| z | L   L | b | ve| a   a   a |
+---+---+---+---+---+---+---+---+
```

Explaination:

- `R0`, `X0` and `B0` are derived from VEX and encoded the same way.

- `R0` extends `ModRM.reg` by 1 bit; INVERTED
- `X0` extends `SIB.index` or `ModRM.r/m` depending on operand; INVERTED
- `B0` extends `SIB.base` or `ModRM.r/m` (`X0` is first, then `B0`, then `ccc`); INVERTED

- `mmm`: map_select:

| mmm | Opcode Map | Notes |
|:---:|:----------:|:-----:|
| 000 | 0x0F       |   -   |
| 001 |     ?      | unused|
| 010 | 0x0F 0x38  |   -   |
| 011 | 0x0F 0x3A  |   -   |
| 100 |     ?      | unused|
| 101 |     -      | AVX-512-FP16 |
| 110 |     -      | AVX-512-FP16 |

- `W`: equivalent to `VEX.w/e`
- `vvvv`: equivalent to `VEX.vvvv`; also inverted
- `pp`: equivalent to `VEX.pp` (1:1 copy)
- `z`: if 1 then it means that instruction used `{z}` subexpression.
- `LL`: vector length:

| LL | Size |
|:--:|:----:|
| 00 | XMM  |
| 01 | YMM  |
| 10 | ZMM  |
| 11 |  -   |

- `b`: depending on instruction: either broadcasting (most common), `{er}` or `{sae}`
- `ve`: extension to `VEX.vvvv`; INVERTED
- `aaa`: mask code

### sae (supress all exceptions)

If instruction can use `{sae}` and `b` is set, then it will use `sae`.

### er (embedded rounding)

If instruction can use `{er}` and if `b` is set, then instruction will use embedded rounding.

Embedded rounding can have up to 4 modes:

- rn-sae (0b00)
- rd-sae (0b01)
- ru-sae (0b10)
- rz-sae (0b11)

They are encoded in `LL`.

## references

- [wikipedia](https://en.wikipedia.org/wiki/EVEX_prefix)
- [pasm/src/core/evex.rs](https://github.com/Matissoss/pasm/blob/main/src/core/evex.rs)
