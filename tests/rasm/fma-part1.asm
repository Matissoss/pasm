!bits $64
!global _start
_start:
	vfmadd132ps %xmm0, %xmm1, %xmm2
	vfmadd213ps %xmm0, %xmm1, %xmm2
	vfmadd231ps %xmm0, %xmm1, %xmm2
	
	vfmadd132pd %xmm0, %xmm1, %xmm2
	vfmadd213pd %xmm0, %xmm1, %xmm2
	vfmadd231pd %xmm0, %xmm1, %xmm2
	
	vfmadd132ss %xmm0, %xmm1, %xmm2
	vfmadd213ss %xmm0, %xmm1, %xmm2
	vfmadd231ss %xmm0, %xmm1, %xmm2
	
	vfmadd132sd %xmm0, %xmm1, %xmm2
	vfmadd213sd %xmm0, %xmm1, %xmm2
	vfmadd231sd %xmm0, %xmm1, %xmm2
	
	vfmsub132ps %xmm0, %xmm1, %xmm2
	vfmsub213ps %xmm0, %xmm1, %xmm2
	vfmsub231ps %xmm0, %xmm1, %xmm2
	
	vfmsub132pd %xmm0, %xmm1, %xmm2
	vfmsub213pd %xmm0, %xmm1, %xmm2
	vfmsub231pd %xmm0, %xmm1, %xmm2
	
	vfmsub132ss %xmm0, %xmm1, %xmm2
	vfmsub213ss %xmm0, %xmm1, %xmm2
	vfmsub231ss %xmm0, %xmm1, %xmm2
	
	vfmsub132sd %xmm0, %xmm1, %xmm2
	vfmsub213sd %xmm0, %xmm1, %xmm2
	vfmsub231sd %xmm0, %xmm1, %xmm2
