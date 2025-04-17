_start:
	xor %rax, $10
	xor %rax, $256
	xor %rax, $65537
	xor %rax, %rax
	xor %rbx, %rax
	xor %r8, %r9
	xor %r8, $0
	xor (%rax) !qword, $1000
	xor (%rax + %rcx) !qword, $10
	xor (%rax + %rcx*$4) !qword, $10
	xor (%rcx*$4) !qword, %rax
	xor (%rax+%rcx*$4+$20) !qword, $10
	xor (%rax+%rcx*$4) !qword, $10
	xor (%rax+%r8*$4+$20) !qword, $10
	xor (%r9+%r8*$4+$20) !qword, $10
	xor (%rax) !qword, %rax
	xor (%rax + %rcx) !qword, %rax
	xor (%rax + %rcx*$4) !qword, %rax
	xor (%rcx*$4) !qword, %rax
	xor (%rax+%rcx*$4+$20) !qword, %r8
	xor (%rax+%rcx*$4) !qword, %r9
	xor (%rax+%r8*$4+$20) !qword, %r10
	xor (%r9+%r8*$4+$20) !qword, %r11
