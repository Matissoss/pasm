!bits $64
!global _start
_start:
	xadd %al, %bl
	xadd %ax, %bx
	xadd %eax, %ebx
	xadd %rax, %rbx

	xabort $0
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

	xrstor !dword (%rax)
	xrstor64 !qword (%rax)
	
	xsave !dword (%rax)
	xsave64 !qword (%rax)
	
	xsavec !dword (%rax)
	xsavec64 !qword (%rax)
	
	xsaveopt !dword (%rax)
	xsaveopt64 !qword (%rax)
	
	xsaves !dword (%rax)
	xsaves64 !qword (%rax)
