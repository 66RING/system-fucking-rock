mod task;
mod context;
mod switch;

use crate::loader::{get_num_app,get_app_data};
use crate::sync::UPSafeCell;
pub use context::TaskContext;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use lazy_static::*;
use alloc::vec::Vec;
use crate::trap::TrapContext;

// 全局任务管理器
pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

// 变量(inner)常量分离
struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,        // 无法推断出还剩多少没有执行
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        // 依次为每个应用创建TCB
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(
                get_app_data(i),
                i,
            ));
        }
        TaskManager {
            num_app,
            inner: unsafe{ UPSafeCell::new(TaskManagerInner {
                tasks,
                current_task: 0,
            })},
        }
    };

}



fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

impl TaskManager {
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }
    fn run_next_task(&self) {
        // 找一个Ready的app
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            // 收到drop掉借用标记，因为__switch => 下一次调用才会返回
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(
                    current_task_cx_ptr,
                    next_task_cx_ptr
                );
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // 找current后第一个Ready的app
        // cool
        (current + 1..current + self.num_app + 1).map(|id| id % self.num_app)
            .find(|id| {
                inner.tasks[*id].task_status == TaskStatus::Ready
            })
    }
    // 加载第一个应用
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        // 运行第一个任务前并没有执行任何app，分配一个unused上下文
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(
                &mut _unused as *mut TaskContext,
                next_task_cx_ptr,
            );
        }
        panic!("unreachable in run_first_task!");
    }
    // 获取地址空间
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }
    // 获取trap上下文
    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }
}

// 修改状态然后切换
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}
// 获得当前应用地址空间token
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}
// 获得当前应用地址空间Trap上下文
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

