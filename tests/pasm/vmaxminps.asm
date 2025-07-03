bits 64
_start:
	vminps xmm0, xmm1, xmm2
	vminps xmm8, xmm9, xmm10
	vminps xmm8, xmm9, xword (rax)

	vminps ymm0, ymm1, ymm2
	vminps ymm8, ymm9, yword (rax)
	vminps ymm9, ymm0, ymm10
	
	vmaxps xmm0, xmm1, xmm2
	vmaxps xmm8, xmm9, xmm10
	vmaxps xmm8, xmm9, xword (rax)

	vmaxps ymm0, ymm1, ymm2
	vmaxps ymm8, ymm9, yword (rax)
	vmaxps ymm9, ymm0, ymm10
