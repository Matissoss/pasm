[bits 64]
_start:
	mov rax, qword [eax + ebx * 1]
	mov rax, qword [rax + rbx * 1]
	mov ax, bx
	or ax, bx
