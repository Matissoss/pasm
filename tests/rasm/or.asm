!bits $64
_start:
	or  %rax, $10
	or  %rax, $256
	or  %rax, $65537
	or  %rbx, %rax
	or  %r8, %r9
	or  %r8, $0
	or  (%rax) !qword, $1000
	or  (%rax + %rcx) !qword, $10
	or  (%rax + %rcx*$4) !qword, $10
	or  (%rcx*$4) !qword, %rax
	or  (%rax+%rcx*$4+$20) !qword, $10
	or  (%rax+%rcx*$4) !qword, $10
	or  (%rax+%r8*$4+$20) !qword, $10
	or  (%r9+%r8*$4+$20) !qword, $10
	or  (%rax) !qword, %rax
	or  (%rax + %rcx) !qword, %rax
	or  (%rax + %rcx*$4) !qword, %rax
	or  (%rcx*$4) !qword, %rax
	or  (%rax+%rcx*$4+$20) !qword, %r8
	or  (%rax+%rcx*$4) !qword, %r9
	or  (%rax+%r8*$4+$20) !qword, %r10
	or  (%r9+%r8*$4+$20) !qword, %r11
