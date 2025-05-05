section .text
	bits 64
	global _start
_start:
	paddb xmm0, oword [rax]
	paddw xmm1, xmm2
	paddd xmm3, xmm4
	paddq xmm5, xmm6
	paddsb xmm0, oword [rax]
	paddsw xmm1, xmm2
	paddsb xmm3, xmm4
	paddsw xmm5, xmm6
	
	psubb xmm0, oword [rax]
	psubw xmm1, xmm2
	psubd xmm3, xmm4
	
	psubsb xmm0, oword [rax]
	psubsw xmm1, xmm2
	psubsb xmm3, xmm4
	psubsw xmm5, xmm6
	
	pmullw xmm0, oword [rax]
	pmullw xmm1, xmm2
	pmulhw xmm0, oword [rax]
	pmulhw xmm1, xmm2

	pmaddwd xmm0, oword [rax]
	pmaddwd xmm0, xmm1

	pcmpeqb xmm0, oword [rax]
	pcmpeqb xmm0, xmm1
	
	pcmpeqw xmm0, oword [rax]
	pcmpeqw xmm0, xmm1
	
	pcmpeqd xmm0, oword [rax]
	pcmpeqd xmm0, xmm1
	
	pcmpgtb xmm0, oword [rax]
	pcmpgtb xmm0, xmm1
	
	pcmpgtw xmm0, oword [rax]
	pcmpgtw xmm0, xmm1
	
	pcmpgtd xmm0, oword [rax]
	pcmpgtd xmm0, xmm1

	packssdw xmm0, oword [rax]
	packssdw xmm0, xmm5
	
	packsswb xmm0, oword [rax]
	packsswb xmm0, xmm5
	
	packuswb xmm0, oword [rax]
	packuswb xmm0, xmm5

	punpcklbw xmm0, oword [rax]
	punpcklbw xmm0, xmm1
	
	punpcklwd xmm0, oword [rax]
	punpcklwd xmm0, xmm1
	
	punpckldq xmm0, oword [rax]
	punpckldq xmm0, xmm1

	punpckhbw xmm0, oword [rax]
	punpckhbw xmm0, xmm1
	
	punpckhwd xmm0, oword [rax]
	punpckhwd xmm0, xmm1
	
	punpckhdq xmm0, oword [rax]
	punpckhdq xmm0, xmm1

	por xmm0, oword [rax]
	por xmm0, xmm1
	
	pxor xmm0, oword [rax]
	pxor xmm0, xmm1
	
	pand xmm0, oword [rax]
	pand xmm0, xmm1
	
	pandn xmm0, oword [rax]
	pandn xmm0, xmm1

	psllw xmm0, 1
	psllw xmm0, xmm1
	psllw xmm1, oword [rax]
	
	pslld xmm0, 1
	pslld xmm0, xmm1
	pslld xmm1, oword [rax]
	
	psllq xmm0, 1
	psllq xmm0, xmm1
	psllq xmm1, oword [rax]
	
	psrlw xmm0, 1
	psrlw xmm0, xmm1
	psrlw xmm1, oword [rax]
	
	psrld xmm0, 1
	psrld xmm0, xmm1
	psrld xmm1, oword [rax]
	
	psrlq xmm0, 1
	psrlq xmm0, xmm1
	psrlq xmm1, oword [rax]
	
	psraw xmm0, 1
	psraw xmm0, xmm1
	psraw xmm1, oword [rax]
	
	psrad xmm0, 1
	psrad xmm0, xmm1
	psrad xmm1, oword [rax]

	emms
