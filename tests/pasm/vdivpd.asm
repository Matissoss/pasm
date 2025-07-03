bits 64
_start:
	vdivpd xmm0, xmm1, xmm2
	vdivpd xmm8, xmm9, xword (rax)
	vdivpd xmm9, xmm0, xmm10

	vdivpd ymm0, ymm1, ymm2
	vdivpd ymm8, ymm9, yword (rax)
	vdivpd ymm9, ymm0, ymm10
