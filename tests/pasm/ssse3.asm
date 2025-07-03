bits 64
_start:
	pabsb xmm0, xmm1
	pabsb mm0, mm1
	pabsw xmm0, xmm1
	pabsw mm0, mm1
	pabsd xmm0, xmm1
	pabsd mm0, mm1
	psignb xmm0, xmm1
	psignb mm0, mm1
	psignw xmm0, xmm1
	psignw mm0, mm1
	psignd xmm0, xmm1
	psignd mm0, mm1
	phsubw xmm0, xmm1
	phsubw mm0, mm1
	phsubd xmm0, xmm1
	phsubd mm0, mm1
	phaddw xmm0, xmm1
	phaddw mm0, mm1
	phaddd xmm0, xmm1
	phaddd mm0, mm1
	pshufb xmm0, xmm1
	pshufb mm0, mm1
	phaddsw xmm0, xmm1
	phaddsw mm0, mm1
	phsubsw xmm0, xmm1
	phsubsw mm0, mm1
	palignr xmm0, xmm1, 1
	palignr mm0, mm1, 1

	pmulhrsw xmm0, xmm1
	pmulhrsw mm0, mm1
	pmaddubsw xmm0, xmm1
	pmaddubsw mm0, mm1
