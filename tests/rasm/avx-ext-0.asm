#(bits=64)
_start:
	eaddph:k1:z %xmm0, %xmm1, %xmm2
	eaddph:k2 %ymm0, %ymm1, %ymm2
	eaddph:k3 %zmm0, %zmm1, %zmm2
	eaddph:k3 %zmm0, %zmm1, .zword (%rax)

	eaddsh:k3 %xmm0, %xmm1, .word (%rax)
	eaddsh:k3 %xmm0, %xmm1, %xmm2
	
	eaddsh:k3 %xmm31, %xmm14, .word (%rax)
	eaddsh:k3 %xmm23, %xmm15, %xmm16

	ealignd:k3 %xmm20, %xmm21, %xmm22, $10
	ealignd:k6 %ymm20, %ymm21, %ymm22, $10
	ealignd:k5 %zmm20, %zmm21, .dword:bcst (%rax), $10
	
	ealignq:k3 %xmm20, %xmm21, %xmm22, $10
	ealignq:k6 %ymm20, %ymm21, %ymm22, $10
	ealignq:k5 %zmm20, %zmm21, .qword:bcst (%rax), $10

	; nasm does not want to work here
	;	vbcstnebf162ps %xmm20, .word (%rax)
	;	vbcstnebf162ps %ymm20, .word (%rax)
	;	vbcstnesh2ps %xmm20, .word (%rax)
	;	vbcstnesh2ps %ymm20, .word (%rax)

	eblendmps:k3 %xmm20, %xmm21, %xmm22
	eblendmps:k6 %ymm20, %ymm21, %ymm22
	eblendmps:k5 %zmm20, %zmm21, .dword:bcst (%rax)
	
	eblendmpd:k3 %xmm20, %xmm21, %xmm22
	eblendmpd:k6 %ymm20, %ymm21, %ymm22
	eblendmpd:k5 %zmm20, %zmm21, .qword:bcst (%rax)

	ebroadcastsd:k2 %ymm20, %xmm20
	ebroadcastsd:k2 %zmm20, .qword (%rax)
	
	ebroadcastf32x2:k2 %ymm20, %xmm20
	ebroadcastf32x2:k2 %zmm20, .qword (%rax)
	
	ebroadcastss:k2 %ymm20, %xmm20
	ebroadcastss:k2 %zmm20, .dword (%rax)
	
	ebroadcastf64x2:k2 %zmm20, .xword (%rax)
	ebroadcastf32x4:k2 %zmm20, .xword (%rax)
	
	ebroadcastf64x4:k2 %zmm20, .yword (%rax)
	ebroadcastf32x8:k2 %zmm20, .yword (%rax)

	;ecmpph:k2 %k1, %xmm20, .xword (%rax), $10
	;ecmpph:k2 %k1, %ymm20, .yword (%rax), $10
	ecmpph:k2:sae %k1, %zmm20, %zmm21, $10
	ecmpsh:k2:sae %k1, %xmm20, %xmm21, $10
	ecomish:sae %xmm20, %xmm21
	
	ecompresspd:k1 %xmm20, %xmm21
	ecompresspd:k1 %ymm20, %ymm21
	ecompresspd:k1 %zmm20, %zmm21
	
	ecompressps:k1 %xmm20, %xmm21
	ecompressps:k1 %ymm20, %ymm21
	ecompressps:k1 %zmm20, %zmm21
