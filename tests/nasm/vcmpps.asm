section .text
	bits 64
	global _start
_start:
	vcmpps xmm0, xmm1, xmm2, 1
	vcmpps xmm8, xmm9, xmm10, 1
	vcmpps xmm8, xmm9, oword [rax], 1

	vcmpps ymm0, ymm1, ymm2, 1
	vcmpps ymm8, ymm9, yword [rax], 1
	vcmpps ymm9, ymm0, ymm10, 1
