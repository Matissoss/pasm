[bits 64]
_start:
	vaddph xmm0{k1}{z}, xmm1, xmm2
	vaddph ymm0{k2}, ymm1, ymm2
	vaddph zmm0{k3}, zmm1, zmm2
	vaddph zmm0{k3}, zmm1, [rax]
	
	vaddsh xmm0 {k3}, xmm1, word [rax]
	vaddsh xmm0 {k3}, xmm1, xmm2
	
	vaddsh xmm31 {k3}, xmm14, word [rax]
	vaddsh xmm23 {k3}, xmm15, xmm16

	valignd xmm20 {k3}, xmm21, xmm22, 10
	valignd ymm20 {k6}, ymm21, ymm22, 10
	valignd zmm20 {k5}, zmm21, [rax]{1to16}, 10
	
	valignq xmm20 {k3}, xmm21, xmm22, 10
	valignq ymm20 {k6}, ymm21, ymm22, 10
	valignq zmm20 {k5}, zmm21, [rax]{1to8}, 10

	;vbcstnebf162ps xmm20, [rax]
	;vbcstnebf162ps ymm20, [rax]
	;vbcstnesh2ps xmm20, [rax]
	;vbcstnesh2ps ymm20, [rax]
	vblendmps xmm20 {k3}, xmm21, xmm22
	vblendmps ymm20 {k6}, ymm21, ymm22
	vblendmps zmm20 {k5}, zmm21, [rax]{1to16}
	
	vblendmpd xmm20 {k3}, xmm21, xmm22
	vblendmpd ymm20 {k6}, ymm21, ymm22
	vblendmpd zmm20 {k5}, zmm21, [rax]{1to8}

	vbroadcastsd ymm20 {k2}, xmm20
	vbroadcastsd zmm20 {k2}, qword [rax]
	
	vbroadcastf32x2 ymm20 {k2}, xmm20
	vbroadcastf32x2 zmm20 {k2}, qword [rax]
	
	vbroadcastss ymm20 {k2}, xmm20
	vbroadcastss zmm20 {k2}, dword [rax]
	
	vbroadcastf64x2 zmm20 {k2}, oword [rax]
	vbroadcastf32x4 zmm20 {k2}, oword [rax]
	
	vbroadcastf64x4 zmm20 {k2}, yword [rax]
	vbroadcastf32x8 zmm20 {k2}, yword [rax]

	vcmpph k1 {k2}, zmm20, zmm21, {sae}, 10
	vcmpsh k1 {k2}, xmm20, xmm21, {sae}, 10
	vcomish xmm20, xmm21, {sae}

	vcompresspd xmm20 {k1}, xmm21
	vcompresspd ymm20 {k1}, ymm21
	vcompresspd zmm20 {k1}, zmm21
	
	vcompressps xmm20 {k1}, xmm21
	vcompressps ymm20 {k1}, ymm21
	vcompressps zmm20 {k1}, zmm21

	vfmadd213ps xmm20 {k1}, xmm21, xmm22
