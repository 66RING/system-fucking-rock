mod context;

pub use context::TrapContext;
use crate::{syscall::syscall, task::{current_trap_cx, current_user_token}, config::{TRAMPOLINE, TRAP_CONTEXT}};
use core::arch::{global_asm, asm};
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
    set_kernel_trap_entry();
}


// trap的分发和处理
#[no_mangle]
pub fn trap_handler() -> ! {
    // 设置内核态的trap处理
    // 将stvec修改为同模块下trap_from_kernel的地址
    //   设置S -> S 跳转，这里直接panic返回
    set_kernel_trap_entry();
    // Trap上下文不再kernel空间，通过current_trap_cx获取
    //      **不再是直接通过参数传入**
    let cx = current_trap_cx();

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

    // 返回用户态
    //  换空间，切虚拟内存，所有有额外的清缓存
    trap_return();
}

// 使S特权级时钟中断不被屏蔽
pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer(); }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_return() -> ! {
    // 让(恢复)trap时跳转到__alltraps
    // 设为TRAMPOLINE而不是__alltraps
    //  因为启动分页后内内核只能通过跳板获取__alltraps和__restore的汇编代码
    set_user_trap_entry();
    // __restore的参数
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    // **计算__restore虚地址**
    //  因为__alltraps的对齐到TRAMPOLINE的，__restore的虚地址只需要加上偏移
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            // fence.i清空 i-cache
            "fence.i",
            // 跳转到`__restore`执行
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

