!bits $64
_start:
	sub %rax, $10
	sub %rax, $256
	sub %rax, $65537
	sub %rbx, %rax
	sub %r8, %r9
	sub %r8, $0
	sub (%rax) !qword, $1000
	sub (%rax + %rcx) !qword, $10
	sub (%rax + %rcx*$4) !qword, $10
	sub (%rcx*$4) !qword, %rax
	sub (%rax+%rcx*$4+$20) !qword, $10
	sub (%rax+%rcx*$4) !qword, $10
	sub (%rax+%r8*$4+$20) !qword, $10
	sub (%r9+%r8*$4+$20) !qword, $10
	sub (%rax) !qword, %rax
	sub (%rax + %rcx) !qword, %rax
	sub (%rax + %rcx*$4) !qword, %rax
	sub (%rcx*$4) !qword, %rax
	sub (%rax+%rcx*$4+$20) !qword, %r8
	sub (%rax+%rcx*$4) !qword, %r9
	sub (%rax+%r8*$4+$20) !qword, %r10
	sub (%r9+%r8*$4+$20) !qword, %r11
