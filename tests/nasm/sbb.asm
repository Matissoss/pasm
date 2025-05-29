section .text
	bits 64
	global _start
_start:
	sbb rax, 10
	sbb rax, 256
	sbb rax, 65537
	sbb rax, rax
	sbb rbx, rax
	sbb r8, r9
	sbb r8, 0
	sbb qword [rax], 1000
	sbb qword [rax + rcx], 10
	sbb qword [rax + rcx*4], 10
	sbb qword [rcx*4], rax
	sbb qword [rax+rcx*4+20], 10
	sbb qword [rax+rcx*4], 10
	sbb qword [rax+r8*4+20], 10
	sbb qword [r9+r8*4+20], 10
	sbb qword [rax], rax
	sbb qword [rax + rcx], rax
	sbb qword [rax + rcx*4], rax
	sbb qword [rcx*4], rax
	sbb qword [rax+rcx*4+20], r8
	sbb qword [rax+rcx*4], r9
	sbb qword [rax+r8*4+20], r10
	sbb qword [r9+r8*4+20], r11
