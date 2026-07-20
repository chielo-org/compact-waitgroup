#![cfg(not(loom))]

use core::pin::Pin;

use alloc::boxed::Box;

use crate::tests::utils::{Arc, FutureTestExt, SharedData};
use crate::{GroupTokenExt, MonoWaitGroup, WaitGroup};

#[futures_test::test]
#[cfg(panic = "unwind")]
async fn test_wg_threads_panic() {
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

    let tokens = core::iter::repeat_n(factory.into_token(), 4)
        .map(|h| {
            let (wg, token) = WaitGroup::new();
            async move {
                wg.await;
                panic!();
            }
            .release_on_ready(h)
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
