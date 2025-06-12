.bits $64
.global _start
_start:
	vsubpd %xmm0, %xmm1, %xmm2
	vsubpd %xmm8, %xmm9, .xword (%rax)
	vsubpd %xmm9, %xmm0, %xmm10
	
	vsubpd %ymm0, %ymm1, %ymm2
	vsubpd %ymm8, %ymm9, .yword (%rax)
	vsubpd %ymm9, %ymm0, %ymm10
