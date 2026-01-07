target elf64

bits 64

// for Linux x86-64 SysV ABI
section .data
        writeable
        alloc
	public hello_world
	hello_world: 
                string "Hello, World!\n"
section .text
        executable
        alloc
	align 16
        public _start
	_start:
		mov rax, 1
		mov rdi, 1
		; we can also use dereference: lea rsi, qword @[hello_world]
		mov rsi, @[hello_world, abs32]
		mov rdx, 19
		syscall

		mov rax, 60
		xor rdi, rdi
		syscall
