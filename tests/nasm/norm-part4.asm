section .text
	bits 64
	global _start
_start:
	scasq
	senduipi rax
	rdrand rax
	rdrand eax
	rdrand ax
	rdseed rax
	rdseed eax
	rdseed ax
	rdsspd eax
	rdsspq rax
	rorx rax, rbx, 1
	shlx rax, rbx, rcx
	shrx rax, rbx, rcx
	sarx rax, rbx, rcx
	rdpid rax

	rdmsr
	rdpkru
	rdpmc
	rdtsc
	rdtscp
	rsm
	sahf
	scasb
	scasw
	scasd
	serialize
	setssby
	rstorssp qword [rax]
