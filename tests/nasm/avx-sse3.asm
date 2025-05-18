section .text
	bits 64
	global _start
_start:
	vaddsubps xmm0, xmm1, xmm2
	vaddsubps xmm0, xmm1, oword [rax]
	vaddsubps ymm0, ymm1, ymm2
	vaddsubps ymm0, ymm1, yword [rax]
	
	vaddsubpd xmm0, xmm1, xmm2
	vaddsubpd xmm0, xmm1, oword [rax]
	vaddsubpd ymm0, ymm1, ymm2
	vaddsubpd ymm0, ymm1, yword [rax]
	
	vhaddps xmm0, xmm1, xmm2
	vhaddps xmm0, xmm1, oword [rax]
	vhaddps ymm0, ymm1, ymm2
	vhaddps ymm0, ymm1, yword [rax]
	
	vhaddpd xmm0, xmm1, xmm2
	vhaddpd xmm0, xmm1, oword [rax]
	vhaddpd ymm0, ymm1, ymm2
	vhaddpd ymm0, ymm1, yword [rax]
	
	vhsubps xmm0, xmm1, xmm2
	vhsubps xmm0, xmm1, oword [rax]
	vhsubps ymm0, ymm1, ymm2
	vhsubps ymm0, ymm1, yword [rax]
	
	vhsubpd xmm0, xmm1, xmm2
	vhsubpd xmm0, xmm1, oword [rax]
	vhsubpd ymm0, ymm1, ymm2
	vhsubpd ymm0, ymm1, yword [rax]

	vmovsldup xmm0, xmm2
	vmovsldup xmm0, oword [rax]
	vmovsldup ymm0, ymm2
	vmovsldup ymm0, yword [rax]
	
	vmovshdup xmm0, xmm2
	vmovshdup xmm0, oword [rax]
	vmovshdup ymm0, ymm2
	vmovshdup ymm0, yword [rax]
	
	vmovddup xmm0, xmm2
	vmovddup xmm0, qword [rax]
	vmovddup ymm0, ymm2
	vmovddup ymm0, yword [rax]
	
	vlddqu xmm0, oword [rax]
	vlddqu ymm0, yword [rax]
