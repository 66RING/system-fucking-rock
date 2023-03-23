// 通过 #[derive(...)] 可以让编译器为你的类型提供一些 Trait 的默认实现。
// 实现PartialEq可以使用==比较
use super::TaskContext;
use crate::mm::{MemorySet, MapPermission, PhysPageNum, KERNEL_SPACE, VirtAddr};
use crate::config::{TRAP_CONTEXT, kernel_stack_position};
use crate::trap::{TrapContext, trap_handler};



#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    // 应用地址空间
    pub memory_set: MemorySet,
    // 保存位于次高页的trap上下文
    pub trap_cx_ppn: PhysPageNum,
    // 应用数据大小: 0x0到用户栈结束， 即初始状态sp
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program header/trampoline/trap context/user stack
        // 解析ELF格式数据
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        // 找到trap上下文所在物理帧
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map a kernel stack int kernel space
        // 静态分配内核栈, 进程间内核栈相互隔离
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access()
            .insert_framed_area(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(),
                MapPermission::R | MapPermission::W,
            );
        let task_control_block = Self {
            task_status,
            // 应用的内核栈顶压入 "跳转到trap return"的任务上下文
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

}



