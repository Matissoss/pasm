.bits $64
_start:
	sbb %rax, $10
	sbb %rax, $256
	sbb %rax, $65537
	sbb %rax, %rax
	sbb %rbx, %rax
	sbb %r8, %r9
	sbb %r8, $0
	sbb (%rax) .qword, $1000
	sbb (%rax + %rcx) .qword, $10
	sbb (%rax + %rcx*$4) .qword, $10
	sbb (%rcx*$4) .qword, %rax
	sbb (%rax+%rcx*$4+$20) .qword, $10
	sbb (%rax+%rcx*$4) .qword, $10
	sbb (%rax+%r8*$4+$20) .qword, $10
	sbb (%r9+%r8*$4+$20) .qword, $10
	sbb (%rax) .qword, %rax
	sbb (%rax + %rcx) .qword, %rax
	sbb (%rax + %rcx*$4) .qword, %rax
	sbb (%rcx*$4) .qword, %rax
	sbb (%rax+%rcx*$4+$20) .qword, %r8
	sbb (%rax+%rcx*$4) .qword, %r9
	sbb (%rax+%r8*$4+$20) .qword, %r10
	sbb (%r9+%r8*$4+$20) .qword, %r11
