# elf.md

> [!NOTE]
> This is only temporary documentation

## relocation table

### x86

> [!NOTE]
> [linux foundation elf pdf](https://refspecs.linuxfoundation.org/elf/TIS1.1.pdf) page 36

|   Type   | Value |   Calculation   |
|:--------:|:-----:|:---------------:|
|NONE      |   0   | none            |
|32        |   1   | S + A           |
|PCRel32   |   2   | S + A - P       |
|GOT32     |   3   | G + A           |
|PLT32     |   4   | L + A - P       |
|COPY      |   5   | none            |
|GLOB_DAT  |   6   | S               |
|JMP_SLOT  |   7   | S               |
|RELATIVE  |   8   | B + A           |
|GOTOFF    |   9   | S + A - GOT     |
|GOTPCRel32|   10  | GOT + A - P     |

### x86-64

> [!NOTE]
> [linux foundation elf pdf](https://refspecs.linuxfoundation.org/elf/x86_64-abi-0.21.pdf) page 42

|   Type   | Value | Size |   Calculation   |
|:--------:|:-----:|:----:|:---------------:|
|NONE      |   0   | none | none            |
|64        |   1   | qword| S + A           |
|PCRel32   |   2   | dword| S + A - P       |
|GOT32     |   3   | dword| G + A           |
|PLT32     |   4   | dword| L + A - P       |
|COPY      |   5   | none | none            |
|GLOB_DAT  |   6   | qword| S               |
|JUMP_SLOT |   7   | qword| S               |
|RELATIVE  |   8   | qword| B + A           |
|GOTPCRel32|   9   | dword| G + GOT + A - P |
| 32       |   10  | dword| S + A           |
| 32S      |   11  | dword| S + A           |
| 16       |   12  | word | S + A           |
| 16S      |   13  | word | S + A           |
|PCRel16   |   14  | word | S + A - P       |
|8         |   15  | byte | S + A           |
|PCRel8    |   16  | byte | S + A - P       |


## sources

- https://refspecs.linuxfoundation.org/elf
