use super::context::TaskContext;
use super::id::*;
use super::process::ProcessControlBlock;
use crate::mm::*;
use crate::sync::UPSafeCell;
use crate::trap::context::TrapContext;
use alloc::sync::{Arc, Weak};
use core::cell::RefMut;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocking,
}

/// 线程
pub struct TaskControlBlock {
    pub process: Weak<ProcessControlBlock>,
    /// 内核栈
    pub kstack: KernelStack,
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    /// trap上下文的物理页号
    pub trap_cx_ppn: PhysPageNum,
    /// 上下文
    pub task_cx: TaskContext,
    /// 状态
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }

    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner_exclusive_access();
        inner.memory_set.token()
    }

    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                })
            },
        }
    }
}
