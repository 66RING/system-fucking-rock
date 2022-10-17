mod context;

pub use context::TrapContext;
use crate::syscall::syscall;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap, Interrupt},
    stval, stvec,
    sie,
};
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;


core::arch::global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" { fn __alltraps(); }
    // 中断处理函数入口
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}


// trap的分发和处理
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        // 处理系统调用
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, core dumped.");
            // 进程状态标记退出, 执行下一个程序
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            // 进程状态标记退出, 执行下一个程序
            exit_current_and_run_next();
        }
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // 设置下一次中断时间
            set_next_trigger();
            // 标记suspend, 调度下一个程序
            suspend_current_and_run_next();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    cx
}

// 使S特权级时钟中断不被屏蔽
pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer(); }
}

