mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::*;
pub use frame_allocator::*;
pub use memory_set::*;
pub use page_table::*;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    memory_set::KERNEL_SPACE.exclusive_access().activate();
}
