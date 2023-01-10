mod context;
mod id;
pub mod manager;
mod process;
mod processor;
mod signal;
mod switch;
mod task;

use self::{context::TaskContext, id::TaskUserRes, manager::*, process::ProcessControlBlock};
use crate::fs::{open_file, OpenFlags};
use crate::{board::*, timer::remove_timer};
use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;

pub use processor::*;
pub use signal::*;
pub use task::*;

lazy_static! {
    /// 初始进程
    pub static ref INITPROC: Arc<ProcessControlBlock> = {
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        ProcessControlBlock::new(v.as_slice())
    };
}

/// 加载初始进程
pub fn add_initproc() {
    let _initproc = INITPROC.clone();
    info!("first task initproc loaded");
}

pub fn suspend_current_and_run_next() {
    let task = task_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task);
    schedule(task_cx_ptr);
}

pub const IDLE_PID: usize = 0;

/// 切换线程
pub fn exit_current_and_run_next(exit_code: i32) {
    let task = task_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let process = task.process.upgrade().unwrap();
    let tid = task_inner.res.as_ref().unwrap().tid;

    task_inner.exit_code = Some(exit_code);
    task_inner.res = None;
    drop(task_inner);
    drop(task);

    if tid == 0 {
        let pid = process.getpid();
        if pid == IDLE_PID {
            kernel!("Idle process exit with exit_code {} ...", exit_code);
            if exit_code != 0 {
                crate::board::QEMU_EXIT_HANDLE.exit_failure();
            } else {
                crate::board::QEMU_EXIT_HANDLE.exit_success();
            }
        }
        remove_from_pid2process(pid);
        let mut process_inner = process.inner_exclusive_access();
        process_inner.is_zombie = true;
        process_inner.exit_code = exit_code;
        {
            let mut initproc_inner = INITPROC.inner_exclusive_access();
            for child in process_inner.children.iter() {
                child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }
        let mut recycle_res = Vec::<TaskUserRes>::new();
        for task in process_inner.tasks.iter().filter(|t| t.is_some()) {
            let task = task.as_ref().unwrap();
            remove_inactive_task(Arc::clone(&task));
            let mut task_inner = task.inner_exclusive_access();
            if let Some(res) = task_inner.res.take() {
                recycle_res.push(res);
            }
        }
        drop(process_inner);
        recycle_res.clear();

        let mut process_inner = process.inner_exclusive_access();
        process_inner.children.clear();
        process_inner.memory_set.recycle_data_pages();
        process_inner.fd_table.clear();
        process_inner.tasks.clear();
    }
    drop(process);
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

pub fn block_current_and_run_next() {
    let task = task_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Blocking;
    drop(task_inner);
    schedule(task_cx_ptr);
}

pub fn current_add_signal(signal: SignalFlags) {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.signals |= signal;
}

pub fn check_signals_error_of_current() -> Option<(i32, &'static str)> {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    process_inner.signals.check_error()
}

pub fn remove_inactive_task(task: Arc<TaskControlBlock>) {
    remove_task(Arc::clone(&task));
    remove_timer(Arc::clone(&task));
}
