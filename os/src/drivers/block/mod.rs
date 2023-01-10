mod virtio_blk;

use crate::board::BlockDeviceImpl;
use alloc::sync::Arc;
use easy_fs::BlockDevice;
use lazy_static::*;
pub use virtio_blk::VirtIOBlock;

lazy_static! {
    /// 块设备
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}
