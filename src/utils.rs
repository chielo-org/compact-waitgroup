#[cfg(all(not(loom), not(feature = "portable-atomic")))]
pub use core::sync::atomic::{self as _atomic, AtomicU8, AtomicUsize};
#[cfg(loom)]
pub use loom::sync::atomic::{self as _atomic, AtomicU8, AtomicUsize};
#[cfg(all(not(loom), feature = "portable-atomic"))]
pub use portable_atomic::{self as _atomic, AtomicU8, AtomicUsize};

pub mod atomic {
    pub use super::_atomic::Ordering::*;
    pub use super::_atomic::fence;
}

#[cfg(not(loom))]
pub use core::cell::UnsafeCell;
#[cfg(loom)]
pub use loom::cell::UnsafeCell;
