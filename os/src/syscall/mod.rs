const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_GET_TASKINFO: usize = 100;

mod fs;
mod process;
mod task;

use crate::sync::UPSafeCell;
use fs::*;
use lazy_static::*;
use process::*;
use task::*;

lazy_static! {
    static ref SYSCALL_TIMES: UPSafeCell<[usize; 3]> = unsafe { UPSafeCell::new([0; 3]) };
}

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => {
            SYSCALL_TIMES.exclusive_access()[0] += 1;
            trace!(
                "write({}): call times {}",
                syscall_id,
                SYSCALL_TIMES.exclusive_access()[0]
            );
            sys_write(args[0], args[1] as *const u8, args[2])
        }
        SYSCALL_EXIT => {
            SYSCALL_TIMES.exclusive_access()[1] += 1;
            trace!(
                "exit({}): call times {}",
                syscall_id,
                SYSCALL_TIMES.exclusive_access()[1]
            );
            sys_exit(args[0] as i32)
        }
        SYSCALL_GET_TASKINFO => {
            SYSCALL_TIMES.exclusive_access()[2] += 1;
            trace!(
                "get_taskinfo({}): call times {}",
                syscall_id,
                SYSCALL_TIMES.exclusive_access()[2]
            );
            sys_get_taskinfo()
        }
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
