!bits $64
!global _start
_start:
	vdivsd %xmm0, %xmm1, %xmm2
	vdivsd %xmm8, %xmm9, %xmm10
	vdivsd %xmm8, %xmm9, !qword (%rax)
