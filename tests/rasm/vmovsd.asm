.bits $64
.global _start
_start:
	vmovsd %xmm0, %xmm1, %xmm2
	vmovsd %xmm0, .qword (%rax)
	vmovsd .qword (%rax), %xmm0
