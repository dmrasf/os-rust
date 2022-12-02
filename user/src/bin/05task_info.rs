#![no_std]
#![no_main]

use user_lib::get_taskinfo;

#[no_mangle]
fn main() -> i32 {
    get_taskinfo();
    0
}
