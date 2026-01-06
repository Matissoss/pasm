bits 64
_start:
	not rax
	not rax
	not rax
	not rbx
	not r8
	not r8
	not qword [rax]
	not qword [rax + rcx] 
	not qword [rax + rcx*4]
	not qword [rcx*4]
	not qword [rax+rcx*4+20]
	not qword [rax+rcx*4]
	not qword [rax+r8*4+20]
	not qword [r9+r8*4+20]
	not qword [rax]
	not qword [rax + rcx]
	not qword [rax + rcx*4]
	not qword [rcx*4]
	not qword [rax+rcx*4+20]
	not qword [rax+rcx*4]
	not qword [rax+r8*4+20]
	not qword [r9+r8*4+20]
