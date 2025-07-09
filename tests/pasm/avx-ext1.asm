bits 64
_start:
	kaddb k0, k1, k2
	kaddw k0, k1, k2
	kaddd k0, k1, k2
	kaddq k0, k1, k2
	kandb k0, k1, k2
	kandw k0, k1, k2
	kandd k0, k1, k2
	kandq k0, k1, k2
	kandnb k0, k1, k2
	kandnw k0, k1, k2
	kandnd k0, k1, k2
	kandnq k0, k1, k2
	korb k0, k1, k2
	korw k0, k1, k2
	kord k0, k1, k2
	korq k0, k1, k2
	kxorb k0, k1, k2
	kxorw k0, k1, k2
	kxord k0, k1, k2
	kxorq k0, k1, k2
	kxnorb k0, k1, k2
	kxnorw k0, k1, k2
	kxnord k0, k1, k2
	kxnorq k0, k1, k2
	knotb k0, k1
	knotw k0, k1
	knotd k0, k1
	knotq k0, k1
	ktestb k0, k1
	ktestw k0, k1
	ktestd k0, k1
	ktestq k0, k1
	kortestb k0, k1
	kortestw k0, k1
	kortestd k0, k1
	kortestq k0, k1
	kunpckbw k0, k1, k2
	kunpckdq k0, k1, k2
	kunpckwd k0, k1, k2
	
	kmovb byte (rcx), k2
	kmovb k2, byte (rcx)
	kmovb k1, k2
	kmovb ebx, k2
	kmovb k2, ebx

	kmovw word (rcx), k2
	kmovw k2, word (rcx)
	kmovw k1, k2
	kmovw ebx, k2
	kmovw k2, ebx
	
	kmovd dword (rcx), k2
	kmovd k2, dword (rcx)
	kmovd k1, k2
	kmovd ebx, k2
	kmovd k2, ebx
	
	kmovq qword (rcx), k2
	kmovq k2, qword (rcx)
	kmovq k1, k2
	kmovq rbx, k2
	kmovq k2, rbx
