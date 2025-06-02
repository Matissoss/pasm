!bits $64
!entry _start
!global _start

!const X !qword $10
!const str_X !byte "Hello, World!"
!const str_Y !word "Hello, World!"

!uninit Z $10
!uninit Y !qword

!ronly Xc !qword $10
!ronly str_Xc !byte "Hello, World!"
!ronly str_Yc !word "Hello, World!"

_start:
	mov %rax, $10
	lea %rax, @Z
	lea %rax, @_jmplabel
	jmp @_jmplabel
	jmp %rax
_jmplabel:
	mov %rax, $60
	mov %rdi, $0
	syscall
