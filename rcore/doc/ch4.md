# 地址空间

- RISCV页表机制
	* `satp`寄存器启动分页
	* MODE为0时访存视为物理地址
	* MODE为8时SV39分页启动, 访存虚拟地址, MMU转换
```
# satp
| 63 .. 60 | 59 .. 44 | 43 .. 0 |
 MODE       ASID       PPN
```

- SV39分页模式
	* TODO: 页表项: `...`
- TODO: 理清rcore中对分页的描述

- 物理页帧管理
	* 栈式物理页帧管理, TODO
	* `frame_allocator.rs`
- 多级页表
- 地址空间
- 内核空间
	* trampoline

- rcore中heap_allocator和frame_allocator的区别?
    * `heap_allocator`是内核堆的分配, 用于使能rust堆分配, 如智能指针, vec等
    * `frame_allocator`是我们操作系统管理的堆, 用于分配页等
    * `heap_allocator`和`frame_allocator`会重叠吗?
        + 不会, 因为链接器中规定了我们操作系统这个程序的bss段, 我们的内核堆`HEAP_SPACE`是静态开辟在那的, 而`frame_allocator`是bss段后的空间

## Result

- TODO: 内存分配器
	* 内存空间之际管理
- TODO: 栈式内存管理
- 开启分页
	* 设置satp寄存器: mode + 页表地址
	* satp寄存器mode位域置为8
- trap逻辑变更
	* 启用虚拟内存后每个程序看到的都是独立的空间, 不再像上一节分蛋糕一样分段使用了
	* TODO: 约定?
		+ 内存最高处映射"trampoline"
			+ 为何??
		+ trampoline之下映射TrapContext
			+ 为何??
- TODO: trap from user & trap from kernel 
	* 暂不支持trap from kernel
	* trap from user
- utils
	* 虚拟地址翻译, 用于内核维护用户的页表
	* TODO: 上下取整
