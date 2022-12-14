use crate::{block_cache::get_block_cache, block_dev::BlockDevice, BLOCK_SZ};
use alloc::sync::Arc;

/// 512字节的比特数
const BLOCK_BITS: usize = BLOCK_SZ * 8;

type BitmapBlock = [u64; 64];

pub struct Bitmap {
    /// 位图开始的块号
    start_block_id: usize,
    /// 位图所占的块数
    blocks: usize,
}

impl Bitmap {
    /// 新建位图
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }

    /// 位图申请bit 返回bit位置
    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>) -> Option<usize> {
        for block_id in 0..self.blocks {
            let pos = get_block_cache(block_id + self.start_block_id, Arc::clone(block_device))
                .lock()
                .modify(0, |bitmap_block: &mut BitmapBlock| {
                    //  512字节的块被解析为BitmapBlock格式
                    if let Some((bits64_pos, inner_pos)) = bitmap_block
                        .iter()
                        .enumerate()
                        .find(|(_, bits64)| **bits64 != u64::MAX)
                        .map(|(bits64_pos, bits64)| (bits64_pos, bits64.trailing_ones() as usize))
                    {
                        bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                        Some(block_id * BLOCK_BITS + bits64_pos * 64 + inner_pos as usize)
                    } else {
                        None
                    }
                });
            if pos.is_some() {
                return pos;
            }
        }
        None
    }

    pub fn dealloc(&self, block_device: &Arc<dyn BlockDevice>, bit: usize) {
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);
        get_block_cache(block_pos + self.start_block_id, Arc::clone(block_device))
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
                bitmap_block[bits64_pos] -= 1u64 << inner_pos;
            });
    }

    /// Get the max number of allocatable blocks
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}

/// (block_pos, bits64_pos, inner_pos)
fn decomposition(mut bit: usize) -> (usize, usize, usize) {
    let block_pos = bit / BLOCK_BITS;
    bit %= BLOCK_BITS;
    (block_pos, bit / 64, bit % 64)
}
