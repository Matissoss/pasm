<div align=center>
    <h1>rasm</h1>
</div>

## about

rasmx86-64 (or just rasm) is assembler for x86-64 architecture.

> [!NOTE]
> This assembler is only "proof of concept", because
> I didn't know very much about x86-64 architecture at beginning of rasmx86-64, so I made a lot of bad decisions.
> rasm only implements basics: SIB, displacement, REX, ModRM, Registers, immediates and opcode. 
> No support for constants and sections (which I also messed up). No support for `.o` files, only flat binary.
>
> Hovewer, you can expect, that I'll make another x86-64 assembler in close future, learning from my mistakes. :)
>
> -- matissoss

## roadmap

- [x] Frontend for assembler (tokenizer, lexer, parser)
- [x] AST
- [ ] Instruction encoding support (with REX, ModRM, registers, SIB, displacement and immediates)

## credits

made by matissoss [matissossgamedev@proton.me]

licensed under MPL 2.0
