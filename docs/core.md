# Core.md

## About
This note contains info about instruction encoding for x86_64 architecture.

# Content

## Shortcuts

This note contains some shortcuts you should be familiar with:
- Op    : Operand
- OpS   : Operand Size
- Addr  : Address
- AddrS : Address Size
- r     : (depends on context) register or read
- m     : memory
- w     : write
- rw    : read-write
- ro    : read-only
- dst   : destination
- src   : source

## Instruction Format

x86_64 Instruction are saved in Little-Endian (reverse) and in following order:

- Legacy Prefix (we skip some parts here) [1-4B, optional]
- Opcode with Prefixes [1-4B, required]
- ModR/M [1B, if required]
- SIB [1B, if required]
- Displacement [1, 2, 4 or 8B, if required]
- Immediate [1, 2, 4 or 8B, if required]

## Legacy Prefix

| Number | Prefix Group | Prefix          |
|:------:|:------------:|:---------------:|
|  0xF0  |      1       |  LOCK           |
|  0xF2  |      1       |  REPNE/REPNZ    |
|  0xF3  |      1       |  REP/REPE/REPZ  |
|   -    |      2       |   (skipped)     |
|  0x66  |      3       |  OpS override   |
|  0x67  |      3       |  AddrS override |

> [!NOTE]
> When there are 2 or more prefixes from single group, the behaviour is undefined !

### LOCK

**LOCK** is used for rw operations in atomic way. It can only be used with following instructions, 
otherwise `Invalid Opcode Exception` occurs:
- ADC, 
- ADD, 
- AND, 
- BTC, 
- BTR, 
- BTS, 
- CMPXCHG, 
- CMPXCHG8B, 
- CMPXCHG16B, 
- DEC, 
- INC, 
- NEG, 
- NOT, 
- OR, 
- SBB, 
- SUB, 
- XADD, 
- XCHG,
- XOR

### REP*

