.bits $64
.global _start
_start:
	vcmpsd %xmm0, %xmm1, %xmm2, $1
	vcmpsd %xmm8, %xmm9, %xmm10, $1
