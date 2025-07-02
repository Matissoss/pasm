.bits $64
.define math_test $(10 * 2)
_start:
	mov %rax, $(2 * 5)
	mov %rax, @math_test
