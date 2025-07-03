bits 64
_start:
	vcomisd xmm1, xmm2
	vcomisd xmm9, xmm10
	vcomisd xmm9, qword (rax)
	
	vucomisd xmm1, xmm2
	vucomisd xmm9, xmm10
	vucomisd xmm9, qword (rax)
