section .text
	bits 64
	global _start
_start:
	vmovss xmm0, xmm1, xmm2
	vmovss xmm0, dword [rax]
	vmovss dword [rax], xmm0
