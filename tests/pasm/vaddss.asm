bits 64
_start:
	vaddss xmm0, xmm1, xmm2
	vaddss xmm8, xmm9, xmm10
	vaddss xmm8, xmm9, dword (rax)
