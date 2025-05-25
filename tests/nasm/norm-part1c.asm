section .text
	bits 64
	global _start
_start:
	cmpsb
	cmpsw
	cmpsd
	cmpsq
	endbr32
	endbr64

	cmpxchg al, bl
	cmpxchg eax, ebx
	cmpxchg rax, rbx

	cldemote [rax]
	clrssbsy qword [rax]
	cmpxchg8b qword [rax]
	cmpxchg16b oword [rax]
