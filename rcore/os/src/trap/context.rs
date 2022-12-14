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
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) { self.x[2] = sp; }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };
        cx.set_sp(sp);
        cx
    }
}
