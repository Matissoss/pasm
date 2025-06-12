.bits $64
_start:
	imul %al
	imul %ax
	imul %eax
	imul %rbx
	imul (%rax) .byte
	imul (%rax) .word
	imul (%rax) .dword
	imul (%rax) .qword
	imul %rax, (%rax) .qword, $10
	imul %rax, %rbx, $10
