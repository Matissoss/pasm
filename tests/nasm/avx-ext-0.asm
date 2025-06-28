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
