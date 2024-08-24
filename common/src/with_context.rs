use futures_core::stream::Stream;
use futures_sink::Sink;
use crate::context::Context;
use std::pin::Pin;
use std::task::Poll;
use std::task::Context as TaskContext;

use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug,Clone)]
    pub struct WithContext<T> {
        #[pin]
        inner: T,
        context: Context,
    }
}



impl <T:std::future::Future> std::future::Future for WithContext<T> {
    type Output = T::Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut TaskContext<'_>) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let _guard = this.context.clone().attach();
        this.inner.poll(cx)
    }
}

impl <T:Stream> Stream for WithContext<T> {
    type Item = T::Item;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut TaskContext<'_>) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let _guard = this.context.clone().attach();
        this.inner.poll_next(cx)
    }
}

impl<I,T:Sink<I>> Sink<I> for WithContext<T>
where T:Sink<I> {
    type Error = T::Error;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let _guard = this.context.clone().attach();
        T::poll_ready(this.inner, cx)
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: I
    ) -> Result<(), Self::Error> {
        let this = self.project();
        let _guard = this.context.clone().attach();
        T::start_send(this.inner, item)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let _guard = this.context.clone().attach();
        T::poll_flush(this.inner, cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let _guard = this.context.clone().attach();
        T::poll_close(this.inner, cx)
    }
}

pub trait FutureExt: Sized {
    fn with_context(self, context: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            context,
        }
    }

    fn with_current_context(self) -> WithContext<Self> {
        let context = Context::current();
        self.with_context(context)
    }
}


impl<T:Sized> FutureExt for T {}
