.bits $64
_start:
	vcvtsi2sd %xmm0, %xmm1, %eax
	vcvtsi2sd %xmm0, %xmm1, %rax
	vcvtsi2ss %xmm0, %xmm1, %eax
	vcvtsi2ss %xmm0, %xmm1, %rax
	
	vcvtss2sd %xmm0, %xmm1, %xmm2
	vcvtdq2ps %xmm0, %xmm1
	vcvttpd2dq %xmm0, %xmm1
	vcvttps2dq %xmm0, %xmm1

	vcvtdq2pd %xmm0, %xmm1
	vcvtdq2pd %xmm0, .qword (%rax)
	vcvtdq2pd %ymm0, %xmm1
	vcvtdq2pd %ymm0, .xword (%rax)

	vcvtsd2ss %xmm0, %xmm1, %xmm2
	vcvttsd2si %eax, %xmm1
	vcvttsd2si %rax, %xmm1

	vcvtss2si %eax, %xmm0
	vcvtss2si %rax, %xmm0

	vcvttss2si %eax, %xmm0
	vcvttss2si %rax, %xmm0

	vcvtsd2si %eax, %xmm1
	vcvtsd2si %rax, %xmm1

	vcvtpd2dq %xmm0, %xmm1
	vcvtpd2ps %xmm0, %xmm1
	vcvtps2dq %xmm0, %xmm1
	vcvtps2pd %xmm0, %xmm1
