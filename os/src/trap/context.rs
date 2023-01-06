use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    /// 内核页表起始物理地址
    pub kernel_satp: usize,
    /// 应用的内核栈栈顶虚拟地址
    pub kernel_sp: usize,
    /// trap_handler 虚拟地址
    pub trap_handler: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        unsafe {
            sstatus::set_spp(SPP::User);
        }
        let sstatus = sstatus::read();
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
