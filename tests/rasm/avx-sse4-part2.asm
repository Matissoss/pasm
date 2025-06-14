.bits $64
_start:
	vblendps %xmm0, %xmm1, %xmm0, $10
	vblendps %ymm0, %ymm1, %ymm0, $10
	
	vblendpd %xmm0, %xmm1, %xmm0, $10
	vblendpd %ymm0, %ymm1, %ymm0, $10
	
	vpblendw %xmm0, %xmm1, %xmm0, $10
	vpblendw %ymm0, %ymm1, %ymm0, $10

	vpcmpeqq %xmm0, %xmm1, %xmm2
	vpcmpeqq %ymm0, %ymm1, %ymm2
	
	vroundps %xmm0, %xmm1, $10
	vroundps %ymm0, %ymm1, $10
	vroundpd %xmm0, %xmm1, $10
	vroundpd %ymm0, %ymm1, $10
	
	vroundss %xmm2, %xmm0, %xmm1, $10
	vroundss %xmm2, %xmm0, .dword (%rax), $10
	vroundsd %xmm2, %xmm0, %xmm1, $10
	vroundsd %xmm2, %xmm0, .qword (%rax), $10

	vmpsadbw %xmm0, %xmm1, %xmm2, $10
	vmpsadbw %ymm0, %ymm1, %ymm2, $10
	
	vpcmpgtq %xmm0, %xmm1, %xmm2
	vpcmpgtq %ymm0, %ymm1, %ymm2
	
	vblendvps %xmm0, %xmm1, %xmm2, %xmm2
	vblendvps %ymm0, %ymm1, %ymm2, %ymm2
	vblendvpd %xmm0, %xmm1, %xmm2, %xmm2
	vblendvpd %ymm0, %ymm1, %ymm2, %ymm2
	
	vpblendvb %xmm0, %xmm1, %xmm2, %xmm3
	vpblendvb %ymm0, %ymm1, %ymm2, %ymm3
	vpblendvb %xmm0, %xmm1, %xmm2, %xmm3
	vpblendvb %ymm0, %ymm1, %ymm2, %ymm3

	vinsertps %xmm0, %xmm1, %xmm2, $10

	vpackusdw %xmm0, %xmm1, %xmm2
	vpackusdw %ymm0, %ymm1, %ymm2

	vmovntdqa %xmm0, .xword (%rax)
	vmovntdqa %ymm0, .yword (%rax)

	vpcmpestri %xmm0, %xmm1, $10
	vpcmpestrm %xmm0, %xmm1, $10
	vpcmpistri %xmm0, %xmm1, $10
	vpcmpistrm %xmm0, %xmm1, $10
	
	vphminposuw %xmm0, %xmm1
