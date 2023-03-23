//
// 此处为进入系统后的应用程序环境
// 实现应用程序需要的工具, 相当于标准库
//
#![no_std]
#![feature(asm)]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

// 大部分和内核程序的一样的
#[macro_use]
pub mod console;
mod syscall;
mod lang_items;

use crate::syscall::*;

use buddy_system_allocator::LockedHeap;

const USER_HEAP_SIZE: usize = 16384;
static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

// 标准库对用户程序的封装
// 定义库入口 _start
// 将_start编译到.text.entry段中
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    exit(main());
}

// 若链接, bin中不存在main时使用
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}


// 标准库对外提供API
pub fn write(fd: usize, buf: &[u8]) -> isize { sys_write(fd, buf) }
pub fn exit(exit_code: i32) -> ! { sys_exit(exit_code) }
pub fn yield_() -> isize { sys_yield() }
pub fn get_time() -> isize { sys_get_time() }

pub fn getpid() -> isize { sys_getpid() }
pub fn fork() -> isize { sys_fork() }
pub fn exec(path: &str) -> isize { sys_exec(path) }
// 等待任意子进程结束
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as &mut _) {
            // -2没有退出，让出CPU
            -2 => { yield_();}
            // -1 or real pid
            exit_pid => return exit_pid,
        }
    }
}

// 等待指定pid子进程结束
pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            // os -> sys_waitpid 返回-2表示还没退出, 不停让出，等待
            -2 => { yield_(); }
            // -1 or real pid
            exit_pid => return exit_pid,
        }
    }
}


pub fn read(fd: usize, buf: &mut [u8]) -> isize { sys_read(fd, buf) }

