section .text
	bits 64
	global _start
_start:
	vextractps eax, xmm1, 10
	vpmaxub xmm1, xmm2, xmm3
	vpmaxub ymm1, ymm2, ymm3
	vpextrb rax, xmm2, 10
	vpextrb byte [rax], xmm2, 10
	vpextrb eax, xmm2, 10
	
	vpextrd eax, xmm2, 10
	vpextrq rax, xmm2, 10
