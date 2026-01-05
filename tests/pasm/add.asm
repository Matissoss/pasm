bits 64
_start:
	add rax, 10
	add rax, 256
	add rax, 65537
	add rbx, rax
	add r8, r9
	add r8, 0
	add qword [rax], 1000
	add qword [rax + rcx], 10
	add qword [rax + rcx*4], 10
	add qword [rcx*4], rax
	add qword [rax+rcx*4+20], 10
	add qword [rax+rcx*4], 10
	add qword [rax+r8*4+20], 10
	add qword [r9+r8*4+20], 10
	add qword [rax], rax
	add qword [rax + rcx], rax
	add qword [rax + rcx*4], rax
	add qword [rcx*4], rax
	add qword [rax+rcx*4+20], r8
	add qword [rax+rcx*4], r9
	add qword [rax+r8*4+20], r10
	add qword [r9+r8*4+20], r11
