.bits $64
.global _start
_start:
	adc %rax, $10
	adc %rax, $256
	adc %rax, $65537
	adc %rbx, %rax
	adc %r8, %r9
	adc %r8, $0
	adc (%rax) .qword, $1000
	adc (%rax + %rcx) .qword, $10
	adc (%rax + %rcx*$4) .qword, $10
	adc (%rcx*$4) .qword, %rax
	adc (%rax+%rcx*$4+$20) .qword, $10
	adc (%rax+%rcx*$4) .qword, $10
	adc (%rax+%r8*$4+$20) .qword, $10
	adc (%r9+%r8*$4+$20) .qword, $10
	adc (%rax) .qword, %rax
	adc (%rax + %rcx) .qword, %rax
	adc (%rax + %rcx*$4) .qword, %rax
	adc (%rcx*$4) .qword, %rax
	adc (%rax+%rcx*$4+$20) .qword, %r8
	adc (%rax+%rcx*$4) .qword, %r9
	adc (%rax+%r8*$4+$20) .qword, %r10
	adc (%r9+%r8*$4+$20) .qword, %r11
