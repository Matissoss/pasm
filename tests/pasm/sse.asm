bits 64
_start:
	cvtpi2ps xmm0, mm0
	cvtpi2ps xmm0, qword [rax]
	
	cvtps2pi mm0, xmm0
	cvtps2pi mm0, qword [rax]
	
	cvttps2pi mm0, xmm0
	cvttps2pi mm0, qword [rax]
	
	cvtsi2ss xmm0, dword [rax]
	cvtsi2ss xmm0, qword [rax]

	cvtss2si eax, xmm0
	cvtss2si eax, dword [rax]
	
	cvtss2si rax, xmm0
	cvtss2si rax, dword [rax]

	shufps xmm0, xmm1, 8
	shufps xmm0, xword [rax], 8
	
	movlhps xmm0, xmm1
	movhlps xmm0, xmm1
	
	movss xmm0, xmm1
	movss xmm0, dword [rax]
	movss dword [rax], xmm0
	
	movaps xmm0, xmm1
	movaps xmm0, xword [rax]
	movaps xword [rax], xmm0
	
	movups xmm0, xmm1
	movups xmm0, xword [rax]
	movups xword [rax], xmm0
	
	movlps xmm0, qword [rax]
	movlps qword [rax], xmm0
	
	movhps xmm0, qword [rax]
	movhps qword [rax], xmm0
	
	addps xmm0, xmm1
	addps xmm0, xword [rax]
	
	addss xmm0, xmm1
	addss xmm0, dword [rax]
	
	subps xmm0, xmm1
	subps xmm0, xword [rax]
	
	subss xmm0, xmm1
	subss xmm0, dword [rax]
	
	divps xmm0, xmm1
	divps xmm0, xword [rax]
	
	divss xmm0, xmm1
	divss xmm0, dword [rax]
	
	mulps xmm0, xmm1
	mulps xmm0, xword [rax]
	
	mulss xmm0, xmm1
	mulss xmm0, dword [rax]
	
	rcpps xmm0, xmm1
	rcpps xmm0, xword [rax]
	
	rcpss xmm0, xmm1
	rcpss xmm0, dword [rax]
	
	minps xmm0, xmm1
	minps xmm0, xword [rax]
	
	minss xmm0, xmm1
	minss xmm0, dword [rax]
	
	maxps xmm0, xmm1
	maxps xmm0, xword [rax]
	
	maxss xmm0, xmm1
	maxss xmm0, dword [rax]
	
	cmpps xmm0, xmm1, 1
	cmpps xmm0, xword [rax], 1
	
	cmpss xmm0, xmm1, 1
	cmpss xmm0, dword [rax], 1
	
	sqrtps xmm0, xmm1
	sqrtps xmm0, xword [rax]

	sqrtss xmm0, xmm1
	sqrtss xmm0, dword [rax]
	
	rsqrtps xmm0, xmm1
	rsqrtps xmm0, xword [rax]

	rsqrtss xmm0, xmm1
	rsqrtss xmm0, dword [rax]
	
	; sse2
	addpd xmm0, xmm1
	addpd xmm0, xword [rax]
	
	addsd xmm0, xmm1
	addsd xmm0, qword [rax]
	
	subpd xmm0, xmm1
	subpd xmm0, xword [rax]
	
	subsd xmm0, xmm1
	subsd xmm0, qword [rax]
	
	divpd xmm0, xmm1
	divpd xmm0, xword [rax]
	
	divsd xmm0, xmm1
	divsd xmm0, qword [rax]
	
	mulpd xmm0, xmm1
	mulpd xmm0, xword [rax]
	
	mulsd xmm0, xmm1
	mulsd xmm0, qword [rax]
	
	minpd xmm0, xmm1
	minpd xmm0, xword [rax]
	
	minsd xmm0, xmm1
	minsd xmm0, qword [rax]
	
	maxpd xmm0, xmm1
	maxpd xmm0, xword [rax]
	
	maxsd xmm0, xmm1
	maxsd xmm0, qword [rax]
	
	cmppd xmm0, xmm1, 1
	cmppd xmm0, xword [rax], 1
	
	cmpsd xmm0, xmm1, 1
	cmpsd xmm0, qword [rax], 1

	sqrtpd xmm0, xmm1
	sqrtpd xmm0, xword [rax]

	sqrtsd xmm0, xmm1
	sqrtsd xmm0, qword [rax]
	
	movapd xmm0, xmm1
	movapd xword [rax], xmm0
	movapd xmm1, xword [rax]
	
	movupd xmm0, xmm1
	movupd xword [rax], xmm0
	movupd xmm1, xword [rax]
	
	movlpd qword [rax], xmm0
	movlpd xmm1, qword [rax]
	
	movhpd qword [rax], xmm0
	movhpd xmm1, qword [rax]
	
	movsd qword [rax], xmm0
	movsd xmm1, qword [rax]

	movmskpd rax, xmm1
	movmskpd eax, xmm1

	movdq2q mm0, xmm0
	movdqa  xmm0, xmm1
	movdqa  xmm0, xword [rax]
	movdqa  xword [rax], xmm0
