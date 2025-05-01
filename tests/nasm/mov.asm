section .text
	bits 64

_start:
	mov rax, dr0
	mov rax, dr8
	mov rax, dr9
	mov dr0, rax
	mov dr8, rax
	mov dr9, rax
	mov rax, cr0
	mov rax, cr8
	mov rax, cr9
	mov cr0, rax
	mov cr8, rax
	mov cr9, rax
	mov rax, fs
	mov eax, 1
	mov rax, 1
	mov rax, 60
	mov rbx, 256
	mov rcx, 65536
	mov rdx, rax
	mov rdi, rbx
	mov rsi, r9
	mov rsp, r10
	mov rbp, rsp
	mov r8, r8
	mov qword [rax], 1000
	mov qword [rax + rcx], 10
	mov qword [rax + rcx*4], 10
	mov qword [rcx*4], rax
	mov qword [rax+rcx*4+20], 10
	mov qword [rax+rcx*4], 10
	mov qword [rax+r8*4+20], 10
	mov qword [r9+r8*4+20], 10
	mov rax, qword [r9+r8*4+20]
