!bits $64
!global _start
_start:
	vmulss %xmm0, %xmm1, %xmm2
	vmulss %xmm8, %xmm9, %xmm10
	vmulss %xmm8, %xmm9, !dword (%rax)
