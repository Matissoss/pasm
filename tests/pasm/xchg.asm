.bits $64
_start:
	xchg %rbx, %rax
	xchg %r8, %r9
	xchg (%rcx*$4) .qword, %rax
	xchg (%rax) .qword, %rax
	xchg (%rax + %rcx) .qword, %rax
	xchg (%rax + %rcx*$4) .qword, %rax
	xchg (%rcx*$4) .qword, %rax
	xchg (%rax+%rcx*$4+$20) .qword, %r8
	xchg (%rax+%rcx*$4) .qword, %r9
	xchg (%rax+%r8*$4+$20) .qword, %r10
	xchg (%r9+%r8*$4+$20) .qword, %r11
