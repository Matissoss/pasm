section .text
	bits 64
	global _start
_start:
	vdpps xmm0, xmm1, xmm2, 10
	vdpps ymm0, ymm1, ymm2, 10
	
	vdppd xmm0, xmm1, xmm2, 10

	vptest xmm0, oword [rax]
	vptest ymm0, yword [rax]

	;vpextrb rax, xmm2, 10
	;vpextrb eax, xmm2, 10
	
	;vpextrd eax, xmm2, 10
	;vpextrq rax, xmm2, 10

	vpinsrb xmm1, xmm2, byte [rax], 10
	vpinsrb xmm1, xmm2, eax, 10
	
	vpinsrd xmm1, xmm2, eax, 10
	vpinsrq xmm1, xmm2, rax, 10

	vpmaxsb xmm1, xmm2, xmm3
	vpmaxsb ymm1, ymm2, ymm3
	vpmaxsd xmm1, xmm2, xmm3
	vpmaxsd xmm1, xmm2, xmm3
	
	vpmaxuw xmm1, xmm2, xmm3
	vpmaxuw ymm1, ymm2, ymm3
	;vpmaxub xmm1, xmm2, xmm3
	;vpmaxub ymm1, ymm2, ymm3
	
	vpminsb xmm1, xmm2, xmm3
	vpminsb ymm1, ymm2, ymm3
	vpminsd xmm1, xmm2, xmm3
	vpminsd xmm1, xmm2, xmm3
	
	vpminuw xmm1, xmm2, xmm3
	vpminuw ymm1, ymm2, ymm3
	vpminub xmm1, xmm2, xmm3
	vpminub ymm1, ymm2, ymm3

	vpmuldq xmm1, xmm2, xmm3
	vpmuldq ymm1, ymm2, ymm3
	vpmulld xmm1, xmm2, xmm3
	vpmulld ymm1, ymm2, ymm3
	
	vblendps xmm0, xmm1, xmm2, 10
	vblendps ymm0, ymm1, ymm2, 10
	vblendpd xmm0, xmm1, xmm2, 10
	vblendpd ymm0, ymm1, ymm2, 10
	
	vblendvps xmm0, xmm1, xmm2, xmm0
	vblendvps ymm0, ymm1, ymm2, ymm0
	vblendvpd xmm0, xmm1, xmm2, xmm15
	vblendvpd ymm0, ymm1, ymm2, ymm15
