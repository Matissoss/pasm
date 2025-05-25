section .text
	bits 64
	global _start
_start:
	andn eax, ebx, ecx
	andn rax, rbx, rcx
	bzhi eax, ebx, ecx
	bzhi rax, rbx, rcx
	bextr eax, ebx, ecx
	bextr rax, rbx, rcx
