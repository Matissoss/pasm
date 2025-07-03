bits 64
_start:
	vorpd xmm0, xmm1, xmm2
	vorpd xmm8, xmm9, xmm10
	vorpd xmm8, xmm9, xword (rax)

	vorpd ymm0, ymm1, ymm2
	vorpd ymm8, ymm9, yword (rax)
	vorpd ymm9, ymm0, ymm10
	
	vandpd xmm0, xmm1, xmm2
	vandpd xmm8, xmm9, xmm10
	vandpd xmm8, xmm9, xword (rax)

	vandpd ymm0, ymm1, ymm2
	vandpd ymm8, ymm9, yword (rax)
	vandpd ymm9, ymm0, ymm10
	
	vandnpd xmm0, xmm1, xmm2
	vandnpd xmm8, xmm9, xmm10
	vandnpd xmm8, xmm9, xword (rax)

	vandnpd ymm0, ymm1, ymm2
	vandnpd ymm8, ymm9, yword (rax)
	vandnpd ymm9, ymm0, ymm10
	
	vxorpd xmm0, xmm1, xmm2
	vxorpd xmm8, xmm9, xmm10
	vxorpd xmm8, xmm9, xword (rax)

	vxorpd ymm0, ymm1, ymm2
	vxorpd ymm8, ymm9, yword (rax)
	vxorpd ymm9, ymm0, ymm10
