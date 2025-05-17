section .text
	bits 64
	global _start
_start:
	vmovapd xmm0, xmm1
	vmovapd oword [rax], xmm1
	vmovapd xmm0, oword [rax]
	
	vmovapd ymm0, ymm1
	vmovapd yword [rax], ymm1
	vmovapd ymm0, yword [rax]

	vmovapd xmm8, xmm9
