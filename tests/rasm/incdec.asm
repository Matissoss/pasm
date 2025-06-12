.bits $64
_start:
	inc %al
	inc (%rax) .byte
	inc %ax
	inc (%rax) .word
	inc %eax
	inc (%rax) .dword
	inc %rax
	inc (%rax) .qword
	inc %r8
	inc %r9
	dec %al
	dec (%rax) .byte
	dec %ax
	dec (%rax) .word
	dec %eax
	dec (%rax) .dword
	dec %rax
	dec (%rax) .qword
	dec %r8
	dec %r9
