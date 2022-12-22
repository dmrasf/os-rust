use crate::config::PAGE_SIZE;
use crate::mm::MapPermission;
use crate::sbi::console_getchar;
use crate::{console, task::*};
use crate::{mm::*, task::*};
use bitflags::bitflags;
use core::arch::asm;

const FD_STDOUT: usize = 1;
const FD_STDIN: usize = 0;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

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

pub fn sys_openat(dirfd: usize, path: &str, flags: u32, mode: u32) -> isize {
    0
}

pub fn sys_close(fd: usize) -> isize {
    0
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            let mut c: usize;
            loop {
                c = console_getchar();
                if c == 0 {
                    suspend_current_and_run_next();
                    continue;
                } else {
                    break;
                }
            }
            let ch = c as u8;
            let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(ch);
            }
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
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

pub fn sys_mail_read(buffer: &mut [u8]) -> isize {
    0
}

pub fn sys_mail_write(pid: usize, buffer: &[u8]) -> isize {
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
        if let Some(pte) = pt.translate(vpn) {
            if (pte.is_valid()) {
                return -1;
            }
        }
    }
    let processor = PROCESSOR.exclusive_access();
    processor
        .current()
        .unwrap()
        .mmap(start.into(), (start + len).into(), mp);
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
        if let Some(pte) = pt.translate(vpn) {
            if (!pte.is_valid()) {
                return -1;
            }
        } else {
            return -1;
        }
    }
    let processor = PROCESSOR.exclusive_access();
    processor
        .current()
        .unwrap()
        .munmap(start.into(), (start + len).into());
    0
}

pub fn sys_spawn(path: &str) -> isize {
    0
}

pub fn sys_dup(fd: usize) -> isize {
    0
}

pub fn sys_pipe(pipe: &mut [usize]) -> isize {
    0
}
