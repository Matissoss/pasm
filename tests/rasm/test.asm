.bits $64
_start:
	test %al, $255
	test %ax, $256
	test %eax, $65536
	test %rax, $65536
	test (%rbx) .byte, $5
	test %bl, $5
	test (%rbx) .word, $256
	test %bx, $256
	test (%rbx) .dword, $65536
	test %ebx, $65536
	test (%rbx) .qword, $65537
	test %rbx, $65536

	test %al, %bl
	test (%rax) .byte, %bl
	test %ax, %bx
	test (%rax) .word, %bx
	test %eax, %ebx
	test (%rax) .dword, %ebx
	test %rax, %rbx
	test (%rax) .qword, %rbx
