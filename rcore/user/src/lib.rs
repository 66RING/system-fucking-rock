#![no_std]
#![feature(asm)]
#![feature(linkage)]
#![feature(panic_info_message)]


// 大部分和内核程序的一样的
#[macro_use]
pub mod console;
mod syscall;
mod lang_items;

use crate::syscall::*;

// 此处为进入系统后的应用程序环境

// 我们的批处理系统会顺序加载然后运行
// 实现应用程序需要的工具, 相当于标准库

// 标准库对外提供API
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

// TODO: doc here
// 标准库对用户程序的封装
// 定义库入口 _start
// 将_start编译到.text.entry段中
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    // unsafe {
    //     HEAP.lock()
    //         .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE)
    // }
    exit(main());
    panic!("unreachable after sys_exit!");
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}


// 若链接, bin中不存在main时使用
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

