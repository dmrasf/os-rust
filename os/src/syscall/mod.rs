const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_GET_TASKINFO: usize = 100;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

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
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => {
            // SYSCALL_TIMES.exclusive_access()[1] += 1;
            // trace!(
            //     "exit({}): call times {}",
            //     syscall_id,
            //     SYSCALL_TIMES.exclusive_access()[1]
            // );
            sys_exit(args[0] as i32)
        }
        SYSCALL_GET_TASKINFO => sys_get_taskinfo(),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
