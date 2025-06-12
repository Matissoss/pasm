.bits $64
.global _start
_start:
	paddb %mm0, .qword (%rax)
	paddw %mm1, %mm2
	paddd %mm3, %mm4
	paddq %mm5, %mm6
	paddsb %mm0, .qword (%rax)
	paddsw %mm1, %mm2
	paddsb %mm3, %mm4
	paddsw %mm5, %mm6
	
	psubb %mm0, .qword (%rax)
	psubw %mm1, %mm2
	psubd %mm3, %mm4
	
	psubsb %mm0, .qword (%rax)
	psubsw %mm1, %mm2
	psubsb %mm3, %mm4
	psubsw %mm5, %mm6
	
	pmullw %mm0, .qword (%rax)
	pmullw %mm1, %mm2
	pmulhw %mm0, .qword (%rax)
	pmulhw %mm1, %mm2
	
	pmaddwd %mm0, .qword (%rax)
	pmaddwd %mm0, %mm1
	
	pcmpeqb %mm0, .qword (%rax)
	pcmpeqb %mm0, %mm1
	
	pcmpeqw %mm0, .qword (%rax)
	pcmpeqw %mm0, %mm1
	
	pcmpeqd %mm0, .qword (%rax)
	pcmpeqd %mm0, %mm1
	
	pcmpgtb %mm0, .qword (%rax)
	pcmpgtb %mm0, %mm1
	
	pcmpgtw %mm0, .qword (%rax)
	pcmpgtw %mm0, %mm1
	
	pcmpgtd %mm0, .qword (%rax)
	pcmpgtd %mm0, %mm1
	
	packssdw %mm0, .qword (%rax)
	packssdw %mm0, %mm5
	
	packsswb %mm0, .qword (%rax)
	packsswb %mm0, %mm5
	
	packuswb %mm0, .qword (%rax)
	packuswb %mm0, %mm5
	
	punpcklbw %mm0, .qword (%rax)
	punpcklbw %mm0, %mm1
	
	punpcklwd %mm0, .qword (%rax)
	punpcklwd %mm0, %mm1
	
	punpckldq %mm0, .qword (%rax)
	punpckldq %mm0, %mm1
	
	punpckhbw %mm0, .qword (%rax)
	punpckhbw %mm0, %mm1
	
	punpckhwd %mm0, .qword (%rax)
	punpckhwd %mm0, %mm1
	
	punpckhdq %mm0, .qword (%rax)
	punpckhdq %mm0, %mm1
	
	por %mm0, .qword (%rax)
	por %mm0, %mm1
	
	pxor %mm0, .qword (%rax)
	pxor %mm0, %mm1
	
	pand %mm0, .qword (%rax)
	pand %mm0, %mm1
	
	pandn %mm0, .qword (%rax)
	pandn %mm0, %mm1
	
	psllw %mm0, $1
	psllw %mm0, %mm1
	psllw %mm1, .qword (%rax)
	
	pslld %mm0, $1
	pslld %mm0, %mm1
	pslld %mm1, .qword (%rax)
	
	psllq %mm0, $1
	psllq %mm0, %mm1
	psllq %mm1, .qword (%rax)
	
	psrlw %mm0, $1
	psrlw %mm0, %mm1
	psrlw %mm1, .qword (%rax)
	
	psrld %mm0, $1
	psrld %mm0, %mm1
	psrld %mm1, .qword (%rax)
	
	psrlq %mm0, $1
	psrlq %mm0, %mm1
	psrlq %mm1, .qword (%rax)
	
	psraw %mm0, $1
	psraw %mm0, %mm1
	psraw %mm1, .qword (%rax)
	
	psrad %mm0, $1
	psrad %mm0, %mm1
	psrad %mm1, .qword (%rax)

	emms
