.bits $64
_start:
	shld %rax, %rbx, %cl
	shld %rax, %rbx, $10
	shld %eax, %ebx, %cl
	shld %eax, %ebx, $10
	shld %ax, %bx, %cl
	shld %ax, %bx, $10
	shrd %rax, %rbx, %cl
	shrd %rax, %rbx, $10
	shrd %eax, %ebx, %cl
	shrd %eax, %ebx, $10
	shrd %ax, %bx, %cl
	shrd %ax, %bx, $10

	wrfsbase %rbx
	wrgsbase %rbx

	sfence
	stac
	stc
	std
	sti
	stosb
	stosw
	stosd
	stosq
	stui
	sysenter
	sysexit
	sysret
	testui
	ud2
	uiret
	wait
	fwait
	wbinvd
	wrmsr
	wrpkru
	tpause %ecx
	umwait %ecx

	ud0 %eax, %ebx
	ud1 %eax, %ebx

	;umonitor %eax
	umonitor %rax

	smsw %ax
	smsw %eax
	smsw %rax

	str %ax
	verr %ax
	verw %ax
