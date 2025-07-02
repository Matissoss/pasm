#(bits=32)
_start:
	mov %eax, .dword (%eax + %ebx * $1)
	mov %ax, %bx
	or %ax, %bx
