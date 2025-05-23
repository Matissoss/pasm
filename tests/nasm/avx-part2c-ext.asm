section .text
	bits 64
	global _start
_start:
	vpmaxud xmm0, xmm1, xmm2
	pmaxsw xmm0, xmm1
	pmaxsw mm0, mm1
	pminsw xmm0, xmm1
	pminsw mm0, mm1
	pmulhuw xmm0, xmm1
	pmulhuw mm0, mm1
	pmaxud xmm0, xmm1
	pinsrw xmm0, eax, 10
	pinsrw xmm0, word [rax], 10
