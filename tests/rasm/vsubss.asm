!bits $64
!global _start
_start:
	vsubss %xmm0, %xmm1, %xmm2
	vsubss %xmm8, %xmm9, %xmm10
	vsubss %xmm8, %xmm9, !dword (%rax)
