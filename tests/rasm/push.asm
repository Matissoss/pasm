_start:
	push %rax
	push %r8
	push $10
	push $256
	push $65537
	push (%rax) !qword
	push (%rax+$8) !qword
	push (%rax,%rcx*$4+$20) !qword
	push (%rax,%rcx*$4) !qword
	push (%rcx*$4) !qword
