mod context;
mod switch;
mod task;
mod manager;
mod processor;
mod pid;

use crate::loader::get_app_data_by_name;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;
use manager::fetch_task;
use lazy_static::*;
use crate::fs::{open_file, OpenFlags};

pub use context::TaskContext;
pub use processor::{
    run_tasks,
    current_task,
    current_user_token,
    current_trap_cx,
    take_current_task,
    schedule,
};
pub use manager::add_task;
pub use pid::{PidHandle, pid_alloc, KernelStack};


/// 暂停当前任务, 调度下一个就绪任务
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    // 获取当前任务
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    // 改变任务状态
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    // 调整到队尾
    add_task(task);
    // jump to scheduling cycle
    // 重新调度
    schedule(task_cx_ptr);
}

/// 退出当前任务, 调度下一个就绪任务
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();
    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    // 使能被父进程waitpid回收
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        // 退出进程的子进程转移到initproc
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB
    // 清空当前进程的子进程向量
    inner.children.clear();
    // deallocate user space
    // 回收当前进程的早期资源
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        // 基于文件系统打开初始进程
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}

/// 添加初始进程
pub fn add_initproc() {
    add_task(INITPROC.clone());
}

