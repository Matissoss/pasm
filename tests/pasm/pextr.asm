bits 64
_start:
	pextrb eax, xmm1, 1
	pextrd eax, xmm1, 1
	pextrq rax, xmm1, 1
	pextrw word (rdi), xmm2, 1
