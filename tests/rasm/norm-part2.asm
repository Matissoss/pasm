.bits $64
_start:
	invpcid %rax, .xword (%rax)
	iret
	iretd
	iretq
	lodsq
	enter $10, $1
	enter $10, $0
	enter $10, $11
	hlt
	insb
	insw
	insd
	int3
	int1
	lahf
	leave
	lodsb
	lodsw
	lodsd
	hreset $10

	indxb
	indxw
	indxd

	inportb $10
	inportw $10
	inportd $10

	lar %ax, %eax
	lar %ax, %bx

	lldt %ax
	lmsw %ax
