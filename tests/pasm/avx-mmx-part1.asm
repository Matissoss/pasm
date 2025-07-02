.bits $64
_start:
	vpor %xmm0, %xmm1, %xmm2
	vmovd %xmm0, %eax
	vmovd %eax, %xmm1
	vmovq %xmm0, %rax
	vmovq %rax, %xmm2
	vpand %xmm0, %xmm1, %xmm2
	vpxor %xmm0, %xmm1, %xmm2
	vpand %xmm0, %xmm1, %xmm2
	vpaddb %xmm0, %xmm1, %xmm2
	vpaddw %xmm0, %xmm1, %xmm2
	vpaddd %xmm0, %xmm1, %xmm2
	vpaddq %xmm0, %xmm1, %xmm2
	vpsubb %xmm0, %xmm1, %xmm2
	vpsubw %xmm0, %xmm1, %xmm2
	vpsubd %xmm0, %xmm1, %xmm2
	vpsubq %xmm0, %xmm1, %xmm2
	vpandn %xmm0, %xmm1, %xmm2
	
	vpsllw %xmm0, %xmm1, %xmm2
	vpsllw %xmm0, %xmm1, $10
	vpslld %xmm0, %xmm1, %xmm2
	vpslld %xmm0, %xmm1, $10
	vpsllq %xmm0, %xmm1, %xmm2
	vpsllq %xmm0, %xmm1, $10
	
	vpsrlw %xmm0, %xmm1, %xmm2
	vpsrlw %xmm0, %xmm1, $10
	vpsrld %xmm0, %xmm1, %xmm2
	vpsrld %xmm0, %xmm1, $10
	vpsrlq %xmm0, %xmm1, %xmm2
	vpsrlq %xmm0, %xmm1, $10
	
	vpsraw %xmm0, %xmm1, %xmm2
	vpsraw %xmm0, %xmm1, $10
	vpsrad %xmm0, %xmm1, %xmm2
	vpsrad %xmm0, %xmm1, $10
	
	vpsubsb %xmm0, %xmm1, %xmm2
	vpsubsw %xmm0, %xmm1, %xmm2
	vpaddsb %xmm0, %xmm1, %xmm2
	vpaddsw %xmm0, %xmm1, %xmm2
	vpmullw %xmm0, %xmm1, %xmm2
	vpmulhw %xmm0, %xmm1, %xmm2
