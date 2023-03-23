use riscv::register::sstatus::{self, Sstatus, SPP};
/// Trap Context
#[repr(C)]
pub struct TrapContext {
    /// 0..31: general regs[0..31]
    pub x: [usize; 32],
    /// 32: CSR sstatus      
    pub sstatus: Sstatus,
    /// 33: CSR sepc
    pub sepc: usize,

    // scause/stval: trap的第一时间就保存或使用
    // sstatus/sepc: trap嵌套，故也需要保存

    // 初始化时写入，以后不再修改
    // 内核地址空间token，即内核页表的起始物理地址
    pub kernel_satp: usize,
    // 当前应用的内核栈栈顶
    pub kernel_sp: usize,
    // tra屏handler入口
    pub trap_handler: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) { self.x[2] = sp; }
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        cx.set_sp(sp);
        cx
    }
}
