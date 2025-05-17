section .text
	bits 64
	global _start
_start:
	vsqrtps xmm0, xmm1
	vsqrtps xmm0, oword [rax]

	vsqrtps ymm0, ymm1
	vsqrtps ymm0, yword [rax]
	
	vrsqrtps xmm0, xmm1
	vrsqrtps xmm0, oword [rax]

	vrsqrtps ymm0, ymm1
	vrsqrtps ymm0, yword [rax]
