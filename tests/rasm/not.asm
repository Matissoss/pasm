!bits $64
_start:
	not %rax
	not %rax
	not %rax
	not %rbx
	not %r8
	not %r8
	not (%rax) !qword
	not (%rax + %rcx) !qword 
	not (%rax + %rcx*$4) !qword
	not (%rcx*$4) !qword
	not (%rax+%rcx*$4+$20) !qword
	not (%rax+%rcx*$4) !qword
	not (%rax+%r8*$4+$20) !qword
	not (%r9+%r8*$4+$20) !qword
	not (%rax) !qword
	not (%rax + %rcx) !qword
	not (%rax + %rcx*$4) !qword
	not (%rcx*$4) !qword
	not (%rax+%rcx*$4+$20) !qword
	not (%rax+%rcx*$4) !qword
	not (%rax+%r8*$4+$20) !qword
	not (%r9+%r8*$4+$20) !qword
