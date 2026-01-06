bits 64
_start:
	and rax, 10
	and rax, 256
	and rax, 65537
	and rbx, rax
	and r8, r9
	and r8, 0
	and qword [rax], 1000
	and qword [rax + rcx], 10
	and qword [rax + rcx*4], 10
	and qword [rcx*4], rax
	and qword [rax+rcx*4+20], 10
	and qword [rax+rcx*4], 10
	and qword [rax+r8*4+20], 10
	and qword [r9+r8*4+20], 10
	and qword [rax], rax
	and qword [rax + rcx], rax
	and qword [rax + rcx*4], rax
	and qword [rcx*4], rax
	and qword [rax+rcx*4+20], r8
	and qword [rax+rcx*4], r9
	and qword [rax+r8*4+20], r10
	and qword [r9+r8*4+20], r11
