_start:
	cmp %al, $8
	cmp %ax, $256
	cmp %ax, $255
	cmp %eax, $65536
	cmp %rax, $6546546
	cmp %rbx, $10
	cmp (%rax) !byte, $10
	cmp (%rax) !byte, $128
	cmp (%rbx) !word, $256
	cmp (%rcx) !dword, $65536
	cmp (%rdx) !qword, $65536
	cmp (%rbx) !word, $255
	cmp (%rcx) !dword, $255
	cmp (%rcx) !qword, $255
	cmp (%rax) !byte, %al
	cmp (%rax) !word, %bx
	cmp (%rax) !dword, %ecx
	cmp (%rax) !qword, %rdx
	cmp %al, (%rax) !byte
	cmp %ax, (%rax) !word
	cmp %eax, (%rax) !dword
	cmp %rax, (%rax) !qword
