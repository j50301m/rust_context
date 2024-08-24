use std::{any::Any, sync::Arc, task::{Context, Poll}};
use pin_project_lite::pin_project;
use tower::Service;

use crate::{db_manager::Database, with_context::{FutureExt, WithContext}};


#[derive(PartialEq, Debug)]
pub struct TestStruct(pub &'static str);

#[derive(Debug,Clone)]
pub struct TestService<S,D> {
    inner: S,
    db_resources: Arc<D>
}

impl<S,D> TestService<S,D> {
    fn new(inner: S,db_resources:Arc<D>) -> Self {
        TestService { inner,db_resources }
    }
}


impl<S,Request,D> Service<Request> for TestService<S,D>
where
    D:Send +Sync+'static,
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
        let  cx = crate::context::Context::current().with_value(self.db_resources.clone());

        let _guard = cx.clone().attach();

        let response_future = self.inner.call(request).with_context(cx);

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
pub struct TestLayer<D> {
    db_resources: Arc<D>,
}

impl<D> TestLayer<D> {
    pub fn new(db_resources:Arc<D>) -> Self {
        TestLayer {
            db_resources,
        }
    }
}

impl<S,D> tower::Layer<S> for TestLayer<D> {
    type Service = TestService<S,D>;

    fn layer(&self, inner: S) -> Self::Service {
        TestService::new(inner,self.db_resources.clone())
    }
}