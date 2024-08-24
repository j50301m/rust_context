use std::sync::Arc;

use common::db_manager::Database;
use common::with_context::FutureExt;
use kgs_tracing::tracing;
use sea_orm::{ActiveModelTrait, Set, TryIntoModel};
use tonic::{self, Request, Response};
use crate::{entity, SeaPostgres};



#[derive(Debug, Default)]
pub struct TestService;
use common::context::Context;
use common::context_middleware::TestStruct;

#[tonic::async_trait]
impl api::test::test_service_server::TestService for TestService {
    #[tracing::instrument]
    async fn say_hello(
        &self,
        request: Request<api::test::Message>,
    ) -> Result<Response<api::test::Message>, tonic::Status> {
        // Assign the context to the current thread
        let cx = Context::current().with_value(TestStruct("test"));

        // Call an async function that uses `cx`
        do_something().with_context(cx.clone()).await;

        // Ensure the context remains the same
        assert_eq!(cx.get::<TestStruct>().unwrap().0, "test");

        // Return the response
        Ok(Response::new(api::test::Message {
            msg: format!("Hello, {}!", request.into_inner().msg),
        }))
    }

    #[tracing::instrument]
    async fn save_msg(
        &self,
        request: Request<api::test::Message>,
    ) -> Result<Response<api::test::Message>, tonic::Status> {
        // Get the context
        let cx = Context::current();

        // Get the sea_orm database implementation
        let db = cx.get::<Arc<SeaPostgres>>().expect("the DB struct `SeaPostgres` not found");
        let txn =  db.create_transaction().await.unwrap();
        let msg = request.into_inner().msg;

        // Insert a new record
        let entity = entity::hello::ActiveModel {
            name: Set(msg),
            ..Default::default()
        }.save(&txn).await.unwrap().try_into_model().unwrap();

        // Commit the transaction
        db.commit_transaction(txn).await.unwrap();

        // Return the response
        Ok(Response::new(api::test::Message {
            msg: format!("Saved: {}", entity.id),
        }))
    }
}


async fn do_something() {
    // Check the context with the value `TestStruct("test")`
    let cx = Context::current().with_value(TestStruct("test"));
    assert_eq!(cx.get::<TestStruct>().unwrap().0, "test");

    // Simulate some async work
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    //  Modify the context
    let cx = cx.with_value(TestStruct("test2"));
    assert_eq!(cx.get::<TestStruct>().unwrap().0, "test2");

    // Check the context with the value `TestStruct("test2")`
    assert_eq!(cx.get::<TestStruct>().unwrap().0, "test2");
}