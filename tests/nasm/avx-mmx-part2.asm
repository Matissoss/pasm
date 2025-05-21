section .text
	bits 64
	global _start
_start:
	vpsubusb xmm0, xmm1, xmm2
	vpsubusw xmm0, xmm1, xmm2
	vpaddusb xmm0, xmm1, xmm2
	vpaddusw xmm0, xmm1, xmm2
	vpmaddwd xmm0, xmm1, xmm2
	vpcmpeqb xmm0, xmm1, xmm2
	vpcmpeqw xmm0, xmm1, xmm2
	vpcmpeqd xmm0, xmm1, xmm2
	vpcmpgtb xmm0, xmm1, xmm2
	vpcmpgtw xmm0, xmm1, xmm2
	vpcmpgtd xmm0, xmm1, xmm2
	vpackuswb xmm0, xmm1, xmm2
	vpacksswb xmm0, xmm1, xmm2
	vpackssdw xmm0, xmm1, xmm2
	vpunpcklbw xmm0, xmm1, xmm2
	vpunpcklwd xmm0, xmm1, xmm2
	vpunpckldq xmm0, xmm1, xmm2
	vpunpckhbw xmm0, xmm1, xmm2
	vpunpckhwd xmm0, xmm1, xmm2
	vpunpckhdq xmm0, xmm1, xmm2
