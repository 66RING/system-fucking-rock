//! Synchronization and interior mutability primitives

// 简单封装, 实现单处理(U P)的sync
mod up;

pub use up::UPSafeCell;
