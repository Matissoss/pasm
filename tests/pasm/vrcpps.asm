bits 64
_start:
	vrcpps xmm0, xmm1
	vrcpps xmm0, xword (rax)
	
	vrcpps ymm0, ymm1
	vrcpps ymm0, yword (rax)

	vrcpps xmm8, xmm9
