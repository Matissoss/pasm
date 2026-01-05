bits 64
_start:
	inc al
	inc byte [rax]
	inc ax
	inc word [rax]
	inc eax
	inc dword [rax]
	inc rax
	inc qword [rax]
	inc r8
	inc r9
	dec al
	dec byte [rax]
	dec ax
	dec word [rax]
	dec eax
	dec dword [rax]
	dec rax
	dec qword [rax]
	dec r8
	dec r9
