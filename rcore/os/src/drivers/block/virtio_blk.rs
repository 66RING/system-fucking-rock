use super::BlockDevice;
use crate::mm::{
    frame_alloc, frame_dealloc, kernel_token, FrameTracker, PageTable, PhysAddr, PhysPageNum,
    StepByOne, VirtAddr,
};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::*;
use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};

/// QEMU virtio的MMIO地址
const VIRTIO0: usize = 0x10001000;

pub struct VirtioHal;

pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<'static, VirtioHal>>);

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        // VirtIOHeader代表一组寄存器
        // 将MMIO空间转换成寄存器
        unsafe {
            Self(UPSafeCell::new(
                VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
            ))
        }
    }
}

/// 实现设备接口trait接入文件系统
impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .exclusive_access()
            .read_block(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .exclusive_access()
            .write_block(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { UPSafeCell::new(Vec::new()) };
}

/// VritIO使用的环形队列传徐消息
/// 需要用到物理内存的分配额和回收
/// virtio-drivers声明了Hal trait相关接口
/// 在共享内存区域放置VirtQueue
impl Hal for VirtioHal {
    /// 分配连续的物理帧
    fn dma_alloc(pages: usize) -> usize {
        let mut ppn_base = PhysPageNum(0);
        for i in 0..pages {
            // frame_alloc不能保证分配的物理帧连续
            // 但是因为这个阶段只发生在内核初始化阶段
            // 所以可保证连续
            let frame = frame_alloc().unwrap();
            if i == 0 {
                ppn_base = frame.ppn;
            }
            assert_eq!(frame.ppn.0, ppn_base.0 + i);
            QUEUE_FRAMES.exclusive_access().push(frame);
        }
        let pa: PhysAddr = ppn_base.into();
        pa.0
    }

    /// 回收连续的物理帧
    fn dma_dealloc(pa: usize, pages: usize) -> i32 {
        let pa = PhysAddr::from(pa);
        let mut ppn_base: PhysPageNum = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
        0
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        PageTable::from_token(kernel_token())
            .translate_va(VirtAddr::from(vaddr))
            .unwrap()
            .0
    }
}



