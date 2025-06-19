<div align=center>
    <h1>encoding-std.md</h1>
</div>

## encoding order

Legacy Prefix -> Opcode with Prefix -> MODRM -> SIB -> Displacement -> Immediate

## legacy prefixes

Only one of prefixes from these groups can be used at once (except `override prefixes`).

Meaning of following prefixes (especially ones from `lock/repXX`) can be used to differ opcodes (in SSE).

### lock/repXX prefixes

- 0xF0 - LOCK prefix
- 0xF2 - REPNE/Z prefix
- 0xF3 - REP prefix

### segment prefixes

- 0x2E - CS
- 0x36 - SS
- 0x3E - DS
- 0x26 - ES
- 0x64 - FS
- 0x65 - GS

> [!NOTE]
> In long mode ES, DS, SS and CS segment overrides are ignored.

### override prefixes

0x66 - Operand Size Override
0x67 - Address Size Override (size of registers used in memory addressing)

| Bits | Prefix | Operand Size | Address Size |
|:----:|:------:|:------------:|:------------:|
|  16  |  No    |     16       |       -      |
|  16  |  Yes   |     32       |      32      |
|  32  |  Yes   |     16       |       -      |
|  32  |  No    |     32       |      32      |
|  64  |  No    |     32       |      64      |
|  64  | Yes    |     16       |      32      |
|  64  | No     | 64 (if REX.W)|       -      |
|  64  | Yes    | 64 (if no REX.W)|    -      |

> [!NOTE]
> Some opcodes default to 64-bit and don't need REX prefix

## rex prefix

REX prefix is used in long mode (64-bit) to allow for usage of extended (r8-15) registers and use of 64-bit register (rXX).

REX prefix is used if:
- instruction does not default to 64-bit (minority of instructions)
- you use extended registers (r8-15, xmm8-15 (in context of SSE), cr8-15, dr8-15)
- using SPL, DIL, BPL or SIL 8-bit registers and AH, BH, CH, DH 8-bit registers are not used.

REX prefix takes 1 byte to encode in following bit order:

```
+---+---+---+---+---+---+---+---+
| 0   1   0   0 | W | R | X | B |
+---+---+---+---+---+---+---+---+
```

- `0100` - fixed prefix
- `W` - if `1`: 64-bit operand size is used, `0`: default operand override is used
- `R` - extension to `MODRM.reg`
- `X` - extension to `SIB.index`
- `B` - extension to `MODRM.r/m` or `SIB.base`

## opcode

Opcode are basically part to differ beetwen instructions. Opcode can have: 1, 2 or 3 bytes in length.

- `<OPCODE>`
- `0x0F <OPCODE>`
- `0x0F 0x38 <OPCODE>`
- `0x0F 0x3A <OPCODE>`

> [!NOTE]
> `0x0F`/`0x0F 0x38`/`0x0F 0x3A` are called opcode maps. Same terminology is used in VEX prefix (`map_select`).

## modrm

MODRM tell us which operands to use in instruction. MODRM takes 1 byte to encode:

```
+---+---+---+---+---+---+---+---+
|  mod  |    reg    |    r/m    |
+---+---+---+---+---+---+---+---+
```

- `MODRM.mod`: tells us which addressing mode to use:
    - `0b00`: register + memory addressing without displacement
    - `0b01`: register + memory addressing with 1 byte displacement
    - `0b10`: register + memory addressing with 4 byte displacement
    - `0b11`: register + register
- `MODRM.reg`: specifies one of operands. Some instructions use this field to differ from other variants.
- `MODRM.r/m`: specifies one of operands.
    - If it is set to value of `*SP` register and `MODRM.mod != 0b11`, then `SIB` is used.
    - If it is set to value of `*BP` register and `MODRM.mod != 0b11` then `*IP`-relative addressing mode is used.

My recommendation to encoding `MODRM` is that you do NOT assume that `MODRM.reg` and `MODRM.r/m` refer to destination/source, because:

- If you will add VEX encoding, you will need to create new modrm function, because it is often to see `MODRM.reg, VEX_VVVV, MODRM.r/m` operand order (or same, but with `MODRM.reg` and `MODRM.r/m` swapped in places).
- If instruction is something like `mov .dword (%rax), %eax`, then yes: 
`MODRM.r/m` refers to dst operand (`.dword (%rax)`) and `MODRM.reg` refers to src operand (`%eax`). 
But if we reverse the order (`mov %eax, .dword (%rax)`), then 
`MODRM.r/m` refers to `.dword (%rax)` (src) and `MODRM.reg` refers to `%eax` (dst).

In `rasm` we used that approach before, but it was replaced with `GenAPI` by `.ord()` and `.get_ord_oprs()` function that allow to set order and then retrieve operands used by `MODRM.reg` and `MODRM.r/m`.

For more info see `rasm/src/core/modrm.rs` (it is 79 LOC of somewhat readable code).

## sib

SIB (Scale, Index, Base) is used to encode more advanced memory addressing like: `(%rax + %rcx * $1 + $20)`. The formula for getting the address is same as shown before: `base_register + (index_register * scale) + offset`.

You encode SIB when: memory addressing uses `scale` or `index`.

To encode SIB you will need one byte:

```
+---+---+---+---+---+---+---+---+
| scale |   base    |   index   |
+---+---+---+---+---+---+---+---+
```

- `scale`:

