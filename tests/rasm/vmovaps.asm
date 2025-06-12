.bits $64
.global _start
_start:
	vmovaps %xmm0, %xmm1
	vmovaps .xword (%rax), %xmm1
	vmovaps %xmm0, .xword (%rax)
	
	vmovaps %ymm0, %ymm1
	vmovaps .yword (%rax), %ymm1
	vmovaps %ymm0, .yword (%rax)

	vmovaps %xmm8, %xmm9
