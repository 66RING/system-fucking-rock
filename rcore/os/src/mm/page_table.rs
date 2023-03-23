use super::{
    frame_alloc,
    PhysPageNum,
    FrameTracker,
    VirtPageNum,
    VirtAddr,
    PhysAddr,
    StepByOne
};
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use bitflags::*;

// bitflags crate常用来做bit标志, bitflags!宏可以将一个类型封装成一个标志位集合类型
// 将一个 u8 封装成一个标志位的集合类型，支持一些常见的集合运算
bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

// 防止所有权转移
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    // 根据物理页号和标志位生成页表项
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    // 隐含V标志0 不合法
    pub fn empty() -> Self {
        PageTableEntry {
            bits: 0,
        }
    }
    // 从PTE取值的方法
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        // empty隐含V标志0 不合法
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }

}

// 每个应用的(多级)页表不同
//      root_ppn唯一的分区表示
pub struct PageTable {
    root_ppn: PhysPageNum,
    // FrameTracker绑定到frames，妙用生命周期
    frames: Vec<FrameTracker>,
}

impl PageTable {
    // 分配一个根节点，并挂在frames下
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    // ** key **
    // 找vpn对应的页表项
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            // 根据索引取出每级的pte
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            // 叶子节点时返回
            if i == 2 {
                result = Some(pte);
                break;
            }
            // 中间节点还未创建 则创建
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }



    /// Temporarily used to get arguments from user space.
    // 创建一个专门用来手动查表的PageTable
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&PageTableEntry> = None;
        for i in 0..3 {
            let pte = &ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                // 只是找，找不到None
                return None;
            }
            ppn = pte.ppn(); 
        }
        result
    }
    // 调用find_pte, 找到就拷贝一份返回
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn)
            .map(|pte| {pte.clone()})
    }
    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor())
            .map(|pte| {
                //println!("translate_va:va = {:?}", va);
                let aligned_pa: PhysAddr = pte.ppn().into();
                //println!("translate_va:pa_align = {:?}", aligned_pa);
                let offset = va.page_offset();
                let aligned_pa_usize: usize = aligned_pa.into();
                (aligned_pa_usize + offset).into()
            })
    }
    // 按照satp格式要求构造64位无符号整数，使分页模式为SV39
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

// 内核空间访问用户空间的辅助函数
//      因为地址隔离，内核态要知道用户态数据需要手动查表，故提供此辅助函数
// token: 某个地址空间token
// ptr: 缓冲区地址
// len: 缓冲区长度
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table
            .translate(vpn)
            .unwrap()
            .ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

// 从用户态地址空间拿到字符串，\0判断字符串结束
pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table.translate_va(VirtAddr::from(va)).unwrap().get_mut());
        if ch == 0 {
            break;
        } else {
            string.push(ch as char);
            va += 1;
        }
    }
    string
}

// TODO learn
pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    //println!("into translated_refmut!");
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;
    //println!("translated_refmut: before translate_va");
    page_table.translate_va(VirtAddr::from(va)).unwrap().get_mut()
}

pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }

    /// 返回缓冲区的总长度
    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
}

/// Iterator of `UserBuffer`
pub struct UserBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    current_buffer: usize,
    current_idx: usize,
}

impl Iterator for UserBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_buffer >= self.buffers.len() {
            None
        } else {
            let r = &mut self.buffers[self.current_buffer][self.current_idx] as *mut _;
            if self.current_idx + 1 == self.buffers[self.current_buffer].len() {
                self.current_idx = 0;
                self.current_buffer += 1;
            } else {
                self.current_idx += 1;
            }
            Some(r)
        }
    }
}

















