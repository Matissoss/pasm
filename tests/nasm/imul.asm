section .text
	bits 64

_start:
	imul al
	imul ax
	imul eax
	imul rbx
	imul byte [rax]
	imul word [rax]
	imul dword [rax]
	imul qword [rax]
	imul rax, qword [rax], 10
	imul rax, rbx, 10
