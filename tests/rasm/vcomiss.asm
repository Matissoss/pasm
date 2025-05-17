!bits $64
!global _start
_start:
	vcomiss %xmm1, %xmm2
	vcomiss %xmm9, %xmm10
	vcomiss %xmm9, !dword (%rax)
	
	vucomiss %xmm1, %xmm2
	vucomiss %xmm9, %xmm10
	vucomiss %xmm9, !dword (%rax)
