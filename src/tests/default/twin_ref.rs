use derive_more::Deref;

use crate::twin_ref::{ClonableTwinRefLayout, TwinRef, TwinRefLayout};
use crate::utils::*;

struct Canary {
    twin_count: AtomicU8,
    data: AtomicU8,
}

impl Canary {
    fn new() -> Self {
        Self {
            twin_count: AtomicU8::new(2),
            data: AtomicU8::new(0),
        }
    }

    fn load(&self) -> u8 {
        self.data.load(atomic::Relaxed)
    }

    fn set(&self, flag: u8) -> u8 {
        self.data.fetch_or(flag, atomic::Relaxed)
    }
}

unsafe impl TwinRefLayout for Canary {
    fn count(&self) -> &AtomicU8 {
        &self.twin_count
    }
}

#[derive(Deref)]
struct Data {
    cloned_count: AtomicUsize,
    twin_count: AtomicU8,
    #[deref]
    canary: TwinRef<Canary>,
}

impl Data {
    fn new(canary: TwinRef<Canary>) -> Self {
        Self {
            cloned_count: AtomicUsize::new(1),
            twin_count: AtomicU8::new(2),
            canary,
        }
    }
}

unsafe impl TwinRefLayout for Data {
    fn count(&self) -> &AtomicU8 {
        &self.twin_count
    }
}

unsafe impl ClonableTwinRefLayout for Data {
    fn cloned_count(&self) -> &AtomicUsize {
        &self.cloned_count
    }

    fn action_on_zero(&self) {
        self.canary.set(2);
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        self.canary.set(1);
    }
}

#[cfg_attr(not(loom), test)]
pub fn test_twin_ref_mono() {
    let (canary, inspector) = TwinRef::new_mono(Canary::new());
    assert_eq!(inspector.load(), 0);

    let (a, b) = TwinRef::new_mono(Data::new(canary));
    assert_eq!(inspector.load(), 0);
    assert_eq!(a.load(), 0);
    assert_eq!(b.load(), 0);

    b.set(4);
    assert_eq!(inspector.load(), 4);
    assert_eq!(a.load(), 4);
    assert_eq!(b.load(), 4);

    drop(b);
    assert_eq!(inspector.load(), 4);
    assert_eq!(a.load(), 4);

    drop(a);
    assert_eq!(inspector.load(), 5);
}

#[cfg_attr(not(loom), test)]
pub fn test_twin_ref_clonable() {
    let (canary, inspector) = TwinRef::new_mono(Canary::new());
    assert_eq!(inspector.load(), 0);

    let (a, b) = TwinRef::new_clonable(Data::new(canary));
    let c = b.clone();
    assert_eq!(inspector.load(), 0);
    assert_eq!(a.load(), 0);
    assert_eq!(b.load(), 0);
    assert_eq!(c.load(), 0);

    b.set(4);
    assert_eq!(inspector.load(), 4);
    assert_eq!(a.load(), 4);
    assert_eq!(b.load(), 4);
    assert_eq!(c.load(), 4);

    drop(c);
    assert_eq!(inspector.load(), 4);
    assert_eq!(a.load(), 4);
    assert_eq!(b.load(), 4);

    drop(b);
    assert_eq!(inspector.load(), 6);
    assert_eq!(a.load(), 6);

    drop(a);
    assert_eq!(inspector.load(), 7);
}
