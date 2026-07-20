use core::borrow::Borrow;
use core::panic::{RefUnwindSafe, UnwindSafe};

use derive_more::Deref;

use crate::sync::{WaitGroupData, WaitGroupLayout, WaitGroupLayoutExt};
use crate::twin_ref::{ClonableTwinRefLayout, TwinRef, TwinRefLayout};
use crate::utils::*;

#[derive(Debug)]
pub(crate) struct MonoLayout {
    twin_count: AtomicU8,
    state: AtomicU8,
    data: UnsafeCell<WaitGroupData>,
}

#[cfg(not(loom))]
const _: () = {
    assert!(core::mem::size_of::<MonoLayout>() == core::mem::size_of::<usize>() * 3);
    assert!(core::mem::align_of::<MonoLayout>() == core::mem::size_of::<usize>());
};

unsafe impl Send for MonoLayout {}
unsafe impl Sync for MonoLayout {}
impl UnwindSafe for MonoLayout {}
impl RefUnwindSafe for MonoLayout {}

impl MonoLayout {
    #[inline]
    pub fn new() -> Self {
        Self {
            twin_count: AtomicU8::new(2),
            state: AtomicU8::new(0),
            data: UnsafeCell::new(WaitGroupData::None),
        }
    }
}

#[derive(Debug, Deref)]
pub(crate) struct SharedLayout {
    cloned_count: AtomicUsize,
    #[deref]
    inner: MonoLayout,
}

#[cfg(not(loom))]
const _: () = {
    assert!(core::mem::size_of::<SharedLayout>() == core::mem::size_of::<usize>() * 4);
    assert!(core::mem::align_of::<SharedLayout>() == core::mem::size_of::<usize>());
};

impl SharedLayout {
    #[inline]
    pub fn new() -> Self {
        Self {
            cloned_count: AtomicUsize::new(1),
            inner: MonoLayout::new(),
        }
    }
}

impl Borrow<MonoLayout> for SharedLayout {
    #[inline]
    fn borrow(&self) -> &MonoLayout {
        self
    }
}

impl Borrow<MonoLayout> for TwinRef<SharedLayout> {
    #[inline]
    fn borrow(&self) -> &MonoLayout {
        self
    }
}

unsafe impl<T: Borrow<MonoLayout>> TwinRefLayout for T {
    #[inline]
    fn count(&self) -> &AtomicU8 {
        &self.borrow().twin_count
    }
}

unsafe impl<T: Borrow<MonoLayout>> WaitGroupLayout for T {
    #[inline]
    fn state(&self) -> &AtomicU8 {
        &self.borrow().state
    }

    #[inline]
    unsafe fn slot(&self) -> &UnsafeCell<WaitGroupData> {
        &self.borrow().data
    }
}

unsafe impl<T: Borrow<SharedLayout>> ClonableTwinRefLayout for T {
    #[inline]
    fn cloned_count(&self) -> &AtomicUsize {
        &self.borrow().cloned_count
    }

    #[inline]
    fn action_on_zero(&self) {
        unsafe {
            self.borrow().send_done();
        }
    }
}
