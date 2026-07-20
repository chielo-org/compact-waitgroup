use core::pin::Pin;

use alloc::boxed::Box;

use crate::tests::utils::{Arc, FutureTestExt, SharedData};
use crate::{GroupTokenExt, MonoWaitGroup, WaitGroup};

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_wg_await_background() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, token) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();
    assert!(!inspector.load());
    token.release();
    bg_wg.await;
    assert!(inspector.load());
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_wg_await_background_twice() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, token) = WaitGroup::new();
    let (token_a, token_b) = token.scope(|token| (token.clone(), token));
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();
    assert!(!inspector.load());
    token_a.release();
    for _ in 0..100 {
        assert!(!inspector.load());
    }
    token_b.release();
    bg_wg.await;
    assert!(inspector.load());
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_wg_await_background_twice_rev() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, token) = WaitGroup::new();
    let (token_a, token_b) = token.scope(|token| (token.clone(), token));
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();
    assert!(!inspector.load());
    token_b.release();
    for _ in 0..100 {
        assert!(!inspector.load());
    }
    token_a.release();
    bg_wg.await;
    assert!(inspector.load());
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_mono_wg_await_background() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, token) = MonoWaitGroup::new();
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();
    assert!(!inspector.load());
    token.release();
    bg_wg.await;
    assert!(inspector.load());
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_mono_wg_pinned_drop_in_another_thread() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_token) = MonoWaitGroup::new();
    let (mut wg, token) = MonoWaitGroup::new();
    async move {
        let wg_pin = Pin::new(&mut wg);
        wg_pin.await;
        async move {
            drop(wg);
        }
        .run_in_background();
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();
    assert!(!inspector.load());
    token.release();
    bg_wg.await;
    assert!(inspector.load());
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_wg_await() {
    let (wg, token) = WaitGroup::new();
    token.release();
    wg.await;
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_wg_await_multiple_repeat_n() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, factory) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();

    assert!(!inspector.load());

    for token in core::iter::repeat_n(factory.into_token(), 100) {
        assert!(!inspector.load());
        token.release();
    }

    bg_wg.await;
    assert!(inspector.load());
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_wg_await_multiple_repeat_with() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, factory) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();

    assert!(!inspector.load());

    for token in factory
        .scope(|token| core::iter::repeat_with(move || token.clone()))
        .take(100)
    {
        assert!(!inspector.load());
        token.release();
    }

    bg_wg.await;
    assert!(inspector.load());
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_wg_await_pin_multiple_repeat_n() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (mut bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, factory) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();

    assert!(!inspector.load());

    for token in core::iter::repeat_n(factory.into_token(), 100) {
        assert!(!inspector.load());
        token.release();
    }

    let mut bg_wg = Pin::new(&mut bg_wg);
    bg_wg.as_mut().await;
    assert!(inspector.load());
    assert!(bg_wg.is_done());
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_wg_await_pin_multiple_repeat_with() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (mut bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, factory) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();

    assert!(!inspector.load());

    for token in factory
        .scope(|token| core::iter::repeat_with(move || token.clone()))
        .take(100)
    {
        assert!(!inspector.load());
        token.release();
    }

    let mut bg_wg = Pin::new(&mut bg_wg);
    bg_wg.as_mut().await;
    assert!(inspector.load());
    assert!(bg_wg.is_done());
}

#[cfg(not(loom))]
const _TESTING_THREADS: usize = 16;
#[cfg(loom)]
const _TESTING_THREADS: usize = 2;

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_mono_wg_await_pin_multiple_threads() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (mut bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, factory) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();

    assert!(!inspector.load());

    let tokens = core::iter::repeat_n(factory.into_token(), _TESTING_THREADS)
        .map(|t| {
            let (wg, token) = MonoWaitGroup::new();
            async move {
                wg.await;
            }
            .release_on_ready(t)
            .run_in_background();
            token
        })
        .collect::<Box<[_]>>();

    assert!(!inspector.load());
    drop(tokens);

    let mut bg_wg = Pin::new(&mut bg_wg);
    bg_wg.as_mut().await;
    assert!(inspector.load());
    assert!(bg_wg.is_done());
}

#[cfg_attr(not(loom), futures_test::test)]
pub async fn test_wg_await_pin_multiple_threads() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (mut bg_wg, bg_token) = MonoWaitGroup::new();
    let (wg, factory) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
    }
    .release_on_ready(bg_token)
    .run_in_background();

    assert!(!inspector.load());

    let tokens = core::iter::repeat_n(factory.into_token(), _TESTING_THREADS)
        .map(|t| {
            let (wg, factory) = WaitGroup::new();
            async move {
                wg.await;
            }
            .release_on_ready(t)
            .run_in_background();
            factory.into_token()
        })
        .collect::<Box<[_]>>();

    assert!(!inspector.load());
    drop(tokens);

    let mut bg_wg = Pin::new(&mut bg_wg);
    bg_wg.as_mut().await;
    assert!(inspector.load());
    assert!(bg_wg.is_done());
}
