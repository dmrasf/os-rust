use core::any::Any;

pub trait BlockDevice: Send + Sync + Any {
    /// 把编号为block_id的块从磁盘读入内存区
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// 将内存区数据写入到编号为block_id的磁盘块
    fn write_block(&self, block_id: usize, buf: &[u8]);
}
