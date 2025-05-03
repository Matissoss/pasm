!global _start
!bits $64

_start:
	mov %eax, #fs:(%rax) !dword
	mov %ebx, #cs:(%rbx) !dword
	mov %ecx, #es:(%rax + $10) !dword
	mov %edx, #ss:(%rbx + %rcx * $4) !dword
	mov %esp, #gs:(%rbx + %rcx * $4 + $10) !dword
	mov %edi, #ds:(%rbx + %rcx * $4 - $10) !dword
