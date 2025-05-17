section .text
	bits 64
	global _start
_start:
	vrcpps xmm0, xmm1
	vrcpps xmm0, oword [rax]
	
	vrcpps ymm0, ymm1
	vrcpps ymm0, yword [rax]

	vrcpps xmm8, xmm9
