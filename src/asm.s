# .extern __tb_base

/* TODO
1. place this next to the actual thread storage, so we can construct an adr instruction pointing straight to it
   (needs to be a trampoline in ram? that's fine)
2. consider other implementations that don't require sequential storage. a system can have a lot of interrupts and not all of them need TLS, in fact only thread mode really does?
3. just use r9 platform register for this, that's what it's for!
*/

.thumb_func
.globl __aeabi_read_tp
.section .text.__aeabi_read_tp
__aeabi_read_tp:
	push {r1}
	mrs r0, IPSR
/*
	# dynamic tb_size, requires an extra register+mul
	.extern __tb_size
	ldr r1, __tb_size
	muls r0, r0, r1
*/
	.extern __tb_base_tdata # thread block base pointer + offset to tdata
	ldr r1, =__tb_base_tdata
	lsls r0, r0, #{{tb_pow2}}
	add r0, r0, r1

	pop {r1}
	bx lr
