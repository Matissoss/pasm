.bits $64
_start:
	vminpd %xmm0, %xmm1, %xmm2
	vminpd %xmm8, %xmm9, %xmm10
	vminpd %xmm8, %xmm9, .xword (%rax)

	vminpd %ymm0, %ymm1, %ymm2
	vminpd %ymm8, %ymm9, .yword (%rax)
	vminpd %ymm9, %ymm0, %ymm10
	
	vmaxpd %xmm0, %xmm1, %xmm2
	vmaxpd %xmm8, %xmm9, %xmm10
	vmaxpd %xmm8, %xmm9, .xword (%rax)

	vmaxpd %ymm0, %ymm1, %ymm2
	vmaxpd %ymm8, %ymm9, .yword (%rax)
	vmaxpd %ymm9, %ymm0, %ymm10
