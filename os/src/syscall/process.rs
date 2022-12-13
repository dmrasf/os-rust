use core::borrow::BorrowMut;

use crate::{
    mm::{
        MapArea, MapPermission, MapType, PageTable, PhysAddr, VirtAddr, VirtPageNum, KERNEL_SPACE,
    },
    task::{current_user_token, exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug, Default)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

impl TimeVal {
    pub fn new() -> Self {
        Self::default()
    }
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    kernel!("Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(time: usize, tz: usize) -> isize {
    let mut pt = PageTable::from_token(current_user_token());
    let pa = VirtAddr::from(time);
    let page_offset = pa.page_offset();
    let vpn = VirtPageNum::from(pa.floor());
    if let Some(pte) = pt.translate(vpn) {
        let timeval_phyaddr: usize = PhysAddr::from(pte.ppn()).0 + page_offset;
        unsafe {
            let t = &mut *(timeval_phyaddr as *mut TimeVal);
            let current_time = get_time_us();
            t.sec = current_time / 1_000_000;
            t.usec = current_time - t.sec * 1_000_000;
        }
    } else {
        panic!("not found physical addr");
    }
    0
}
pub fn sys_getpid() -> isize {
    0
}

pub fn sys_fork() -> isize {
    0
}

pub fn sys_exec(path: &str) -> isize {
    0
}

pub fn sys_waitpid(pid: isize, xstatus: *mut i32) -> isize {
    0
}

pub fn sys_set_priority(prio: isize) -> isize {
    0
}
