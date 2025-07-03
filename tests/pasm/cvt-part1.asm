bits 64
_start:
	cvtpd2pi mm0, xmm9
	cvtss2si eax, xmm9
	cvtss2si rax, xmm9
	cvttpd2pi mm0, xmm9
	cvtps2pi mm0, xmm9
	cvttps2pi mm0, xmm9
	cvtpi2ps xmm9, mm0
	cvtpi2pd xmm9, mm0
	cvtsi2ss xmm9, eax
	cvtsi2ss xmm9, rax
	cvtsi2ss xmm9, rax

	cvtpd2dq xmm0, xmm9
	cvtss2sd xmm0, xmm9
	cvtdq2pd xmm0, xmm9
	cvtps2pd xmm0, xmm9
	cvtsd2ss xmm9, xmm0
	cvttss2si eax, xmm9
	cvttss2si rax, xmm9

	cvtsd2si eax, xmm9
	cvtsd2si rax, xmm9

	cvtsi2sd xmm9, eax
	cvtsi2sd xmm9, rax
