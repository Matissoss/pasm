section .text
	bits 64

_start:
	or rax, 10
	or rax, 256
	or  rax, 65537
	or  rbx, rax
	or  r8, r9
	or  r8, 0
	or  qword [rax], 1000
	or  qword [rax + rcx], 10
	or  qword [rax + rcx*4], 10
	or  qword [rcx*4], rax
	or  qword [rax+rcx*4+20], 10
	or  qword [rax+rcx*4], 10
	or  qword [rax+r8*4+20], 10
	or  qword [r9+r8*4+20], 10
	or  qword [rax], rax
	or  qword [rax + rcx], rax
	or  qword [rax + rcx*4], rax
	or  qword [rcx*4], rax
	or  qword [rax+rcx*4+20], r8
	or  qword [rax+rcx*4], r9
	or  qword [rax+r8*4+20], r10
	or  qword [r9+r8*4+20], r11
