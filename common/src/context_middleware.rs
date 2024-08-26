use pin_project_lite::pin_project;
use std::task::{Context as TaskContext, Poll};
use tower::Service;

use crate::context::Context;
use crate::with_context::{FutureExt, WithContext};

#[derive(Debug, Clone)]
pub struct ContextService<S> {
    inner: S,
    context: Context,
}

impl<S> ContextService<S> {
    fn new(inner: S, context: Context) -> Self {
        ContextService { inner, context }
    }
}

impl<S, Request> Service<Request> for ContextService<S>
where
    S: Service<Request>,
    Request: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = FutureResponse<S::Future>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        // let _guard = self.context.clone().attach();
        let response_future = self.inner.call(request).with_context(self.context.clone());

        FutureResponse { response_future }
    }
}

pin_project! {
    pub struct FutureResponse<F> {
        #[pin]
        response_future: WithContext<F>,
    }
}

impl<F, Response, Error> std::future::Future for FutureResponse<F>
where
    F: std::future::Future<Output = Result<Response, Error>>,
{
    type Output = Result<Response, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.response_future.poll(cx)
    }
}

#[derive(Debug, Clone)]
pub struct ContextHolder {
    context: Context,
}

impl ContextHolder {
    pub fn new(context: Context) -> Self {
        ContextHolder { context }
    }
}

impl<S> tower::Layer<S> for ContextHolder {
    type Service = ContextService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ContextService::new(inner, self.context.clone())
    }
}
