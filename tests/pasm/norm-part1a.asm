.bits $64
_start:
	bsf %ax, %bx
	bsf %eax, %ebx
	bsf %rax, %rbx
	bsr %ax, %bx
	bsr %eax, %ebx
	bsr %rax, %rbx

	bt %ax, %bx
	bt %ax, $10
	bt %eax, %ebx
	bt %eax, $10
	bt %rax, %rbx
	bt %rax, $10
	
	bts %ax, %bx
	bts %ax, $10
	bts %eax, %ebx
	bts %eax, $10
	bts %rax, %rbx
	bts %rax, $10
	
	btr %ax, %bx
	btr %ax, $10
	btr %eax, %ebx
	btr %eax, $10
	btr %rax, %rbx
	btr %rax, $10
	
	btc %ax, %bx
	btc %ax, $10
	btc %eax, %ebx
	btc %eax, $10
	btc %rax, %rbx
	btc %rax, $10

	cbw
	cmc
	cwd
	cdq
	cqo
	cld
	cli
