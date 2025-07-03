bits 64
_start:
	vmulpd xmm0, xmm1, xmm2
	vmulpd xmm8, xmm9, xword (rax)
	vmulpd xmm9, xmm0, xmm10
	
	vmulpd ymm0, ymm1, ymm2
	vmulpd ymm8, ymm9, yword (rax)
	vmulpd ymm9, ymm0, ymm10
