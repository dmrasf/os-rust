#![no_std]
#![no_main]
#![allow(unused)]
#![feature(alloc_error_handler)]

#[macro_use]
mod console;
mod board;
mod config;
mod lang_items;
mod loader;
mod mm;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

extern crate alloc;
use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() {
    clear_bss();
    mm::init_heap();
    mm::heap_test();
    mm::init_frame_allocator();
    mm::frame_allocator_test();
    mm::map_area_test();
    panic!("test done!");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
