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

	in %al, %dx
	in %ax, %dx
	in %eax, %dx

	in %al, $10
	in %ax, $10
	in %eax, $10

	lar %ax, %eax
	lar %ax, %bx

	lldt %ax
	lmsw %ax
