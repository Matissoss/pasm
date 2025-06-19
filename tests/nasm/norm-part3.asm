section .text
	bits 64
	global _start
_start:
	mulx eax, ebx, ecx
	mulx rax, rbx, rcx
	pext eax, ebx, ecx
	pext rax, rbx, rcx
	pdep eax, ebx, ecx
	pdep rax, rbx, rcx

	movzx ax, byte [rax]
	movzx eax, byte [rax]
	movzx eax, word [rax]
	movzx rax, byte [rax]
	movzx rax, word [rax]

	movsb
	movsw
	movsd
	movsq

	movdiri dword [rax], eax
	movdiri qword [rax], rax

	movbe ax, word [rax]
	movbe eax, dword [rax]
	movbe rax, qword [rax]
	movbe word [rax], ax
	movbe dword [rax], eax
	movbe qword [rax], rax

	lzcnt ax, bx
	lzcnt eax, ebx
	lzcnt rax, rbx

	ltr word [rax]

	prefetchw [rax]
	;prefetch0 [rax]
	;prefetch1 [rax]
	;prefetch2 [rax]
	;prefetcha [rax]

	lsl ax, bx
	lsl eax, ebx
	lsl rax, rbx

	out 10, al
	out 10, ax
	out 10, eax

	out dx, al
	out dx, ax

	outsb
	outsw
	outsd
	
	out dx, eax
