bits 64
_start:
	vunpcklps xmm0, xmm1, xmm2
	vunpcklps xmm8, xmm9, xmm10
	vunpcklps xmm8, xmm9, xword [rax]

	vunpcklps ymm0, ymm1, ymm2
	vunpcklps ymm8, ymm9, yword [rax]
	vunpcklps ymm9, ymm0, ymm10
	
	vunpckhps xmm0, xmm1, xmm2
	vunpckhps xmm8, xmm9, xmm10
	vunpckhps xmm8, xmm9, xword [rax]

	vunpckhps ymm0, ymm1, ymm2
	vunpckhps ymm8, ymm9, yword [rax]
	vunpckhps ymm9, ymm0, ymm10
