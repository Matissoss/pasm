.bits $64
.global _start
_start:
	vcmppd %xmm0, %xmm1, %xmm2, $1
	vcmppd %xmm8, %xmm9, %xmm10, $1
	vcmppd %xmm8, %xmm9, .xword (%rax), $1

	vcmppd %ymm0, %ymm1, %ymm2, $1
	vcmppd %ymm8, %ymm9, .yword (%rax), $1
	vcmppd %ymm9, %ymm0, %ymm10, $1
