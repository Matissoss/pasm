bits 64
_start:
	vsqrtpd xmm0, xmm1
	vsqrtpd xmm0, xword [rax]

	vsqrtpd ymm0, ymm1
	vsqrtpd ymm0, yword [rax]
