
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,      // ra记录返回后跳转到哪里
    sp: usize,
    s: [usize; 12], // 函数调用，用s保存被调用者的寄存器。当汇编不过rust编译器，要收到保存
}

impl TaskContext {
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" { fn __restore(); }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }


}
