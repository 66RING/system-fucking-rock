mod context;

use core::arch::{global_asm, asm};

use riscv::register::{
    mtvec::TrapMode,
    stvec,
    scause::{
        self,
        Trap,
        Exception,
        Interrupt,
    },
    stval,
    sie,
};
use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next,
        current_trap_cx, current_user_token};
use crate::timer::set_next_trigger;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};

pub use context::TrapContext;



global_asm!(include_str!("trap.S"));

pub fn init() {
    set_kernel_trap_entry();
}

// trap分发和处理
#[no_mangle]
pub fn trap_handler() -> ! {
    // 将stvec修改为同模块下trap_from_kernel的地址
    //   设置S -> S 跳转，这里直接panic返回
    set_kernel_trap_entry();
    // Trap上下文不再kernel空间，通过current_trap_cx获取
    //      **不再是直接通过参数传入**
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        // U特权级的Environment call，即系统调用
        Trap::Exception(Exception::UserEnvCall) => {
            // 因为我们知道由ecall触发，进入trap时硬件置sepc为ecall所在
            // 所以恢复从ecall下一条
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
            // cx is changed during sys_exec. so we have to call it again
            // 因为它用来访问之前trap上下文的物理页帧
            cx = current_trap_cx();
            // trap返回值设为子进程pid
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            // page fault exit code
            exit_current_and_run_next(-2);
        }
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // 重置中断
            set_next_trigger();
            suspend_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    // 返回用户态
    //      换空间，且虚拟内存，所有有额外的清缓存
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

