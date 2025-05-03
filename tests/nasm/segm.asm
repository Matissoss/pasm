section .text
	global _start
	bits 64

_start:
	mov eax, fs:[rax]
	mov ebx, cs:[rbx]
	mov ecx, es:[rax + 10]
	mov edx, ss:[rbx + rcx * 4]
	mov esp, gs:[rbx + rcx * 4 + 10]
	mov edi, ds:[rbx + rcx * 4 - 10]
