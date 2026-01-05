bits 64
_start:
	mov eax, dword fs:[rax]
	mov ebx, dword cs:[rbx]
	mov ecx, dword es:[rax + 10]
	mov edx, dword ss:[rbx + rcx * 4]
	mov esp, dword gs:[rbx + rcx * 4 + 10]
	mov edi, dword ds:[rbx + rcx * 4 - 10]
