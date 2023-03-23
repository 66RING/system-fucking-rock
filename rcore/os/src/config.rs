pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_BITS: usize = 12;
// 硬编码整块物理内存的终止物理地址为 0x80800000
// 而起始地址为0x80000000，所以可用内存8Mib
// [ekernel, MEMORY_END)
pub const MEMORY_END: usize = 0x80800000;

pub const CLOCK_FREQ: usize = 12500000;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

/// 特定的物理地址用于访问外设寄存器
/// QEMU中virtio外设总线的MMIO地址为如下内容
pub const MMIO: &[(usize, usize)] = &[
    (0x10001000, 0x1000),
];
