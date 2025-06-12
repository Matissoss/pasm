.bits $64
.global _start
_start:
	vaddsd %xmm0, %xmm1, %xmm2
	vaddsd %xmm8, %xmm9, %xmm10
	vaddsd %xmm8, %xmm9, .qword (%rax)
