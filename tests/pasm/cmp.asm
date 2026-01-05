bits 64
_start:
	cmp al, 8
	cmp ax, 256
	cmp ax, 255
	cmp eax, 65536
	cmp rax, 6546546
	cmp rbx, 10
	cmp byte [rax], 10
	cmp byte [rax], 128
	cmp word [rbx], 256
	cmp dword [rcx], 65536
	cmp qword [rdx], 65536
	cmp word [rbx], 255
	cmp dword [rcx], 255
	cmp qword [rcx], 255
	cmp byte [rax], al
	cmp word [rax], bx
	cmp dword [rax], ecx
	cmp qword [rax], rdx
	cmp al, byte [rax]
	cmp ax, word [rax]
	cmp eax, dword [rax]
	cmp rax, qword [rax]
