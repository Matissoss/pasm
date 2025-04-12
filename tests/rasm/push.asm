label:
	push %rax
	push %r8
	push $10
	push $256
	push $65537
	push (%rax) !qword
	push 8(%rax) !qword
	push 20(%rax, %rcx, $4) !qword
	push (%rax, %rcx, $4) !qword
