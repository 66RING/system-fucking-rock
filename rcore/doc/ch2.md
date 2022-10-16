# 批处理系统

- 引入程序地址空间, 规定程序加载到该地址后才可运行(pc指针)
- 引用内核态和用户态切换的方法, trap。系统调用, 中断等会导致用户态下陷, 切换特权级
	* 切换机制
		+ 通过修改sstatus寄存器SPP字段修改当前特权级(trap.S)
			+ 上下文切换
		+ 使用sret从s特权级回到


## **如何理解用户程序**

用户程序入口`user/lib.rs::_start()`, 就相当于linux中标准库对main的封装, 我们的标准库实现了一些打印和系统调用的功能。`user/src/bin`中的用户程序使用我们提供的标准库来实现功能, `cargo build`会对`bin`目录下的每个程序进行编译。

如果说内核是: `clear_bss`然后调用内核主函数的话, 那用户程序就是标准库封装`clear_bss`然后调用用户程序主函数。我们的批处理系统标准库封装用户程序, 用户程序结束后回到标准库, 标准库通过系统调用切换下一个程序。


## 特权级切换细节

> 本质就是状态寄存器的自动切换加手动的初态设置

- TODO: trap是指需要从用户态(U级)进入到内核态(S级)这件事, 进入内核态以做一些特权操作，如读写文件。系统调用就是一个trap的例子
- trapframe就是内核进程和用户进程相互切换时保存上下文使用的空间

- `sstatus`寄存器的SPP记录CPU当前特权级(U/S)
- sepc, TODO
- scause/stval, TODO
- stvec, TODO
- sscratch


### 用户态内核态切换

> trap先切换栈空间, restore最后再切换栈空间

> riscv的ld/sd指令格式: ld dst, src 而 sd src, dst方向是相反的

1. 需要使用额外一个寄存器(`sscratch`)保存用户栈/内核栈, 然后使用`csrrw`做交互从而完成栈空间的切换
2. 在内存中开辟一段空间(TrapContext)来保存用户态/内核态的上下文, 根据定义的内存布局将sp等寄存器保存到其中
3. 批处理系统第一次启动是我们处于内核态, 创建用户程序上下文(entry, sp)后, 通过`__restore`返回用户态
	1. 此时sp指向内核栈TrapContext, sscratch未初始化
	2. 将 **用户栈sp** 保存到sscratch, 然后加载一系列通用寄存器
	3. 最后sret前在将sscratch(用户栈)和当前sp(内核栈)交换就完成了栈空间和上下文的切换, sscratch第一次指向内核栈
4. trap发生时先通过`__alltraps`做统一的上下文切换, 在调用`trap_handler`做相应的处理
	1. 继上次`__restore`后, sscratch指向内核栈, sp指向用户栈
	2. 我们需要**先切换栈空间**以便访问到`TrapContext`保存上下文, 然后才是将用户的通用寄存器保存到TrapContext中

Q: 那内核的通用寄存器是怎么切换的? A: 内核态上下文的恢复只有恢复sp就够了。TODO


### abs

- 实现用户栈和内核栈来完成上下文切换
- trap上下文切换, TODO: 需要保存的寄存器
	* 保存上下文`__alltraps`, 执行`trap_handler`完成trap分发的处理, 恢复上下文`__restore`

