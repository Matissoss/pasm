.bits $64
_start:
	add %rax, $10
	add %rax, $256
	add %rax, $65537
	add %rbx, %rax
	add %r8, %r9
	add %r8, $0
	add (%rax) .qword, $1000
	add (%rax + %rcx) .qword, $10
	add (%rax + %rcx*$4) .qword, $10
	add (%rcx*$4) .qword, %rax
	add (%rax+%rcx*$4+$20) .qword, $10
	add (%rax+%rcx*$4) .qword, $10
	add (%rax+%r8*$4+$20) .qword, $10
	add (%r9+%r8*$4+$20) .qword, $10
	add (%rax) .qword, %rax
	add (%rax + %rcx) .qword, %rax
	add (%rax + %rcx*$4) .qword, %rax
	add (%rcx*$4) .qword, %rax
	add (%rax+%rcx*$4+$20) .qword, %r8
	add (%rax+%rcx*$4) .qword, %r9
	add (%rax+%r8*$4+$20) .qword, %r10
	add (%r9+%r8*$4+$20) .qword, %r11
