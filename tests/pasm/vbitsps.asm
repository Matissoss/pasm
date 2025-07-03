bits 64
_start:
	vorps xmm0, xmm1, xmm2
	vorps xmm8, xmm9, xmm10
	vorps xmm8, xmm9, xword (rax)

	vorps ymm0, ymm1, ymm2
	vorps ymm8, ymm9, yword (rax)
	vorps ymm9, ymm0, ymm10
	
	vandps xmm0, xmm1, xmm2
	vandps xmm8, xmm9, xmm10
	vandps xmm8, xmm9, xword (rax)

	vandps ymm0, ymm1, ymm2
	vandps ymm8, ymm9, yword (rax)
	vandps ymm9, ymm0, ymm10
	
	vandnps xmm0, xmm1, xmm2
	vandnps xmm8, xmm9, xmm10
	vandnps xmm8, xmm9, xword (rax)

	vandnps ymm0, ymm1, ymm2
	vandnps ymm8, ymm9, yword (rax)
	vandnps ymm9, ymm0, ymm10
	
	vxorps xmm0, xmm1, xmm2
	vxorps xmm8, xmm9, xmm10
	vxorps xmm8, xmm9, xword (rax)

	vxorps ymm0, ymm1, ymm2
	vxorps ymm8, ymm9, yword (rax)
	vxorps ymm9, ymm0, ymm10
