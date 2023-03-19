use core::any::Any;

/// 块设备接口
pub trait BlockDevice: Send + Sync + Any {
    /// 读取块内容到缓冲区
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// 将缓冲区内容写入块
    fn write_block(&self, block_id: usize, buf: &[u8]);
}
