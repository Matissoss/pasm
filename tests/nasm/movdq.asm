section .text
	bits 64
	global _start
_start:
	movd mm0, eax
	movd eax, mm0
	movq mm0, rax
	movq rax, mm0
