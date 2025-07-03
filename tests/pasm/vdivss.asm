bits 64
_start:
	vdivss xmm0, xmm1, xmm2
	vdivss xmm8, xmm9, xmm10
	vdivss xmm8, xmm9, dword (rax)
