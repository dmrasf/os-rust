mod context;
pub mod manager;
mod pid;
mod processor;
mod switch;
mod task;

use self::{
    context::TaskContext,
    manager::add_task,
    processor::{schedule, task_current_task},
    task::{TaskControlBlock, TaskStatus},
};
use crate::{
    fs::{open_file, OpenFlags},
    loader::get_app_data_by_name,
};
use alloc::sync::Arc;
use lazy_static::lazy_static;

pub use processor::*;

lazy_static! {
    /// 初始进程
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}

/// 加载初始进程
pub fn add_initproc() {
    add_task(INITPROC.clone());
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

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = task_current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.task_status = TaskStatus::Zombie;
    inner.exit_code = exit_code;
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    inner.children.clear();
    inner.memory_set.recycle_data_pages();
    drop(inner);
    drop(task);
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}
