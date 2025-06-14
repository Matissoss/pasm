.bits $64
_start:
	vfmaddsub132ps %xmm0, %xmm1, %xmm2
	vfmaddsub213ps %xmm0, %xmm1, %xmm2
	vfmaddsub231ps %xmm0, %xmm1, %xmm2
	
	vfmaddsub132pd %xmm0, %xmm1, %xmm2
	vfmaddsub213pd %xmm0, %xmm1, %xmm2
	vfmaddsub231pd %xmm0, %xmm1, %xmm2
	
	vfmsubadd132ps %xmm0, %xmm1, %xmm2
	vfmsubadd213ps %xmm0, %xmm1, %xmm2
	vfmsubadd231ps %xmm0, %xmm1, %xmm2
	
	vfmsubadd132pd %xmm0, %xmm1, %xmm2
	vfmsubadd213pd %xmm0, %xmm1, %xmm2
	vfmsubadd231pd %xmm0, %xmm1, %xmm2
