format "elf64"
bits 64

section ".data"
        alloc
        writeable
        hello_world:
                string "Hello, World!\n\0"
	public hello_world
section ".text"
        alloc
        executable
        _start:
	public _start
		mov rdi, @[hello_world, abs32]
		call @[strlen]
		
		mov rdx, rax
		mov rax, 1
		mov rdi, 1
		mov rsi, @[hello_world, abs32]
		syscall

		mov rax, 60
		mov rdi, 0
		syscall
	strlen:
	public strlen
                xor rcx, rcx
	_strlen_loop:
		mov al, byte [rdi]
		cmp al, 0
		je @[_strlen_end]
		inc rcx
		inc rdi
		jmp @[_strlen_loop]
	_strlen_end:
		mov rax, rcx
		ret
