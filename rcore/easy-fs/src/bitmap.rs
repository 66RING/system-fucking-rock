use crate::block_cache::get_block_cache;
use alloc::sync::Arc;
use super::BLOCK_SZ;
use super::block_dev::BlockDevice;

/// Number of bits.
const BLOCK_BITS: usize = BLOCK_SZ * 8;

/// 整个数组有4096个bits
/// 以此为单位进行管理
/// TODO: hard code
type BitmapBlock = [u64; 64];

/// TODO: review
/// 如何管理的
pub struct Bitmap {
    // 位图区域开始的块号
    // 对应的bit等于offset + start_block_id
    start_block_id: usize,
    // 位图区域占用的块数
    blocks: usize,
}

impl Bitmap {
    /// A new bitmap from start block id and number of blocks
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }

    /// ⭐分配一个bit
    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>) -> Option<usize> {
        // 遍历整个bitmap, 找到一个未分配的bit
        for block_id in 0..self.blocks {
            // 获取bitmap的缓存
            // 对应bitmap所在位置为start_block_id + block_id
            let pos = get_block_cache(
                self.start_block_id + block_id,
                Arc::clone(block_device),
            )
            .lock()
            // 从当前块的0开始读取数据，读取整个块
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                // 遍历bitmap组中的每一个u64
                // 如果u64不是MAX则有空余bit
                if let Some((bits64_pos, inner_pos)) = bitmap_block
                    .iter()
                    .enumerate()
                    .find(|(_, bits64)| **bits64 != u64::MAX)
                    .map(|(bits64_pos, bits64)| {
                        (bits64_pos, bits64.trailing_ones() as usize)
                    }) {
                    // 返回bit的在位图中的索引
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

    /// 释放一个bit
    pub fn dealloc(&self, block_device: &Arc<dyn BlockDevice>, bit: usize) {
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);
        get_block_cache(
            self.start_block_id + block_pos,
            Arc::clone(block_device),
        ).lock().modify(0, |bitmap_block: &mut BitmapBlock| {
            // 检测确实是以前分配的bit
            assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
            // 将bit置为0
            bitmap_block[bits64_pos] -= 1u64 << inner_pos;
        });
    }

    /// Get the max number of allocatable blocks
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}

/// 将bit编号分解成块编号、块内组编号和组内bit编号
fn decomposition(mut bit: usize) -> (usize, usize, usize){
    let block_pos = bit / BLOCK_BITS;
    bit = bit % BLOCK_BITS;
    (block_pos, bit / 64, bit % 64)
}





