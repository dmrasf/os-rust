#![allow(unused)]

use crate::config::PAGE_SIZE;
use crate::fs::{make_pipe, open_file, OpenFlags};
use crate::mm::MapPermission;
use crate::{mm::*, task::*};
use alloc::sync::Arc;
use bitflags::bitflags;

const FD_STDOUT: usize = 1;
const FD_STDIN: usize = 0;

#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    /// ID of device containing file
    pub dev: u64,
    /// inode number
    pub ino: u64,
    /// file type and mode
    pub mode: StatMode,
    /// number of hard links
    pub nlink: u32,
    /// unused pad
    pad: [u64; 7],
}

impl Stat {
    pub fn new() -> Self {
        Stat {
            dev: 0,
            ino: 0,
            mode: StatMode::NULL,
            nlink: 0,
            pad: [0; 7],
        }
    }
}

impl Default for Stat {
    fn default() -> Self {
        Self::new()
    }
}

bitflags! {
    pub struct StatMode: u32 {
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let process = current_process();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = process.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let process = current_process();
    let token = current_user_token();
    let mut inner = process.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_linkat(
    old_dirfd: usize,
    old_path: &str,
    new_dirfd: usize,
    new_path: &str,
    flags: usize,
) -> isize {
    0
}

pub fn sys_unlinkat(dirfd: usize, path: &str, flags: usize) -> isize {
    0
}

pub fn sys_fstat(fd: usize, st: &Stat) -> isize {
    0
}

pub fn sys_mail_read(buffer: *mut u8, len: usize) -> isize {
    0
}

pub fn sys_mail_write(pid: usize, buffer: *const u8, len: usize) -> isize {
    0
}

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if prot & !0x7 != 0 {
        return -1;
    }
    if prot & 0x7 == 0 {
        return -1;
    }
    let mut mp = MapPermission::empty();
    if prot & 0x1 != 0 {
        mp |= MapPermission::R;
    }
    if prot & 0x2 != 0 {
        mp |= MapPermission::W;
    }
    if prot & 0x4 != 0 {
        mp |= MapPermission::X;
    }
    let ma = MapArea::new(start.into(), (start + len).into(), MapType::Framed, mp);
    let pt = PageTable::from_token(current_user_token());
    for vpn in ma.vpn_range {
        if pt.translate(vpn).is_none() {
            return -1;
        }
    }
    let process = current_process();
    process.mmap(start.into(), (start + len).into(), mp);
    for vpn in ma.vpn_range {
        if let Some(pte) = pt.translate(vpn) {
            println!("{:?}: {:?}, {:?}", vpn, pte.ppn(), pte.flags());
        }
    }
    0
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let mut ma = MapArea::new(
        start.into(),
        (start + len).into(),
        MapType::Framed,
        MapPermission::empty(),
    );
    let mut pt = PageTable::from_token(current_user_token());
    for vpn in ma.vpn_range {
        if pt.translate(vpn).is_none() {
            return -1;
        }
    }
    let process = current_process();
    process.munmap(start.into(), (start + len).into());
    0
}

pub fn sys_spawn(path: &str) -> isize {
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}
