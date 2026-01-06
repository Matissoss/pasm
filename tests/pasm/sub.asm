bits 64
_start:
	sub rax, 10
	sub rax, 256
	sub rax, 65537
	sub rbx, rax
	sub r8, r9
	sub r8, 0
	sub qword [rax], 1000
	sub qword [rax + rcx], 10
	sub qword [rax + rcx*4], 10
	sub qword [rcx*4], rax
	sub qword [rax+rcx*4+20], 10
	sub qword [rax+rcx*4], 10
	sub qword [rax+r8*4+20], 10
	sub qword [r9+r8*4+20], 10
	sub qword [rax], rax
	sub qword [rax + rcx], rax
	sub qword [rax + rcx*4], rax
	sub qword [rcx*4], rax
	sub qword [rax+rcx*4+20], r8
	sub qword [rax+rcx*4], r9
	sub qword [rax+r8*4+20], r10
	sub qword [r9+r8*4+20], r11
