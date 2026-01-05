bits 64
_start:
	xor rax, 10
	xor rax, 256
	xor rax, 65537
	xor rax, rax
	xor rbx, rax
	xor r8, r9
	xor r8, 0
	xor qword [rax], 1000
	xor qword [rax + rcx], 10
	xor qword [rax + rcx*4], 10
	xor qword [rcx*4], rax
	xor qword [rax+rcx*4+20], 10
	xor qword [rax+rcx*4], 10
	xor qword [rax+r8*4+20], 10
	xor qword [r9+r8*4+20], 10
	xor qword [rax], rax
	xor qword [rax + rcx], rax
	xor qword [rax + rcx*4], rax
	xor qword [rcx*4], rax
	xor qword [rax+rcx*4+20], r8
	xor qword [rax+rcx*4], r9
	xor qword [rax+r8*4+20], r10
	xor qword [r9+r8*4+20], r11
