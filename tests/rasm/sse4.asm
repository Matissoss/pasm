!bits $64
!global _start
_start:
	pinsrq %xmm0, %rax, $1
	pinsrd %xmm1, %r8d, $1
	pinsrb %xmm1, !byte (%rax), $1
	pinsrb %xmm1, %eax, $1

	pextrw !word (%rax), %xmm0, $1

	ptest %xmm0, %xmm1
	ptest %xmm0, !xword (%rax)

	pmaxsb %xmm0, %xmm1
	pmaxsb %xmm0, %xmm1
	
	pmaxsd %xmm0, %xmm1
	pmaxsd %xmm0, %xmm1
	
	pminsb %xmm0, %xmm1
	pminsb %xmm0, %xmm1
	
	pminsd %xmm0, %xmm1
	pminsd %xmm0, %xmm1
	
	pminuw %xmm0, %xmm1
	pminuw %xmm0, %xmm1
	
	pmuldq %xmm0, %xmm1
	
	pmulld %xmm0, %xmm1

	pcmpeqq %xmm0, %xmm1
	pcmpgtq %xmm0, %xmm1
	
	blendvps %xmm0, %xmm1
	blendvpd %xmm0, %xmm1

	packusdw %xmm0, %xmm1

	popcnt %ax, %bx
	popcnt %eax, %ebx
	popcnt %rax, %rbx

	movntdqa %xmm0, !xword (%rax)
	
	extractps %eax, %xmm0, $1
	extractps %rax, %xmm0, $1

	extractps !dword (%rax), %xmm0, $1

	roundss %xmm0, %xmm1, $1
	roundsd %xmm0, %xmm1, $1
	insertps %xmm0, %xmm1, $1
	roundps %xmm0, %xmm1, $1
	roundpd %xmm0, %xmm1, $1

	; Need to fix this
	;pextrb %eax, %xmm1, $1
	;pextrd %eax, %xmm1, $1
	;pextrq %rax, %xmm1, $1

	dpps %xmm0, %xmm1, $1
	dppd %xmm0, %xmm1, $1

	blendps %xmm0, %xmm1, $1
	blendpd %xmm0, %xmm1, $1

	pblendw %xmm0, %xmm1, $2
	mpsadbw %xmm0, %xmm1, $2
	pcmpestri %xmm0, %xmm1, $1
	pcmpestrm %xmm0, %xmm1, $1
	pcmpistri %xmm0, %xmm1, $1
	pcmpistrm %xmm0, %xmm1, $1
