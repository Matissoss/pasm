bits 64
_start:
	addsubpd xmm0, xmm1
	addsubps xmm0, xmm1
	haddpd xmm0, xmm1
	haddps xmm0, xmm1
	hsubpd xmm0, xmm1
	hsubps xmm0, xmm1

	movsldup xmm0, xmm1
	movshdup xmm0, xmm1
	movddup xmm0, xmm1

	lddqu xmm0, xword (rax)
	
	monitor
	mwait
