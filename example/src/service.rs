use crate::{entity, SeaPostgres};
use common::database::Database;
use common::with_context::FutureExt;
use kgs_tracing::tracing;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set, TryIntoModel};
use tonic::{self, Request, Response};

#[derive(Debug, Default)]
pub struct TestService;
use common::context::Context;

#[derive(Debug, PartialEq)]
struct ValueA(&'static str);

#[tonic::async_trait]
impl api::test::test_service_server::TestService for TestService {
    #[tracing::instrument]
    async fn say_hello(
        &self,
        request: Request<api::test::Message>,
    ) -> Result<Response<api::test::Message>, tonic::Status> {
        // Assign the context to the current thread
        let cx = Context::current().with_value(ValueA("test"));

        // Call an async function that uses `cx`
        do_something().with_context(cx.clone()).await;

        // Ensure the context remains the same
        assert_eq!(cx.get::<ValueA>(), Some(&ValueA("test")));

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
        let db = cx
            .get::<SeaPostgres>()
            .expect("the DB struct `SeaPostgres` not found");

        // Get the sea_orm database implementation
        let cx = db
            .create_transaction_in_context(cx.clone())
            .await
            .expect("Failed to create transaction");

        // Do CRUD operation
        let msg = request.into_inner().msg;
        let res = save_msg(msg).with_context(cx.clone()).await.unwrap();

        // Commit the transaction
        db.commit_transaction_in_context(cx)
            .await
            .expect("Failed to commit transaction");

        // Return the response
        Ok(Response::new(api::test::Message {
            msg: format!("Saved: {}", res),
        }))
    }
}

#[tracing::instrument]
async fn save_msg(msg: String) -> Result<String, sea_orm::DbErr> {
    // Get the context
    let cx = Context::current();

    // Get the sea_orm database implementation
    let txn = cx
        .get::<DatabaseTransaction>()
        .expect("the DB struct `SeaPostgres` not found");

    // Insert a new record
    let entity = entity::hello::ActiveModel {
        name: Set(msg),
        ..Default::default()
    }
    .save(txn)
    .await
    .unwrap()
    .try_into_model()
    .unwrap();

    // Return the response
    Ok(format!("Saved: {}", entity.id))
}

#[tracing::instrument]
async fn do_something() {
    // Check the context with the value `TestStruct("test")`
    let cx = Context::current().with_value(ValueA("test"));
    assert_eq!(cx.get::<ValueA>().unwrap().0, "test");

    // Simulate some async work
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    //  Modify the context
    let cx = cx.with_value(ValueA("test2"));
    assert_eq!(cx.get::<ValueA>().unwrap().0, "test2");

    // Check the context with the value `TestStruct("test2")`
    assert_eq!(cx.get::<ValueA>().unwrap().0, "test2");
}
