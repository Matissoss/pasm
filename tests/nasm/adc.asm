section .text
	bits 64
	global _start
_start:
	adc rax, 10
	adc rax, 256
	adc rax, 65537
	adc rbx, rax
	adc r8, r9
	adc r8, 0
	adc qword [rax], 1000
	adc qword [rax + rcx], 10
	adc qword [rax + rcx*4], 10
	adc qword [rcx*4], rax
	adc qword [rax+rcx*4+20], 10
	adc qword [rax+rcx*4], 10
	adc qword [rax+r8*4+20], 10
	adc qword [r9+r8*4+20], 10
	adc qword [rax], rax
	adc qword [rax + rcx], rax
	adc qword [rax + rcx*4], rax
	adc qword [rcx*4], rax
	adc qword [rax+rcx*4+20], r8
	adc qword [rax+rcx*4], r9
	adc qword [rax+r8*4+20], r10
	adc qword [r9+r8*4+20], r11
