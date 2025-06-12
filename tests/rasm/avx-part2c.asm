.bits $64
.global _start
_start:
	vpmaxsw %xmm0, %xmm1, .xword (%rax)
	vpminsw %xmm0, %xmm1, .xword (%rax)
	vpsignw %xmm0, %xmm1, .xword (%rax)
	vpsignb %xmm0, %xmm1, .xword (%rax)
	vpsignd %xmm0, %xmm1, .xword (%rax)
	vpmuludq %xmm0, %xmm1, .xword (%rax)
	vpmulhuw %xmm0, %xmm1, .xword (%rax)
	vpmulhrsw %xmm0, %xmm1, .xword (%rax)

	vpsrldq %xmm0, %xmm1, $10
	vpinsrw %xmm0, %xmm1, %eax, $10
	vpinsrw %xmm0, %xmm1, .word (%rax), $10
