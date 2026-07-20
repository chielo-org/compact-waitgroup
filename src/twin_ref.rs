//! Adapted from `alloc::sync::Arc`.

use core::borrow::Borrow;
use core::fmt::Debug;
use core::ops::Deref;
use core::panic::{RefUnwindSafe, UnwindSafe};
use core::ptr::NonNull;

use alloc::boxed::Box;
use derive_more::Deref;

use crate::utils::*;

/// # Safety
///
/// - `count` must be a field exclusively reserved for `TwinRefType` and
///   initialized to `2`.
pub(crate) unsafe trait TwinRefLayout {
    fn count(&self) -> &AtomicU8;
}

/// # Safety
///
/// - `cloned_count` must be a field exclusively reserved for
///   `ClonableTwinRefType`, and initialized to `1`.
/// - `action_on_zero` will be called only once just after `cloned_count`
///   reaches zero.
pub(crate) unsafe trait ClonableTwinRefLayout {
    fn cloned_count(&self) -> &AtomicUsize;
    fn action_on_zero(&self);
}

// ThreadSanitizer does not support memory fences. To avoid false positive
// reports in TwinRef use atomic loads for synchronization instead.
#[cfg(tsan)]
macro_rules! acquire {
    ($x:expr) => {
        $x.load(atomic::Acquire)
    };
}
#[cfg(tsan)]
#[allow(unused)]
fn _fence_may_not_be_used() {
    atomic::fence(atomic::Acquire);
}
#[cfg(not(tsan))]
macro_rules! acquire {
    ($_:expr) => {
        atomic::fence(atomic::Acquire)
    };
}

#[must_use]
pub(super) struct TwinRefPtr<T: TwinRefLayout>(NonNull<T>);

unsafe impl<T: TwinRefLayout + Sync + Send> Send for TwinRefPtr<T> {}
unsafe impl<T: TwinRefLayout + Sync + Send> Sync for TwinRefPtr<T> {}
impl<T: TwinRefLayout + RefUnwindSafe> UnwindSafe for TwinRefPtr<T> {}
impl<T: TwinRefLayout + RefUnwindSafe> RefUnwindSafe for TwinRefPtr<T> {}

impl<T: TwinRefLayout> Deref for TwinRefPtr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: TwinRefLayout + Debug> Debug for TwinRefPtr<T> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let inner: &T = self;
        Debug::fmt(inner, f)
    }
}

impl<T: TwinRefLayout> TwinRefPtr<T> {
    #[inline]
    unsafe fn new(data: T) -> TwinRefPtr<T> {
        let data = Box::new(data);
        TwinRefPtr(Box::leak(data).into())
    }

    #[inline]
    unsafe fn dup(&self) -> Self {
        Self(self.0)
    }

    #[inline]
    unsafe fn drop_twin_ref(&mut self) {
        if self.count().fetch_sub(1, atomic::Release) != 1 {
            return;
        }
        acquire!(self.count());
        let raw = self.0.as_ptr();
        let _ = unsafe { Box::from_raw(raw) };
    }
}

#[derive(Debug, Deref)]
pub(crate) struct TwinRef<T: TwinRefLayout>(TwinRefPtr<T>);

#[derive(Debug, Deref)]
pub(crate) struct ClonableTwinRef<T: TwinRefLayout + ClonableTwinRefLayout>(TwinRefPtr<T>);

impl<T: TwinRefLayout> TwinRef<T> {
    #[must_use]
    #[inline]
    pub fn new_mono(data: T) -> (Self, Self) {
        let ptr = unsafe { TwinRefPtr::new(data) };
        (Self(unsafe { ptr.dup() }), Self(ptr))
    }
}

impl<T: TwinRefLayout + ClonableTwinRefLayout> TwinRef<T> {
    #[must_use]
    #[inline]
    pub fn new_clonable(data: T) -> (Self, ClonableTwinRef<T>) {
        let ptr = unsafe { TwinRefPtr::new(data) };
        (Self(unsafe { ptr.dup() }), ClonableTwinRef(ptr))
    }
}

impl<T: TwinRefLayout> Drop for TwinRef<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.0.drop_twin_ref();
        }
    }
}

#[must_use]
struct DropGuard<T: TwinRefLayout + ClonableTwinRefLayout>(TwinRefPtr<T>);

impl<T: TwinRefLayout + ClonableTwinRefLayout> Drop for DropGuard<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.0.drop_twin_ref();
        }
    }
}

impl<T: TwinRefLayout + ClonableTwinRefLayout> Drop for ClonableTwinRef<T> {
    #[inline]
    fn drop(&mut self) {
        if self.cloned_count().fetch_sub(1, atomic::Release) != 1 {
            return;
        }
        acquire!(self.cloned_count());
        let _guard = DropGuard(unsafe { self.dup() });
        self.action_on_zero();
    }
}

impl<T: TwinRefLayout + ClonableTwinRefLayout> Clone for ClonableTwinRef<T> {
    #[inline]
    fn clone(&self) -> Self {
        // Using a relaxed ordering is alright here, as knowledge of the
        // original reference prevents other threads from erroneously deleting
        // the object.
        //
        // As explained in the [Boost documentation][1], Increasing the
        // reference counter can always be done with memory_order_relaxed: New
        // references to an object can only be formed from an existing
        // reference, and passing an existing reference from one thread to
        // another must already provide any required synchronization.
        //
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        let old_size = self.cloned_count().fetch_add(1, atomic::Relaxed);

        if old_size > usize::MAX / 2 {
            panic!("reference count overflow");
        }

        Self(unsafe { self.dup() })
    }
}

impl<T: TwinRefLayout> Borrow<T> for TwinRef<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: TwinRefLayout + ClonableTwinRefLayout> Borrow<T> for ClonableTwinRef<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}
