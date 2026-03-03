use core::{
    pin::Pin,
    task::{Context, Poll},
};

use derive_more::Into;
use pin_project_lite::pin_project;

use crate::{GroupToken, MonoGroupToken, group::GroupTokenFactory};

/// Extension trait for futures to automatically release group tokens.
pub trait GroupTokenExt<T>: Sized {
    /// Releases the group token when the future is ready or dropped.
    #[inline]
    fn release_on_ready(self, token: T) -> GroupTokenReleaseOnReady<Self, T> {
        GroupTokenReleaseOnReady {
            inner: self,
            token: Some(token),
        }
    }

    /// Releases the group token when the future is dropped.
    ///
    /// The token is held for the entire lifetime of the future, even if the
    /// future is ready.
    #[inline]
    fn release_on_drop(self, token: T) -> GroupTokenReleaseOnDrop<Self, T> {
        GroupTokenReleaseOnDrop { inner: self, token }
    }
}

/// Extension trait for `FnOnce` to automatically release group tokens.
pub trait GroupTokenFuncExt<T, Output>: Sized {
    /// Releases the group token when the closure returns.
    fn release_on_return(self, token: T) -> impl FnOnce() -> Output + Send;
}

trait GroupTokenType: Sync + Send + 'static {}

impl GroupTokenType for GroupTokenFactory {}
impl GroupTokenType for GroupToken {}
impl GroupTokenType for MonoGroupToken {}

impl<T: GroupTokenType, F: Future> GroupTokenExt<T> for F {}

impl<T: GroupTokenType, Output, F: Send + FnOnce() -> Output> GroupTokenFuncExt<T, Output> for F {
    #[inline]
    fn release_on_return(self, token: T) -> impl FnOnce() -> Output + Send {
        move || {
            let res = (self)();
            drop(token);
            res
        }
    }
}

pin_project! {
    /// Wrapper that releases a token when the future is ready or dropped.
    ///
    /// Created by [`GroupTokenExt::release_on_ready`].
    #[derive(Debug, Into)]
    pub struct GroupTokenReleaseOnReady<F, T> {
        #[pin]
        inner: F,
        token: Option<T>,
    }
}

pin_project! {
    /// Wrapper that releases a token when the future is dropped.
    ///
    /// Created by [`GroupTokenExt::release_on_drop`].
    #[derive(Debug, Into)]
    pub struct GroupTokenReleaseOnDrop<F, T> {
        #[pin]
        inner: F,
        token: T,
    }
}

impl<F, T> GroupTokenReleaseOnDrop<F, T> {
    /// Returns a pinned mutable reference to the inner future.
    #[inline]
    pub fn inner_pin(self: Pin<&mut Self>) -> Pin<&mut F> {
        self.project().inner
    }

    /// Returns a reference to the associated token.
    #[inline]
    pub fn group_token(&self) -> &T {
        &self.token
    }
}

impl<F, T> GroupTokenReleaseOnReady<F, T> {
    /// Returns a pinned mutable reference to the inner future.
    #[inline]
    pub fn inner_pin(self: Pin<&mut Self>) -> Pin<&mut F> {
        self.project().inner
    }

    /// Returns a reference to the associated token if not yet released.
    #[inline]
    pub fn group_token(&self) -> Option<&T> {
        self.token.as_ref()
    }
}

impl<F: Future, T> Future for GroupTokenReleaseOnDrop<F, T> {
    type Output = F::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner_pin().poll(cx)
    }
}

impl<F: Future, T> Future for GroupTokenReleaseOnReady<F, T> {
    type Output = F::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let res = this.inner.poll(cx);
        if res.is_ready() {
            drop(this.token.take());
        }
        res
    }
}
