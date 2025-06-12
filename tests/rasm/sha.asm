.bits $64
.global _start
_start:
	sha1msg1 %xmm8, %xmm9
	sha1msg2 %xmm8, %xmm9
	sha1nexte %xmm8, %xmm9
	sha1rnds4 %xmm8, %xmm9, $10
	sha256msg1 %xmm8, %xmm9
	sha256msg2 %xmm8, %xmm9
	sha256rnds2 %xmm8, %xmm9
