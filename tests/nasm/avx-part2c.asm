section .text
	bits 64
	global _start
_start:
	vpmaxsw xmm0, xmm1, oword [rax]
	vpminsw xmm0, xmm1, oword [rax]
	vpsignw xmm0, xmm1, oword [rax]
	vpsignb xmm0, xmm1, oword [rax]
	vpsignd xmm0, xmm1, oword [rax]
	vpmuludq xmm0, xmm1, oword [rax]
	vpmulhuw xmm0, xmm1, oword [rax]
	vpmulhrsw xmm0, xmm1, oword [rax]

	vpsrldq xmm0, xmm1, 10
	vpinsrw xmm0, xmm1, eax, 10
	vpinsrw xmm0, xmm1, word [rax], 10
