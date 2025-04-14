section .text
	bits 64

_start:
	mov rax, 1
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
