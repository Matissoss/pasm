bits 64
_start:
	cmova ax, word (rsp + 2)
	cmova eax, ebx
	cmova rax, rbx

	cmovb rax, rbx
	cmovc rax, rbx
	cmove rax, rbx
	cmovg rax, rbx
	cmovl rax, rbx
	cmovo rax, rbx
	cmovp rax, rbx
	cmovs rax, rbx
	cmovz rax, rbx
	cmovae rax, rbx
	cmovbe rax, rbx
	cmovge rax, rbx
	cmovle rax, rbx
	cmovna rax, rbx
	cmovnb rax, rbx
	cmovnc rax, rbx
	cmovne rax, rbx
	cmovng rax, rbx
	cmovnl rax, rbx
	cmovno rax, rbx
	cmovnp rax, rbx
	cmovns rax, rbx
	cmovnz rax, rbx
	cmovpe rax, rbx
	cmovpo rax, rbx
	cmovnbe rax, rbx
	cmovnae rax, rbx
	cmovnge rax, rbx
	cmovnle rax, rbx
