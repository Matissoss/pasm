section .text
	bits 64
	global _start
_start:
	vmovaps xmm0, xmm1
	vmovaps oword [rax], xmm1
	vmovaps xmm0, oword [rax]
	
	vmovaps ymm0, ymm1
	vmovaps yword [rax], ymm1
	vmovaps ymm0, yword [rax]
