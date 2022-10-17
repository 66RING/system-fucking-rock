.altmacro
.macro SAVE_SN n
	sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
	ld s\n, (\n+2)*8(a1)
.endm

	.section .text
	.globl __switch
__switch:
	# 阶段 [1]
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
	# RISCV 调用规范 => 这里的参数在a0 a1
    # 阶段 [2]
    # save kernel stack of current task
	sd sp, 8(a0)
	# save ra & s0~s11 of current execution
	##
	## 符合TaskContext的布局
	##
	sd ra, 0(a0)
	.set n, 0
	.rept 12
		SAVE_SN %n
		.set n, n+1
	.endr
	# 对于一般的函数而言，Rust/C 编译器会在函数的起始位置自动生成代码来保存 s0~s11 这些被调用者保存的寄存器。但 __switch 是一个用汇编代码写的特殊函数，它不会被 Rust/C 编译器处理，所以我们需要在 __switch 中手动编写保存 s0~s11 的汇编代码
	# 阶段 [3]
	# restore ra & s0~s11 of next execution
	ld ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
	# restore kernel stack of next task
	ld sp, 8(a1)
	# 阶段 [4]
	ret












