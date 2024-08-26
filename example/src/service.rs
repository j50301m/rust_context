use crate::entity;
use common::{database::Database, db_impl::SeaOrmPostgres};
use common::with_context::FutureExt;
use kgs_tracing::tracing;
use macros::transactional;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set};
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
        save_msg_2(request.into_inner().msg)
            .await
            .map(|msg| Response::new(api::test::Message { msg }))
            .map_err(|e| {
                tracing::error!("Failed to save message: {:?}", e);
                tonic::Status::internal("Failed to save message")
            })
    }
}

#[tracing::instrument]
async fn save_msg_1(msg: String) -> Result<String, tonic::Status> {
    // Get the context
    let cx = Context::current();

    // Get the sea_orm database implementation
    let txn = {
        let db = cx
            .get::<SeaOrmPostgres>()
            .expect("the DB struct `SeaPostgres` not found");
        db.create_transaction()
            .await
            .expect("Failed to create transaction")
    };

    // Save txn into the context
    let cx = cx.with_value(txn);

    // Insert a new record
    let entity = entity::hello::ActiveModel {
        name: Set(msg),
        ..Default::default()
    }
    .insert(cx.get::<DatabaseTransaction>().unwrap())
    .await
    .unwrap();

    // Do other CRUD operations with same transaction
    let name = update_msg(entity).with_context(cx.clone()).await.unwrap();

    // Commit the transaction
    SeaOrmPostgres::commit_transaction_in_context(cx)
        .await
        .expect("Failed to commit transaction");

    // Return the response
    Ok(format!("{}", name))
}

#[tracing::instrument]
#[transactional(SeaOrmPostgres)]
async fn save_msg_2(msg: String) -> Result<String, tonic::Status> {
    // Get the context
    let cx = Context::current();

    // Create a transaction
    let txn = cx
        .get::<DatabaseTransaction>()
        .expect("the DB struct `SeaPostgres` not found");

    // Insert a new record
    let entity = entity::hello::ActiveModel {
        name: Set(msg),
        ..Default::default()
    }
    .insert(txn)
    .await
    .expect("Failed to insert");

    // Do other CRUD operations with same transaction
    let name = update_msg(entity).await.expect("Failed to update");

    // Auto commit  when the function is successful

    Ok(format!("{}", name))
}

#[tracing::instrument]
async fn update_msg(entity: entity::hello::Model) -> Result<String, sea_orm::DbErr> {
    // Get the context
    let cx = Context::current();

    // Get the sea_orm database implementation
    let txn = cx
        .get::<DatabaseTransaction>()
        .expect("the DB struct `SeaPostgres` not found");

    // Insert a new record
    let active_model = entity::hello::ActiveModel {
        id: Set(entity.id),
        name: Set("Updated".to_string()),
        ..Default::default()
    };

    let entity = active_model.update(txn).await?;

    // Return the response
    Ok(format!("Saved: {},  id: {}", entity.name, entity.id))
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
