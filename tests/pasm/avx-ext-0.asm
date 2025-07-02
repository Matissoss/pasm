#(bits=64)
_start:
	vaddph %xmm0 {k1}{z}, %xmm1, %xmm2
	vaddph %ymm0 {k2}, %ymm1, %ymm2
	vaddph %zmm0 {k3}, %zmm1, %zmm2
	vaddph %zmm0 {k3}, %zmm1, .zword (%rax)

	vaddsh %xmm0 {k3}, %xmm1, .word (%rax)
	vaddsh %xmm0 {k3}, %xmm1, %xmm2
	
	vaddsh {k3} %xmm31, %xmm14, .word (%rax)
	vaddsh {k3} %xmm23, %xmm15, %xmm16

	valignd {k3} %xmm20, %xmm21, %xmm22, $10
	valignd {k6} %ymm20, %ymm21, %ymm22, $10
	valignd {k5} %zmm20, %zmm21, .dword:bcst (%rax), $10
	
	valignq {k3} %xmm20, %xmm21, %xmm22, $10
	valignq {k6} %ymm20, %ymm21, %ymm22, $10
	valignq {k5} %zmm20, %zmm21, .qword:bcst (%rax), $10

	; nasm does not want to work here
	;	vbcstnebf162ps %xmm20, .word (%rax)
	;	vbcstnebf162ps %ymm20, .word (%rax)
	;	vbcstnesh2ps %xmm20, .word (%rax)
	;	vbcstnesh2ps %ymm20, .word (%rax)

	vblendmps {k3} %xmm20, %xmm21, %xmm22
	vblendmps {k6} %ymm20, %ymm21, %ymm22
	vblendmps {k5} %zmm20, %zmm21, .dword:bcst (%rax)
	
	vblendmpd {k3} %xmm20, %xmm21, %xmm22
	vblendmpd {k6} %ymm20, %ymm21, %ymm22
	vblendmpd {k5} %zmm20, %zmm21, .qword:bcst (%rax)

	vbroadcastsd {k2} %ymm20, %xmm20
	vbroadcastsd {k2} %zmm20, .qword (%rax)
	
	vbroadcastf32x2 {k2} %ymm20, %xmm20
	vbroadcastf32x2 {k2} %zmm20, .qword (%rax)
	
	vbroadcastss {k2} %ymm20, %xmm20
	vbroadcastss {k2} %zmm20, .dword (%rax)
	
	vbroadcastf64x2 {k2} %zmm20, .xword (%rax)
	vbroadcastf32x4 {k2} %zmm20, .xword (%rax)
	
	vbroadcastf64x4 {k2} %zmm20, .yword (%rax)
	vbroadcastf32x8 {k2} %zmm20, .yword (%rax)

	vcmpph {sae} %k1 {k2}, %zmm20, %zmm21, $10
	vcmpsh {sae} %k1 {k2}, %xmm20, %xmm21, $10
	vcomish {sae} %xmm20, %xmm21
	
	vcompresspd {k1} %xmm20, %xmm21
	vcompresspd {k1} %ymm20, %ymm21
	vcompresspd {k1} %zmm20, %zmm21
	
	vcompressps {k1} %xmm20, %xmm21
	vcompressps {k1} %ymm20, %ymm21
	vcompressps {k1} %zmm20, %zmm21
	
	vfmadd213ps xmm20 {k1}, xmm21, xmm22
