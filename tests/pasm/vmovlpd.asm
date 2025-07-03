bits 64
_start:
	vmovlpd qword (rax), xmm1
	vmovlpd xmm0, xmm1, qword (rax)
	
	vmovhpd qword (rax), xmm1
	vmovhpd xmm0, xmm1, qword (rax)
