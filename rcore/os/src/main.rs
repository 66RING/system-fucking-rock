#![no_main]
#![no_std]
#![feature(panic_info_message)]

use sbi::shutdown;

#[macro_use]
mod console;

mod lang_items;
mod sbi;
mod trap;
mod syscall;
mod batch;
mod sync;

core::arch::global_asm!(include_str!("entry.asm"));

// 为批处理系统硬编码方式链接应用程序
// link_app.S会在rust项目构建时自动调用根目录下的build.rs完成
core::arch::global_asm!(include_str!("link_app.S"));

// 定义入口函数
// no_mangle防止rust改名
#[no_mangle]
extern "C" fn rust_main() -> ! {
    clean_bss();
    println!("[kernel] Hello, world!");
    info!("system\n");
    warn!("fucking\n");
    debug!("rock\n");
    trap::init();
    batch::init();
    batch::run_next_app();
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
