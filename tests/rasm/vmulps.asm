!bits $64
!global _start
_start:
	vmulps %xmm0, %xmm1, %xmm2
	vmulps %xmm8, %xmm9, !xword (%rax)
	vmulps %xmm9, %xmm0, %xmm10
	
	vmulps %ymm0, %ymm1, %ymm2
	vmulps %ymm8, %ymm9, !yword (%rax)
	vmulps %ymm9, %ymm0, %ymm10
