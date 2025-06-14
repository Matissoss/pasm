.bits $64
_start:
	vsubsd %xmm0, %xmm1, %xmm2
	vsubsd %xmm8, %xmm9, %xmm10
	vsubsd %xmm8, %xmm9, .qword (%rax)
