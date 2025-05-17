section .text
	bits 64
	global _start
_start:
	vmovlps qword [rax], xmm1
	vmovlps xmm0, xmm1, qword [rax]
	
	vmovhps qword [rax], xmm1
	vmovhps xmm0, xmm1, qword [rax]
	
	vmovlhps xmm0, xmm1, xmm2
	vmovhlps xmm0, xmm1, xmm2
