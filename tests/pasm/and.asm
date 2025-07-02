.bits $64
_start:
	and %rax, $10
	and %rax, $256
	and %rax, $65537
	and %rbx, %rax
	and %r8, %r9
	and %r8, $0
	and (%rax) .qword, $1000
	and (%rax + %rcx) .qword, $10
	and (%rax + %rcx*$4) .qword, $10
	and (%rcx*$4) .qword, %rax
	and (%rax+%rcx*$4+$20) .qword, $10
	and (%rax+%rcx*$4) .qword, $10
	and (%rax+%r8*$4+$20) .qword, $10
	and (%r9+%r8*$4+$20) .qword, $10
	and (%rax) .qword, %rax
	and (%rax + %rcx) .qword, %rax
	and (%rax + %rcx*$4) .qword, %rax
	and (%rcx*$4) .qword, %rax
	and (%rax+%rcx*$4+$20) .qword, %r8
	and (%rax+%rcx*$4) .qword, %r9
	and (%rax+%r8*$4+$20) .qword, %r10
	and (%r9+%r8*$4+$20) .qword, %r11
