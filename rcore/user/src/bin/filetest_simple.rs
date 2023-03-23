#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, open, read, write, OpenFlags};

#[no_mangle]

pub fn main() -> i32 {
    let test_str = "Hello, world!";
    let filea = "filea\0";
    // 打开文件
    let fd = open(filea, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    // 写入文件
    write(fd, test_str.as_bytes());
    // 关闭文件
    close(fd);

    // 只读打开文件
    let fd = open(filea, OpenFlags::RDONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    let mut buffer = [0u8; 100];
    // 读取写入的内容
    let read_len = read(fd, &mut buffer) as usize;
    close(fd);
    assert_eq!(test_str, core::str::from_utf8(&buffer[..read_len]).unwrap(),);
    println!("file_test passed!");
    0
}
