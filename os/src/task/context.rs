use crate::trap::trap_return;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize, // 从__switch返回后执行地址 ret指令执行时的返回地址ra寄存器
    pub sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    /// init task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