| Value | Scale |
|:-----:|:-----:|
| 0b00  |   1   |
| 0b01  |   2   |
| 0b10  |   4   |
| 0b11  |   8   |

- `base`: specifies `base_register`
- `index`: specifies `index_register`

Sometimes hovewer you will encounter variant like: `(%rcx * $1)` (no base register). It means that you have to set base register to `*BP*`.

## displacement/immediate

Displacement and immediates are encoded in x86 in Little Endian (not Big Endian!). Displacement is just offset to SIB/base and Immediate is just constant number.

## reading opcode tables

We will go over very basics of reading opcode tables from Intel.

### most common symbols used in opcode tables

- `ib`: 8-bit immediate
- `iw`: 16-bit immediate
- `id`: 32-bit immediate
- `io`: 64-bit immediate
- `rb`: 8-bit register
- `rw`: 16-bit register
- `rd`: 32-bit register
- `/r`: `MODRM.reg` is set to register.
- `/n`: `MODRM.reg` is set to number `n`.
- `REX.W +`: this instruction variant requires `REX.W` set.
- `N+ rX`: add to opcode value of register
- `rXX`: `XX`-bit register
- `mXX`: `XX`-bit memory
- `immXX`: `XX`-bit immediate
- `r/mXX`: `XX`-bit register or memory

### decoding opcode table entry + examples

Let's go with easy one: `MOV r32, imm32`. Opcode "formula" for this instruction is: `B8 +rd id`.

- `B8 +rd`: opcode + value of `r32` register.
- `id`: 32-bit immediate

> [!NOTE]
> We don't use MODRM in this case. Why? Because we don't have `/n`/`/r` part in our opcode.

Next one: `MOV r64, r/m64`. Opcode for this instruction is: `REX.W + B8 /r`.

- `REX.W +`: our instruction requires `REX.W` to be set.
- `B8`: opcode
- `/r`: `MODRM.reg` is set to value of register (we use MODRM).

> [!NOTE]
> All opcodes in Intel's opcode table are hex numbers.

### encoding first instruction

Let's go with previous example: `MOV r64, r/m64`.

Data:
- Opcode: `REX.W + B8 /r`
- Operand Encoding: `RM` (`MODRM.reg`, `MODRM.r/m`)

Let's assert that we use `mov %rax, %rcx` variant.

We do not need Operand Size Override prefix, because we use `REX.W`.

We first start with encoding our REX prefix. We don't need to set `R`/`X`/`B` bits here, so only `W` bit is set.

Then we add opcode: `0xB8`.

Then we encode MODRM:

- `MODRM.mod` is set to `0b11`, because we only use registers.
- `MODRM.reg` is set to `0b000` (`*A*` register)
- `MODRM.r/m` is set to `0b001` (`*C*` register)

We don't need SIB, displacement and immediate here, so we skip.

Our final result: `0x48 0xB8 0xC1`.

## register encoding table

| REX |  Bits | 8-bit GP | 16-bit GP | 32-bit GP | 64-bit GP | 128-bit XMM | 256-bit YMM |
|:---:|:-----:|:--------:|:---------:|:---------:|:---------:|:-----------:|:-----------:|
|  0  |  000  |    AL    |    AX     |    EAX    |    RAX    |    XMM0     |     YMM0    | 
|  0  |  001  |    CL    |    CX     |    ECX    |    RCX    |    XMM1     |     YMM1    | 
|  0  |  010  |    DL    |    DX     |    EDX    |    RDX    |    XMM2     |     YMM2    | 
|  0  |  011  |    BL    |    BX     |    EBX    |    RBX    |    XMM3     |     YMM3    | 
|  0  |  100  |  AH/SPL  |    SP     |    ESP    |    RSP    |    XMM4     |     YMM4    | 
|  0  |  101  |  CH/BPL  |    BP     |    EBP    |    RBP    |    XMM5     |     YMM5    | 
|  0  |  110  |  DH/SIL  |    SI     |    ESI    |    RSI    |    XMM6     |     YMM6    | 
|  0  |  111  |  BH/DIL  |    DI     |    EDI    |    RDI    |    XMM7     |     YMM7    | 
|  1  |  000  |   R8B    |    R8W    |    R8D    |    R8     |    XMM8     |     YMM8    | 
|  1  |  001  |   R9B    |    R9W    |    R9D    |    R9     |    XMM9     |     YMM9    | 
|  1  |  010  |   R10B   |    R10W   |    R10W   |    R10    |    XMM10    |     YMM10   | 
|  1  |  011  |   R11B   |    R11W   |    R11W   |    R11    |    XMM11    |     YMM11   | 
|  1  |  100  |   R12B   |    R12W   |    R12W   |    R12    |    XMM12    |     YMM12   | 
|  1  |  101  |   R13B   |    R13W   |    R13W   |    R13    |    XMM13    |     YMM13   | 
|  1  |  110  |   R14B   |    R14W   |    R14W   |    R14    |    XMM14    |     YMM14   | 
|  1  |  111  |   R15B   |    R15W   |    R15W   |    R15    |    XMM15    |     YMM15   | 

## sources

- https://wiki.osdev.org/X86-64_Instruction_Encoding
- https://cdrdv2-public.intel.com/851038/325462-087-sdm-vol-1-2abcd-3abcd-4.pdf (Intel PDF's), espencially: `Volume 2 chapter 2`
