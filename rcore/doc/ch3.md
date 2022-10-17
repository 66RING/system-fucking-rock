# 多道程序系统

## loader.rs

- `user/build.py`, 调整每个程序的起始地址, 内核根据其加载到不同位置
	* `build.py`依次调整链接脚本中的`BASE_ADDRESS`并编译每个程序
- 多道程序程序加载
	* 批处理系统是将程序加载到一个统一的内存地址, 内存一次只能驻留一个程序
	* 多道程序系统每个程序的加载到的内存地址不同, 内存中可以同时驻留多个程序
	* 加载函数的功能就是: 1. 读取程序地址 2. 根据`BASE_ADDRESS`算出加载地址 3. 拷贝


## TaskContext

- 内核中使用`TaskContext`保存进程的上下文: 寄存器组
	* ra: ra寄存器记录上下文切换后跳转的位置 
	* sp: sp寄存器记录了栈空间
	* 其他
- PCB: `TaskControlBlock`
	* 目前, PCB就是运行状态和上下文的集合
	* 最后再由一个`TaskManager`管理a list of pcb
- 多道运行
	* `sys_yield`, 暂时让出CPU, 切换其他程序
	* `sys_exit`, 生命结束, 切换其他程序
	* 即程序的状态不同, exit/suspend
- `__switch(ctx_curr, ctx_next)`
	* 寄存器现场保存到`ctx_curr`, 读取`ctx_next`的寄存器记录
	* 


## 进入用户态

每个程序出生时都在内核态中, `__switch`和`__restore`配合完成进入用户态操作。

- 初始Context另`ra`寄存器指向`__restore`
- `__restore`

这种情况下sscratch什么时候指向内核栈栈顶的?

**核心流程**: 为进程内核栈伪造一个TrapContext

1. 在当前任务的内核栈(`KERENL_STACK[id]`)压入TrapContext, 此时TrapContext记录待执行的程序, 而寄存器现场还是内核程序
	- TrapContext.sp指向 **用户栈**
	- **为进程内核栈伪造一个TrapContext**
2. 创建TaskContext
	- **TaskContext.sp指向任务的内核栈(`KERENL_STACK[id]`)**, `ra`指向`__restore`
	- 当前sp指向`boot_stack`
3. `__switch`
	- 加载`TaskContext.sp`到当前sp, **栈空间切换到进程内核栈**
4. `__switch`结束跳转到`__restore`
	- 因为此时sp指向进程内核栈, 而进程内核栈是刚刚(1)伪造压入的`TrapContext`
	- `__restore`通过交换`TrapContext.sp`和当前sp(进程内核栈)完成栈空间切换
	- **之后sscratch就指向了内核栈**


## 中断

- [RISCV中断](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/4time-sharing-system.html#risc-v)
	* (偷懒.
- 时钟中断与RR
	* 将`mtimecmp`寄存器设置为`mtime`寄存器触发中断的时机(计算计数器增量然后手动写入)
	* 中断触发进入`trap_handler`的`SupervisorTimer`case
	* 设置下次中断时间, 调度另一个程序





















