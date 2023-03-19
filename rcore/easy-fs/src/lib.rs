//!An file system that isolated from the kernel
#![no_std]
#![deny(missing_docs)]

extern crate alloc;
/// Size of each block in bytes
pub const BLOCK_SZ: usize = 512;

mod bitmap;
mod block_cache;
mod block_dev;
mod layout;
mod efs;
mod vfs;

pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
pub use vfs::Inode;
use block_cache::{block_cache_sync_all, get_block_cache};
use bitmap::Bitmap;
use layout::*;



