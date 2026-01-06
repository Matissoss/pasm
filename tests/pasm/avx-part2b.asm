bits 64
_start:
	stmxcsr dword [rax]
	vstmxcsr dword [rax]
	ldmxcsr dword [rax]
	vldmxcsr dword [rax]

	vmovmskps eax, xmm0
	vmovmskps rax, xmm0

	vpermilpd xmm0, xmm1, 10
	vpermilpd xmm0, xmm1, xmm2
	
	vpermilps xmm0, xmm1, 10
	vpermilps xmm0, xmm1, xmm2

	vpclmulqdq xmm0, xmm1, xmm2, 10
	pclmulqdq xmm0, xmm1, 10

	vperm2f128 ymm0, ymm1, ymm2, 10
	vperm2i128 ymm0, ymm1, ymm2, 10
