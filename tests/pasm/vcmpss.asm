bits 64
_start:
	vcmpss xmm0, xmm1, xmm2, 1
	vcmpss xmm8, xmm9, xmm10, 1
	
	; nasm bug :)
	;vcmpss xmm8, xmm9, !dword (rax), 1
