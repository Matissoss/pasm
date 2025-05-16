section .text
	bits 64
	global _start
_start:
	vaddps xmm0, xmm1, xmm2
	vaddps xmm8, xmm9, xmm10
