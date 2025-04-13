section .text
	bits 64
_start:
	push rax
	push r8
	push 10
	push 256
	push 65537
	push qword [rax]
	push qword [rax+8]
	push qword [rax+rcx*4+20]
	push qword [rax+rcx*4]
	push qword [rcx*4]
