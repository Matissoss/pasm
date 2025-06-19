section .text
	bits 64
_start:
	lock adc rax, rbx
	lock add rax, rbx
	lock and rax, rbx
	lock btc rax, rbx
	lock bts rax, rbx
	lock btr rax, rbx
	lock cmpxchg rax, rbx
	lock cmpxchg16b oword [rax]
	lock cmpxchg8b qword [rax]
	lock dec rax
	lock inc rax
	lock neg rax
	lock not rax
	lock or rax, rbx
	lock sbb rax, rbx
	lock sub rax, rbx
	lock xadd rax, rbx
	lock xchg rax, rbx
	lock xor rax, rbx

	rep insb
	rep lodsb
	rep movsb
	rep outsb

	repe cmpsb
	repz cmpsw

	repne cmpsb
	repne scasb
