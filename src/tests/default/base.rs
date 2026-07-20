use core::pin::Pin;
use core::task::{Context, Poll};

use futures_test::task::new_count_waker;

use crate::{MonoWaitGroup, WaitGroup};

#[cfg_attr(not(loom), test)]
pub fn test_wg_done() {
    let (waker, counter) = new_count_waker();
    let mut cx = Context::from_waker(&waker);
    let (wg, token) = WaitGroup::new();
    let mut rx = core::pin::pin!(wg);
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
    token.release();
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
    assert_eq!(counter.get(), 1);
}

#[cfg_attr(not(loom), test)]
pub fn test_wg_done_twice() {
    let (waker, counter) = new_count_waker();
    let mut cx = Context::from_waker(&waker);
    let (wg, token) = WaitGroup::new();
    let (token_a, token_b) = token.scope(|token| (token.clone(), token));
    let mut rx = core::pin::pin!(wg);
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
    token_b.release();
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
    token_a.release();
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
    assert_eq!(counter.get(), 1);
}

#[cfg_attr(not(loom), test)]
pub fn test_wg_done_twice_rev() {
    let (waker, counter) = new_count_waker();
    let mut cx = Context::from_waker(&waker);
    let (wg, token) = WaitGroup::new();
    let (token_a, token_b) = token.scope(|token_a| {
        let token_b = token_a.clone();
        (token_a, token_b)
    });
    let mut rx = core::pin::pin!(wg);
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
    token_b.release();
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
    token_a.release();
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
    assert_eq!(counter.get(), 1);
}

#[cfg_attr(not(loom), test)]
pub fn test_mono_wg_done() {
    let (waker, counter) = new_count_waker();
    let mut cx = Context::from_waker(&waker);
    let (wg, token) = MonoWaitGroup::new();
    let mut rx = core::pin::pin!(wg);
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
    token.release();
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
    assert_eq!(counter.get(), 1);
}

#[cfg_attr(not(loom), test)]
pub fn test_wg_send_before_poll() {
    let (waker, counter) = new_count_waker();
    let mut cx = Context::from_waker(&waker);
    let (wg, token) = WaitGroup::new();
    token.release();
    let mut rx = core::pin::pin!(wg);
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
    assert_eq!(counter.get(), 0);
}

#[cfg_attr(not(loom), test)]
pub fn test_mono_wg_send_before_poll() {
    let (waker, counter) = new_count_waker();
    let mut cx = Context::from_waker(&waker);
    let (wg, token) = MonoWaitGroup::new();
    token.release();
    let mut rx = core::pin::pin!(wg);
    assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
    assert_eq!(counter.get(), 0);
}

#[cfg_attr(not(loom), test)]
pub fn test_wg_drop_before_send() {
    let (wg, token) = WaitGroup::new();
    drop(wg);
    token.release();
}

#[cfg_attr(not(loom), test)]
pub fn test_mono_wg_drop_before_send() {
    let (wg, token) = MonoWaitGroup::new();
    drop(wg);
    token.release();
}

#[cfg_attr(not(loom), test)]
pub fn test_wg_poll_by_others() {
    let (waker_a, counter_a) = new_count_waker();
    let (waker_b, counter_b) = new_count_waker();

    let (wg, token) = WaitGroup::new();
    let mut wg = core::pin::pin!(wg);

    let mut cx = Context::from_waker(&waker_a);
    assert_eq!(wg.as_mut().poll(&mut cx), Poll::Pending);
    assert_eq!(counter_a.get(), 0);

    let mut cx = Context::from_waker(&waker_b);
    assert_eq!(wg.as_mut().poll(&mut cx), Poll::Pending);
    assert_eq!(counter_a.get(), 0);
    assert_eq!(counter_b.get(), 0);

    token.release();

    assert_eq!(counter_a.get(), 0);
    assert_eq!(counter_b.get(), 1);

    assert_eq!(wg.as_mut().poll(&mut cx), Poll::Ready(()));
    assert_eq!(wg.as_mut().poll(&mut cx), Poll::Ready(()));

    assert_eq!(counter_a.get(), 0);
    assert_eq!(counter_b.get(), 1);
}

#[cfg_attr(not(loom), test)]
pub fn test_wg_drop_early() {
    let (waker, counter) = new_count_waker();
    let mut cx = Context::from_waker(&waker);

    let (mut wg, token) = WaitGroup::new();
    let pinned_wg = Pin::new(&mut wg);
    assert_eq!(pinned_wg.poll(&mut cx), Poll::Pending);

    drop(wg);

    token.release();
    assert_eq!(counter.get(), 0);
}
