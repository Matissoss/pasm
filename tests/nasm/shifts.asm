section .text
	bits 64

_start:
	shl al, 1
	shl al, cl
	shl al, 10
	shl ax, 1
	shl ax, cl
	shl ax, 10
	shl eax, 1
	shl eax, cl
	shl eax, 10
	shl rax, 1
	shl rax, cl
	shl rax, 10
	
	sal al, 1
	sal al, cl
	sal al, 10
	sal ax, 1
	sal ax, cl
	sal ax, 10
	sal eax, 1
	sal eax, cl
	sal eax, 10
	sal rax, 1
	sal rax, cl
	sal rax, 10

	shr al, 1
	shr al, cl
	shr al, 10
	shr ax, 1
	shr ax, cl
	shr ax, 10
	shr eax, 1
	shr eax, cl
	shr eax, 10
	shr rax, 1
	shr rax, cl
	shr rax, 10

	sar al, 1
	sar al, cl
	sar al, 10
	sar ax, 1
	sar ax, cl
	sar ax, 10
	sar eax, 1
	sar eax, cl
	sar eax, 10
	sar rax, 1
	sar rax, cl
	sar rax, 10
