#![no_main]
#![no_std]
#![feature(panic_info_message)]

#[macro_use]
mod console;

mod lang_items;

mod sbi;

use sbi::shutdown;

core::arch::global_asm!(include_str!("entry.asm"));

// 定义入口函数
// no_mangle防止rust改名
#[no_mangle]
extern "C" fn rust_main() -> ! {
    print!("Hello, ");
    println!("world!");
    info!("system\n");
    warn!("fucking\n");
    debug!("rock\n");
    shutdown();
    // sys_exit(9);
}

// 内核态系统调用入口函数, 通过寄存器传递系统调用编号, ecall执行
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id,
        );
    }
    ret
}

// 清空bss段
// 链接脚本中暴露出了符号
// 可以使用extern访问到
// 直接转换成指针然后清零
fn clean_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
