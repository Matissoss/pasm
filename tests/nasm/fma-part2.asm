section .text
	bits 64
	global _start
_start:
	vfnmadd132ps xmm0, xmm1, xmm2
	vfnmadd213ps xmm0, xmm1, xmm2
	vfnmadd231ps xmm0, xmm1, xmm2
	
	vfnmadd132pd xmm0, xmm1, xmm2
	vfnmadd213pd xmm0, xmm1, xmm2
	vfnmadd231pd xmm0, xmm1, xmm2
	
	vfnmadd132ss xmm0, xmm1, xmm2
	vfnmadd213ss xmm0, xmm1, xmm2
	vfnmadd231ss xmm0, xmm1, xmm2
	
	vfnmadd132sd xmm0, xmm1, xmm2
	vfnmadd213sd xmm0, xmm1, xmm2
	vfnmadd231sd xmm0, xmm1, xmm2
	
	vfnmsub132ps xmm0, xmm1, xmm2
	vfnmsub213ps xmm0, xmm1, xmm2
	vfnmsub231ps xmm0, xmm1, xmm2
	
	vfnmsub132pd xmm0, xmm1, xmm2
	vfnmsub213pd xmm0, xmm1, xmm2
	vfnmsub231pd xmm0, xmm1, xmm2
	
	vfnmsub132ss xmm0, xmm1, xmm2
	vfnmsub213ss xmm0, xmm1, xmm2
	vfnmsub231ss xmm0, xmm1, xmm2
	
	vfnmsub132sd xmm0, xmm1, xmm2
	vfnmsub213sd xmm0, xmm1, xmm2
	vfnmsub231sd xmm0, xmm1, xmm2
