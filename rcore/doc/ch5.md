# ch5 进程和进程管理

> 让开发者能够控制程序的运行

1. 开发一个shell
2. 对任务进一步抽象(进化) => 进程
3. `fork/exec/waitpid`三个系统调用
4. 用户初始化程序`initproc.rs`和shell程序`user_shell.rs`
	- 都处于应用层，前者被内核自动加载，后则负责接收用户指令

1. 重构进程管理相关数据结构
	- 更改`build.rs`, `loader.rs`, `link_app.S`，支持应用名来查找elf可执行文件，提供`get_app_data_by_name`接口
	- 分离`TaskManager`: `Processor`负责CPU上的任务，`TaskManager`负责所有任务
	- 扩展`TaskControlBlock`: PID, 内核栈, 父子进程，退出码等. `task.rs`
	- 使用pid作为进程控制块的索引. `pid.rs`
2. 实现进程管理机制
	- 初始进程的创建: `add_initproc`读取并加载ELF，加入`TASK_MANAGER`
	- 进程切换机制: 进程退出或让出时交出CPU使用权，调整`xxx_run_next`的实现，使用`schedule`函数切换
	- 进程调度方法: `TaskManager::fetch_task`
	- 进程生成机制: `fork/exec`, `process.rs`, `task.rs`, `TaskControlBlock::fork/exec`
	- 进程资源回收: `exit_xx`后立刻回收，父进程`waitpid`捕获退出码后被回收
	- 进程的IO: 支持终端读取键盘输入, `read`系统调用


# ch5

- fork
- waitpid
	* exit后不立刻回收，比如进程的内核栈正在处理。内核标记僵尸进程后父进程回收
- exec
	* 将当前进程的地址空间清空并加载一个特定的可执行文件

维护创建进程要fork再exec? => COW(copy on write) 且灵活支持重定向

- 用户初始化程序-init
- shell程序-user shell

## 进程概念

### 系统调用封装

`syscall.rs`中的系统调用在`lib.rs`中被封装成放方便使用的形式和多种api


### shell程序

- 需要获取输入并解析 => `sys_read`
	* `user_shell.rs`


## 进程管理的核心数据结构

1. 基于应用名的链接：编译链接阶段生成多个`link_app.S`
2. 基于应用名的加载器：根据应用名加载elf
3. 进程标识符`PidHandle`，内核栈`KernelStack`
4. 任务控制块`TaskControlBlock`
5. 任务管理器`TaskManager`
6. 处理器管理`Processor`：用于进程调度，维护进程状态


### 应用的链接和加载

链接：`exec`时根据名字获取elf格式数据。`build.rs`，生成链接脚本时，一并把应用名写入符号。

加载：`loader.rs`中分析`link_app.S`的内容，使用`APP_NAMES`全局向量保存应用名。


### 进程标识符和内核栈


1. 管理pid
	- 抽象一个`PidHandle`元组类，让生命周期自动回收
	- 需要唯一表示进程的标识符，类似`FrameAllocator`使用简单栈式策略
2. 内核栈
	- `KernelStack`中保存所属进程的pid
	- 也用RAII思想自动回收 => 实现Drop trait


### 进程控制块

- 改动`TaskControlBlock`，初始化后不再改变的元素和可以改变的元素`inner`
- 新增元素`parent`, `children`等
- 注意`Weak/Arc`引用方式


### 任务管理器

现在，任务管理器将仅负责管理所有任务，不再维护CPU当前任务。使用`Arc`队列为何所有任务，提供`add/fetch`接口。add加到队尾，fetch取一个可执行任务。


### 处理器管理结构

`Processor`负责维护CPU状态. `processor.rs`


- 正在执行的任务
	* 换入换出和一些API
- 任务调度的idle控制流
	* idle控制流会尝试取出一个任务来执行
	* 初始化后调用`run_tasks`进入idle控制流


TODO

- `schedule`总是切换到idle控制流
	* 当然也可以直接找到下一个任务然后切换过去

> 为什么需要额外的一个`idle_task_cx`，不能够直接使用之前`task_cx`吗，为何在前一个task和后一个task之间还要夹一个`idle_task_cx`

使 换入换出(进程自身内核栈) 和 调度流程(初始化时的启动栈) 是为了各自执行在不同的内核栈上，分别是进程自身内核栈和内核初始化时使用的启动栈。这样 调度相关数据不会出现在进程的内核栈上，调度机制对于换出进程的Trap不可见。分工明确，虽然增加开销。


## 进程管理机制的设计实现

- 创建初始进程: `initproc`
- 进程调度机制: `sys_yield`
- 进程生成机制: `sys_fork/sys_exec`
- 进程资源回收: `sys_exit`, `sys_waitpid`
- 字符输入机制: `sys_read`


### 初始进程创建

- 初始化初始进程控制块`INITPROC`
- `add_iniproc`将初始进程加入任务管理器
- `task.rs:new()`


### 进程调度机制

- `suspend_current_and_run_next`
- `sys_yield`


### 进程的生成机制

其他进程都直接/间接地从`initproc`中fork出来，再exec加载另一个可执行文件

- fork系统调用
	* 子进程创建一个和父进程几乎完全一样的地址空间
	* 子进程返回0, 父进程返回子进程pid。在交接处修改`trap_cx.x[10]`
- exec
	* 加载新elf替换原有地址空间内容. `TaskControlBlock::exec()`
	* 新增`translated_str`辅助函数从用户态空间获取字符串
- 系统调用后重新获取trap上下文
	* `trap_handler`中系统调用后再次调用`current_trap_cx()`。因为可能被exec释放了


### shell程序输入机制

实现`sys_read`获取用户的键盘输入

### 进程资源回收机制

- 进程退出
	* 设僵尸，回收资源(早期资源：地址空间)...
- 父进程回收子进程资源

