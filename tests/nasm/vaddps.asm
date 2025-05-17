section .text
	bits 64
	global _start
_start:
	vaddps xmm0, xmm1, xmm2
	vaddps xmm8, xmm9, xmm10
	vaddps xmm8, xmm9, oword [rax]

	vaddps ymm0, ymm1, ymm2
	vaddps ymm8, ymm9, yword [rax]
	vaddps ymm9, ymm0, ymm10
