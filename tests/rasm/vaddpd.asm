.bits $64
.global _start
_start:
	vaddpd %xmm0, %xmm1, %xmm2
	vaddpd %xmm8, %xmm9, %xmm10
	vaddpd %xmm8, %xmm9, .xword (%rax)

	vaddpd %ymm0, %ymm1, %ymm2
	vaddpd %ymm8, %ymm9, .yword (%rax)
	vaddpd %ymm9, %ymm0, %ymm10
