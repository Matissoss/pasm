.bits $64
.global _start
_start:
	adcx %eax, %ebx
	adcx %rax, %rbx
	adox %eax, %ebx
	adox %rax, %rbx

	cwde
	cdqe
	clac
	clts
	clui
	clwb .byte (%rax)

	blsr %eax, %ebx
	blsr %rax, %rbx
	blsi %eax, %ebx
	blsi %rax, %rbx
	blsi %rax, %rbx
	blsmsk %eax, %ebx
	blsmsk %rax, %rbx
	bswap %rax
	bswap %eax
