use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use derive_more::{Constructor, Deref};

use crate::utils::*;

pub(crate) type WaitGroupData = Option<Waker>;

/// # Safety
///
/// - `state` must be a field exclusively reserved for `WaitGroupType`, and
///   initialized to `0`.
/// - `slot` must be a field exclusively reserved for `WaitGroupType`, and the
///   inner value should be initialized to `None`.
pub(crate) unsafe trait WaitGroupLayout: Sized {
    fn state(&self) -> &AtomicU8;
    unsafe fn slot(&self) -> &UnsafeCell<WaitGroupData>;
}

pub(crate) trait WaitGroupLayoutExt: WaitGroupLayout {
    #[inline]
    fn is_done(&self) -> bool {
        self.state().load(atomic::Acquire) & DONE != 0
    }

    #[inline]
    unsafe fn send_done(&self) {
        let prev_state = self.state().fetch_or(DONE | LOCK, atomic::AcqRel);
        if prev_state & LOCK == 0
            && let Some(waker) = unsafe { with_slot_mut(self, |slot| slot.take()) }
        {
            waker.wake();
        }
    }
}

impl<T: WaitGroupLayout> WaitGroupLayoutExt for T {}

#[must_use]
#[derive(Debug, Constructor, Deref)]
pub(crate) struct WaitGroupWrapper<T: WaitGroupLayout>(T);

const DONE: u8 = 0b01;
const LOCK: u8 = 0b10;

#[inline]
unsafe fn with_slot_mut<T: WaitGroupLayout, R, F: FnOnce(&mut WaitGroupData) -> R>(
    val: &T,
    f: F,
) -> R {
    #[cfg(not(loom))]
    {
        f(unsafe { &mut *val.slot().get() })
    }
    #[cfg(loom)]
    {
        unsafe { val.slot() }
            .get()
            .with(|ptr| f(unsafe { &mut *ptr.cast_mut() }))
    }
}

impl<T: WaitGroupLayout> Future for WaitGroupWrapper<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let prev_state = self.state().fetch_or(LOCK, atomic::Acquire);

        if prev_state & DONE != 0 {
            return Poll::Ready(());
        }

        debug_assert!(prev_state & LOCK == 0);

        let guard = UnlockGuard(self.state());

        let waker = cx.waker();
        unsafe {
            with_slot_mut(&self.0, |slot| {
                match slot {
                    Some(old) if old.will_wake(waker) => {}
                    _ => {
                        *slot = Some(waker.clone());
                    }
                };
            });
        }

        guard.defuse();

        let prev_state = self.state().fetch_and(!LOCK, atomic::AcqRel);
        if prev_state & DONE != 0 {
            drop(unsafe { with_slot_mut(&self.0, |slot| slot.take()) });
            self.state().fetch_or(LOCK, atomic::Release);
            return Poll::Ready(());
        }

        Poll::Pending
    }
}

impl<T: WaitGroupLayout> Drop for WaitGroupWrapper<T> {
    #[inline]
    fn drop(&mut self) {
        let prev_state = self.state().fetch_or(LOCK, atomic::Acquire);
        if prev_state & LOCK == 0
            && let Some(waker) = unsafe { with_slot_mut(&self.0, |slot| slot.take()) }
        {
            drop(waker);
        }
    }
}

#[must_use]
struct UnlockGuard<'a>(&'a AtomicU8);

impl<'a> UnlockGuard<'a> {
    #[inline]
    fn defuse(self) {
        core::mem::forget(self);
    }
}

impl<'a> Drop for UnlockGuard<'a> {
    #[inline]
    fn drop(&mut self) {
        self.0.fetch_and(!LOCK, atomic::AcqRel);
    }
}
