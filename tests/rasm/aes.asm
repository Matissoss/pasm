.bits $64
_start:
	aesdec %xmm0, %xmm1
	aesenc %xmm0, %xmm1
	aesimc %xmm0, %xmm1
	vaesimc %xmm0, %xmm1
	aesdeclast %xmm0, %xmm1

	vaesdec %xmm0, %xmm1, %xmm2
	vaesenc %xmm0, %xmm1, %xmm2
	vaesdeclast %xmm0, %xmm1, %xmm2

	aeskeygenassist %xmm0, %xmm1, $10
	vaeskeygenassist %xmm0, %xmm1, $10
