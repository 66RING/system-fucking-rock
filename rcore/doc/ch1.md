# abs

- 更换riscv工具链
	1. 使用rustup管理工具链, `rustup target add riscv64gc-unknown-none-elf`
	2. 编辑rust工程配置, 需要编译成riscv版本

```
# os/.cargo/config
[build]
target = "riscv64gc-unknown-none-elf"
```

- 基础设施
	* 系统调用
		+ ecall指令, 并使用x10, x11, x12, x17寄存器传递系统调用标号等参数
	* 基本打印
		+ 实现write系统调用后用户可以使用该系统调用打印字符额
		+ 使用rust宏和`$fmt`封装print方法, 方便进行打印
	* 使用sbi简化系统调用开发
		+ 如打印, 关机等
		+ 手写系统调用和sbi都使用
	* 指定链接脚本
		1. 在工程配置中指定链接脚本位置
		```
		[target.riscv64gc-unknown-none-elf]
		rustflags = [
			"-Clink-arg=-Tsrc/linker.ld", "-Cforce-frame-pointers=yes"
		]
		```
		2. 链接脚本可以看成是内存布局的模式匹配, 即脚本中内存布局是怎么定义的链接出来的二进制就是怎样
			- 几个要点
				1. `. = addr`表示从`.`开始地址就是addr
				2. `. = ALIGN(4K)`方法保证对齐
				3. =前后有空格
	* 使用`core::arch::global_asm!(include_str!("entry.asm"));`嵌入汇编代码
		+ 完成栈空间的分配和初始化(sp寄存器)等任务
		+ 在rust代码中使用`#[no_mangle]`标注函数防止改名

