.altmacro
.macro SAVE_GP n
	sd x\n, \n*8(sp)
.endm

.macro LOAD_GP n
	ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
	# __alltraps地址4字节对齐，RISCV特权级规范要求
    .align 2
	# Trap时硬件会设置一些CSR，并跳到__alltraps
	# 此时sp -> user stack
__alltraps:
	# 原型: csrrw rd, csr, rs
	# 	csr->rd , rs->csr
	# 这里就是交换sccratch和sp, 
	# 执行前前sp->user stack, sscratch->kernel stack
	csrrw sp, sscratch, sp
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
	#		# 准备内核栈上保存上下文	
	#		# 	预分配34x8字节栈栈帧(内核栈中)
	#		addi sp, sp, -34*8
    # save general-purpose registers
	# 跳过x0和x4：少用或不用
	# x2(sp)后续要用
	# 栈帧区间(sp, sp+8*34)
	# 所以**TrapContext**内存布局:
	# 	基于sp, 从低到高分别x0~x31，最后是sstatus和sepc
	sd x1, 1*8(sp)
	# skip sp(x2), we will save it later
	sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
	.set n, 5
	# 开头加上 .altmacro 才能正常使用 .rept 命令
	.rept 27
		SAVE_GP %n
		.set n, n+1
	.endr
	# we can use t0/t1/t2 freely, because they were saved on kernel stack
	# 读取CSR然后保存到内核栈中
	csrr t0, sstatus
	csrr t1, sepc
	sd t0, 32*8(sp)
	sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
	# 专门处理sp(sccratch=trap前的sp值, 用户栈)
	csrr t2, sscratch
	sd t2, 2*8(sp)

	## ======= 地址空间切换 ========
	# TranContext内存布局
	# load kernel_satp(token) into t0
	ld t0, 34*8(sp)
	# load trap_handler into t1
	ld t1, 36*8(sp)
	# move to kernel_sp
	ld sp, 35*8(sp)
	# switch to kernel space
	csrw satp, t0
	# 刷新快表
	sfence.vma
	#		# set input argument of trap_handler(cx: &mut TrapContext)
	#		mv a0, sp
	#		# 通过寄存器(trap上下文)告知syscall ID等参数
	#		call trap_handler
	# 		  不用call是因为映射到次高页，而call会使用ip相对寻址，会找不到
	# jupm to trap_handler
	jr t1


__restore:
	# TODO: 删除早期不对的注释, 这里a0似乎就有点问题, 也许也不是, 因为这里不是batch system了
	# TODO: 整理见 "拉片分析trap机制"
	# a0: *TrapContext in user space(Constant); a1: user space token
	# swtich to user space
	# 切换回应用地址空间
	csrw satp, a1
	sfence.vma
	# 保存trap上下文
	csrw sscratch, a0
	# sp->TrapContext 用于恢复
	mv sp, a0
	### 因为在__switch后 sp已经指向所需trap上下文 mv sp, a0
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
	ld t0, 32*8(sp)
	ld t1, 33*8(sp)

	csrw sstatus, t0
	csrw sepc, t1

	#		# 初始化时 app_init_context 设置了x2 = user stack
	#		# 读x2(sp)
	#		ld t2, 2*8(sp)
	#		csrw sstatus, t0
	#		csrw sepc, t1
	#		csrw sscratch, t2

    # restore general-purpuse registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
	#		# release TrapContext on kernel stack
	#		# sp->kernel stack
	#		addi sp, sp, 34*8
	#		# now sp->kernel stack, sscratch->user stack
	#		csrrw sp, sscratch, sp
	#		# 有兴趣的同学可以思考： sscratch 是何时被设置为内核栈顶的？
	#		# 	case1
	# back to user stack
	ld sp, 2*8(sp)
    sret











