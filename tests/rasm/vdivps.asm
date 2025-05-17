!bits $64
!global _start
_start:
	vdivps %xmm0, %xmm1, %xmm2
	vdivps %xmm8, %xmm9, !xword (%rax)
	vdivps %xmm9, %xmm0, %xmm10

	vdivps %ymm0, %ymm1, %ymm2
	vdivps %ymm8, %ymm9, !yword (%rax)
	vdivps %ymm9, %ymm0, %ymm10
