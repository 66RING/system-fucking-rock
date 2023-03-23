mod heap_allocator;
mod address;
mod frame_allocator;
mod page_table;
mod memory_set;

use page_table::{PageTable, PTEFlags};
use address::{VPNRange, StepByOne};
pub use address::{PhysAddr, VirtAddr, PhysPageNum, VirtPageNum};
pub use frame_allocator::{FrameTracker, frame_alloc};
pub use page_table::{
    PageTableEntry,
    translated_byte_buffer,
    translated_str,
    translated_refmut,
};
pub use memory_set::{MemorySet, KERNEL_SPACE, MapPermission};
pub use memory_set::remap_test;



pub fn init() {
    // 动态内存分配器
    heap_allocator::init_heap();
    // 物理页帧管理器
    // 与heap_allocator的区别
    //  heap_allocator用于使能rust的堆分配, 如只能指针, Vec等
    //  frame_allocator是我们系统用于申请和分配页内存的本体
    frame_allocator::init_frame_allocator();
    // 创建内核空间，开启分页
    KERNEL_SPACE.exclusive_access().activate();
}
