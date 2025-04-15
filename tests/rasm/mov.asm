_start:
	mov %rax, $1
	mov %rbx, $256
	mov %rcx, $65536
	mov %rdx, %rax
	mov %rdi, %rbx
	mov %rsi, %r9
	mov %rsp, %r10
	mov %rbp, %rsp
	mov %r8, %r8
	mov (%rax) !qword, $1000
	mov (%rax + %rcx) !qword, $10
	mov (%rax + %rcx*$4) !qword, $10
	mov (%rcx*$4) !qword, %rax
	mov (%rax+%rcx*$4+$20) !qword, $10
	mov (%rax+%rcx*$4) !qword, $10
	mov (%rax+%r8*$4+$20) !qword, $10
	mov (%r9+%r8*$4+$20) !qword, $10
	mov %rax, (%r9+%r8*$4+$20) !qword
