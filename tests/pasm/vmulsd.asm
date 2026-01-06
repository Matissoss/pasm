bits 64
_start:
	vmulsd xmm0, xmm1, xmm2
	vmulsd xmm8, xmm9, xmm10
	vmulsd xmm8, xmm9, qword [rax]
