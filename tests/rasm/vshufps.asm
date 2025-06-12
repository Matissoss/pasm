.bits $64
.global _start
_start:
	vshufps %xmm0, %xmm1, %xmm2, $1
	vshufps %xmm8, %xmm9, %xmm10, $1
	vshufps %xmm8, %xmm9, .xword (%rax), $1

	vshufps %ymm0, %ymm1, %ymm2, $1
	vshufps %ymm8, %ymm9, .yword (%rax), $1
	vshufps %ymm9, %ymm0, %ymm10, $1
