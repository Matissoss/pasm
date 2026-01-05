bits 64
_start:
	test al, 255
	test ax, 256
	test eax, 65536
	test rax, 65536
	test byte [rbx], 5
	test bl, 5
	test word [rbx], 256
	test bx, 256
	test dword [rbx], 65536
	test ebx, 65536
	test qword [rbx], 65537
	test rbx, 65536

	test al, bl
	test byte [rax], bl
	test ax, bx
	test word [rax], bx
	test eax, ebx
	test dword [rax], ebx
	test rax, rbx
	test qword [rax], rbx
