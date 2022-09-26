//! Synchronization and interior mutability primitives

// TODO: 为何需要封装成UPSafeCell, 而不直接用RefCell
mod up;

pub use up::UPSafeCell;
