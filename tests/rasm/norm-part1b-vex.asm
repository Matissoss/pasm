.bits $64
_start:
	andn %eax, %ebx, %ecx
	andn %rax, %rbx, %rcx
	bzhi %eax, %ebx, %ecx
	bzhi %rax, %rbx, %rcx
	bextr %eax, %ebx, %ecx
	bextr %rax, %rbx, %rcx
