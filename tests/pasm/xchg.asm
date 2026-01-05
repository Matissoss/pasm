bits 64
_start:
	xchg rbx, rax
	xchg r8, r9
	xchg qword [rcx*4], rax
	xchg qword [rax], rax
	xchg qword [rax + rcx], rax
	xchg qword [rax + rcx*4], rax
	xchg qword [rcx*4], rax
	xchg qword [rax+rcx*4+20], r8
	xchg qword [rax+rcx*4], r9
	xchg qword [rax+r8*4+20], r10
	xchg qword [r9+r8*4+20], r11
