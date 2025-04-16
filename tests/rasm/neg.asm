_start:
	neg %rax
	neg %rax
	neg %rax
	neg %rbx
	neg %r8
	neg %r8
	neg (%rax) !qword
	neg (%rax + %rcx) !qword 
	neg (%rax + %rcx*$4) !qword
	neg (%rcx*$4) !qword
	neg (%rax+%rcx*$4+$20) !qword
	neg (%rax+%rcx*$4) !qword
	neg (%rax+%r8*$4+$20) !qword
	neg (%r9+%r8*$4+$20) !qword
	neg (%rax) !qword
	neg (%rax + %rcx) !qword
	neg (%rax + %rcx*$4) !qword
	neg (%rcx*$4) !qword
	neg (%rax+%rcx*$4+$20) !qword
	neg (%rax+%rcx*$4) !qword
	neg (%rax+%r8*$4+$20) !qword
	neg (%r9+%r8*$4+$20) !qword
