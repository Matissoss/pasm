.bits $64
_start:
	pavgb %xmm0, %xmm1
	pavgb %mm0, %mm1
	pavgw %xmm0, %xmm1
	pavgw %mm0, %mm1
	
	vpavgb %xmm0, %xmm1, %xmm2
	vpavgw %xmm0, %xmm1, %xmm2
	vphaddw %xmm0, %xmm1, %xmm2
	vphaddd %xmm0, %xmm1, %xmm2
	vphsubw %xmm0, %xmm1, %xmm2
	vphsubd %xmm0, %xmm1, %xmm2

	vzeroall
	vzeroupper

	vinsertf128 %ymm0, %ymm1, %xmm0, $10
	vinsertf128 %ymm0, %ymm1, %xmm1, $10
	vextractf128 .xword (%rax), %ymm1, $10
	
	vbroadcastss %xmm0, .dword (%rax)
	vbroadcastss %ymm0, %xmm0
	
	vbroadcastsd %ymm0, .qword (%rax)
	vbroadcastsd %ymm0, %xmm0
	
	vbroadcastf128 %ymm0, .xword (%rax)
