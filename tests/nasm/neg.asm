section .text
	bits 64

_start:
	neg rax
	neg rax
	neg rax
	neg rbx
	neg r8
	neg r8
	neg qword [rax]
	neg qword [rax + rcx]
	neg qword [rax + rcx*4]
	neg qword [rcx*4]
	neg qword [rax+rcx*4+20]
	neg qword [rax+rcx*4]
	neg qword [rax+r8*4+20]
	neg qword [r9+r8*4+20]
	neg qword [rax]
	neg qword [rax + rcx]
	neg qword [rax + rcx*4]
	neg qword [rcx*4]
	neg qword [rax+rcx*4+20]
	neg qword [rax+rcx*4]
	neg qword [rax+r8*4+20]
	neg qword [r9+r8*4+20]
