<div align=center>
    <h1>encoding-vex.md</h1>
</div>

## about

This is very short document based on `src/core/vex.rs` on how to encode/read instructions using VEX.

## start

VEX prefix has two variants: 2-byte and 3-byte.

We use 3-byte version if:
- `MODRM.r/m` requires extension (which would be provided by `REX` prefix (but it is FORBIDDEN to use it with `VEX`)).
- `map_select` field is not `0b00000`.
- `VEX.w/e` is set.

Otherwise 2-byte version is used.

## 2-byte VEX (VEX2)

VEX2 starts with mandatory `0xC5` as first byte.

The other byte is encoded like this:

```
+---+---+---+---+---+---+---+---+
|~R |     ~vvvv     | L |   pp  |
+---+---+---+---+---+---+---+---+
```

- `~R`: inverse of `REX.R`: extension to `MODRM.reg`
- `~vvvv`: inverse of operand (most commonly for source, BUT there are other cases (especially in BMI extension)).
- `L`: vector length (most common variants): `1`: 256-bit, `0`: 128-bit
- `pp`: legacy prefix:

| pp | Prefix |
|:--:|:------:|
| 00 |  none  |
| 01 |  0x66  |
| 10 |  0xF3  |
| 11 |  0xF2  |

## 3-byte VEX (VEX3)

VEX3 starts with mandatory `0xC4` as first byte.

The other two bytes go as following:

first one:

```
+---+---+---+---+---+---+---+---+
|~R |~X |~B |    map_select     |
+---+---+---+---+---+---+---+---+
```

New things:
- `~X`: inverse of `REX.X`
- `~B`: inverse of `REX.B`
- `map_select`: specifies opcode map (like `0x0F 0x3A`)

second one:

```
+---+---+---+---+---+---+---+---+
|W/E|     ~vvvv     | L |   pp  |
+---+---+---+---+---+---+---+---+
```

New things:
- `W/E`: equivalent to `REX.W`, specified with opcode.

## pro-tips

Easiest way to encode `~` is to use `andn` instruction (`(!lhs) & rhs`).

For example `~vvvv` for `%r8` would be: `!(0b0000_1000) & 0b1111`, so: `0b0111`.

## opcode tables

To find most of instructions that use VEX prefix, you should look for instructions that start with `V` letter (they are most probably AVX instructions, ignore ones with CPUID flag: `AVX-512*`, because they use EVEX prefix).

We will take for example instruction: `VADDPD` and `VADDPS`.

For `VADDPS`:

In this instruction `Opcode` field is: `VEX.(128/256).0F.WIG 58 /r`.

- `VEX.`: we need to use VEX prefix.
- `(128/256)`: depending on used operands, we set `VEX.L` to `0` if 128-bit (`xmm`) and `1` if 256-bit (`ymm`) registers are used.
- `0F`: hex for opcode map `1`
- `WIG`: `VEX.W/E` is ignored (and should be set to `0`).
- rest we covered in `docs/encoding-x86.md`

For `VADDPD`:

In this instruction `Opcode` field is: `VEX.(128/256).66.0F.WIG 58 /r`.

- `VEX.`: we need to use VEX prefix.
- `(128/256)`: depending on used operands, we set `VEX.L` to `0` if 128-bit (`xmm`) and `1` if 256-bit (`ymm`) registers are used.
- `66`: pp = 0b01
- `0F`: hex for opcode map `1`
- `WIG`: `VEX.W/E` is ignored (and should be set to `0`).
- rest we covered in `docs/encoding-x86.md`

## useful code snippets

For generating `pp`:

```
const fn pp(v: u8) -> u8 {
    match v {
        0x66 => 0b01,
        0xF3 => 0b10,
        0xF2 => 0b11
        _ => 0,
    }
}
```

For generating `map_select`:

> [!NOTE]
> `0x38` is opcode map for `0F38` and `0x3A` is opcode map for `0F3A` in opcode tables.

```
const fn map_select(v: u8) -> u8 {
    match v {
        0x0F => 0b00001,
        0x38 => 0b00010,
        0x3A => 0b00011,
        _ => 0b00000,
    }
}
```

Code snippets shown here are from `rasm/src/core/api.rs:pp/map_select` functions.

## sources

Same as in `docs/encoding-x86.md`
