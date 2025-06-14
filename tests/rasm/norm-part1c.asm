.bits $64
_start:
	cmpstrb
	cmpstrw
	cmpstrd
	cmpstrq
	endbr32
	endbr64

	cmpxchg %al, %bl
	cmpxchg %eax, %ebx
	cmpxchg %rax, %rbx

	cldemote .byte (%rax)
	clrssbsy .qword (%rax)
	cmpxchg8b .qword (%rax)
	cmpxchg16b .xword (%rax)
