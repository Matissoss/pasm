#(bits=64)
_start:
	eaddph:k1:z %xmm0, %xmm1, %xmm2
	eaddph:k2 %ymm0, %ymm1, %ymm2
	eaddph:k3:er %zmm0, %zmm1, %zmm2
	eaddph:k3:er %zmm0, %zmm1, .zword (%rax)

	eaddsh:k3:er %xmm0, %xmm1, .word (%rax)
	eaddsh:k3:er %xmm0, %xmm1, %xmm2
	
	eaddsh:k3:er %xmm31, %xmm14, .word (%rax)
	eaddsh:k3:er %xmm23, %xmm15, %xmm16
