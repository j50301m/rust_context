use std::{any::{Any, TypeId}, collections::HashMap, sync::Arc, task::{Context, Poll}};
use pin_project_lite::pin_project;
use tower::Service;

use crate::{db_manager::Database, with_context::{FutureExt, WithContext}};


#[derive(PartialEq, Debug)]
pub struct TestStruct(pub &'static str);

#[derive(Debug,Clone)]
pub struct TestService<S> {
    inner: S,
    context: crate::context::Context
}

impl<S> TestService<S> {
    fn new(inner: S,context:crate::context::Context) -> Self {
        TestService { inner, context }
    }
}


impl<S,Request> Service<Request> for TestService<S>
where
    S: Service<Request>,
    Request: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TestResponse<S::Future>;


    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request) -> Self::Future
    {
        let _guard = self.context.clone().attach();

        let response_future = self.inner.call(request).with_context(self.context.clone());

        TestResponse {
            response_future,
        }
    }
}

pin_project! {
    pub struct TestResponse<F> {
        #[pin]
        response_future: WithContext<F>,
    }
}

impl <F, Response, Error> std::future::Future for TestResponse<F>
where
    F: std::future::Future<Output = Result<Response, Error>>,
{
    type Output = Result<Response, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.response_future.poll(cx)
    }
}


#[derive(Debug, Clone)]
pub struct TestLayer {
    context: crate::context::Context,
}

impl TestLayer {
    pub fn new(context:crate::context::Context) -> Self {
        TestLayer {
            context,
        }
    }
}

impl<S> tower::Layer<S> for TestLayer {
    type Service = TestService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TestService::new(inner,self.context.clone())
    }
}