> *part directly from* [wiki.osdev.org](https://wiki.osdev.org/X86-64_Instruction_Encoding#REPNE/REPNZ,_REP_and_REPE/REPZ_prefixes)

The repeat prefixes cause string handling instructions to be repeated.

The REP prefix will repeat the associated instruction up to CX times, decreasing CX with every repetition. 
It can be used with the INS, LODS, MOVS, OUTS and STOS instructions.

REPE and REPZ are synonyms and repeat the instruction until CX reaches 0 or when ZF is set to 0. 
It can be used with the CMPS, CMPSB, CMPSD, CMPSW, SCAS, SCASB, SCASD and SCASW instructions.

REPNE and REPNZ also are synonyms and repeat the instruction until CX reaches 0 or when ZF is set to 1. 
It can be used with the CMPS, CMPSB, CMPSD, CMPSW, SCAS, SCASB, SCASD and SCASW instructions

### AddrS and OpS override

The default OpS and AddrS can be overriden using prefixes found in table below 
(assuming you're in `Long 64-bit mode`):

|   REX.w   |   Prefix (0x66 if Op, 0x67 if Addr)   |    OpS    |   AddrS   |
|:---------:|:-------------------------------------:|:---------:|:---------:|
|     0     |               No                      |    32     |    64     |
|     0     |               Yes                     |    16     |    32     |
|     1     |               No                      |    64     |    64     |
|     1     |               Yes                     |    64     |    32     |

> [!NOTE]
> Some instructions default to 64-bit operands and do not need REX prefix.

## Opcode

### Legacy Opcode

Legacy opcode consists of (ordered):
- mandatory prefix,
- REX prefix,
- opcode

#### Mandatory Prefix

Some instructions (most notably SIMD) require mandatory prefix (0x66, 0xF2 or 0xF3). 
When mandatory prefix is needed, it is put with modifier prefixes BEFORE REX prefix (if any)

#### REX Prefix

> [!NOTE]
> Only availiable in `Long Mode`.

REX Prefix must be added when:

- Using 64-bit operand, while instruction DOES NOT default to 64-bit size,
- Using extended registers (found in [Registers](#Registers), where `REX` = `1`)

In other cases, REX prefix is ignored. If multiple REX prefixes are found, it leads to undefined behaviour.

##### Encoding

```
8   7   6   5   4   3   2   1
+---+---+---+---+---+---+---+
|   b100    | W | R | X | B |
+---+---+---+---+---+---+---+
```

- **0100** : fixed bit pattern
- **W bit**: 1 = 64-bit operand used; 0 = default operand size;
- **R bit**: extension in MODRM.reg field
- **X bit**: extension in SIB.index
- **B bit**: extension to either MODRM.reg or SIB.base

#### VEX/EVEX prefix

will be covered later, if this assembler will even manage into MVP product...

### MODR/M

```
8   7   6   5   4   3   2   1   0
+---+---+---+---+---+---+---+---+
|  mod  |    reg    |     rm    |
+---+---+---+---+---+---+---+---+
```

- **MODRM.mod** - 2 bits: When this field is 0b11, then register-direct addressing mode is used, 
otherwise register-indirect addressing mode is used.
- **MODRM.reg** - 3 bits: This field can have 1 of these 2 values:
    - 3-bit opcode extension: to recognize instruction A from instruction B
    - 3-bit register reference: which can be used as src/dst for instruction (depending on instruction). 
    Referenced register depends on OpS of instruction and instruction itself. 
    REX.R field can extend this field with 1 most-significant bit to 4-bits total.
- **MODRM.rm** - 3 bits: Specifies direct/indirect register operand, optionally with displacement. REX.B field can extend this field with 1 most-significant bit to 4-bits total.

#### MODRM.mod and displacement
| MODRM.mod | Displacement          |
|:---------:|-----------------------|
|   0b00    | 0B (or 4B if disp32)  |
|   0b01    | 1B - short disp       |
|   0b10    | 4B - long disp        |
|   0b11    | No SIB/disp           |

> [!WARNING]
> **IF MODRM.mod = 0b00 AND SIB.base = 0b101**:
> (situations like this: `mov %reg, disp32(_, %index) !scale`)
> you need to add 4B displacement anyways. Only case, where in `MODRM.mod = 0b00` you need to use disp32

#### 16-bit addressing

See [wiki.osdev.org](https://wiki.osdev.org/X86-64_Instruction_Encoding#16-bit_addressing)

#### 32/64-bit addressing
See [wiki.osdev.org](https://wiki.osdev.org/X86-64_Instruction_Encoding#32/64-bit_addressing)

### Registers


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


> [!NOTE]
> If you use register from table where `REX` is `1`, then you need to use `REX` prefix.

> [!NOTE]
> When any REX prefix is used SPL, BPL, SIL or DIL will be used. Otherwise AH, CH, DH or BH will be used

### SIB

```
8   7   6   5   4   3   2   1   0
+---+---+---+---+---+---+---+---+
| scale |   index   |   base    |
+---+---+---+---+---+---+---+---+
```

- SIB.scale - 2 bits: this indicates scaling factor (either byte, word, dword, qword), where scale = `2^SIB.scale`.
- SIB.index - 3 bits: register to use. See [#Registers](#Registers). REX.X can extend this field to 4 bits.
- SIB.base  - 3 bits: register to use. See [#Registers](#Registers). REX.B can extend this field to 4 bits.

#### 16-bit addressing
See [wiki.osdev.org](https://wiki.osdev.org/X86-64_Instruction_Encoding)

#### 32/64-bit addressing
See [wiki.osdev.org](https://wiki.osdev.org/X86-64_Instruction_Encoding)

### Displacement
> [!NOTE]
> See [wiki.osdev.org](https://wiki.osdev.org/X86-64_Instruction_Encoding#Displacement)

A displacement value is a 1, 2, 4, or 8 byte offset added to the calculated address. 
When an 8 byte displacement is used, no immediate operand is encoded.

The displacement value, if any, follows the ModR/M and SIB bytes discussed before. 
When the ModR/M or SIB tables state that a disp value is required, or without a ModR/M byte the use of moffset (AMD) or 
moffs (Intel) in the mnemonic syntax of the instruction, then the displacement bytes are required.

### Immediates

Some instructions require an immediate value. 

The instruction determine the length of the immediate value. 
- imm8  (or 8-bit operand-size ): means a one byte immediate value, 
- imm16 (or 16-bit operand-size): means a two byte immediate value, 
- imm32 (or 32-bit operand-size): a four byte value,
- imm64 (or 64-bit operand-size): an eight byte value. 

When an 8 byte immediate value is encoded, no displacement can be encoded.

# Credits
Most of info here comes from [wiki.osdev.org](https://wiki.osdev.org) and other opcode tables and **Some parts directly come from** [wiki.osdev.org](https://wiki.osdev.org).

For opcode table see this source: [felixcloutier.com/x86](https://www.felixcloutier.com/x86/)
