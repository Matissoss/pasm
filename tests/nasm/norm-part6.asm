section .text
	bits 64
	global _start
_start:
	xadd al, bl
	xadd ax, bx
	xadd eax, ebx
	xadd rax, rbx

	xabort 0
	xacquire
	xrelease
	xend
	xgetbv
	xlat
	xlatb
	xresldtrk
	xsetbv
	xsusldtrk
	xtest

	xrstor [rax]
	xrstor [rax]
	
	xsave [rax]
	xsave64 [rax]
	
	xsavec [rax]
	xsavec64 [rax]
	
	xsaveopt [rax]
	xsaveopt64 [rax]
	
	xsaves [rax]
	xsaves64 [rax]

	lea eax, [rax + rcx * 4 + 10]

	lidt [rbx]
	lgdt [rbx]
