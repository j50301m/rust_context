# Kgs Context

為了簡化 DB transaction 的管理所開發的功能
利用Local Thread Storage 實現類似 go context的功能

## Usage

註冊中間件  
example/src/main.rs

``` rust
    // Start a  service
    tonic::transport::Server::builder()
        .layer(common::context_middleware::ContextHolder::new(cx)) // Register the Context
        .add_service(TestServiceServer::new(service::TestService::default()))
        .serve("127.0.0.1:12345".parse().unwrap())
        .await?;

    // Skip...
```

Case1: 手動管理 Transaction
example/sec/service.rs

``` rust
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
```

Case2: 使用macro管理

``` rust
#[tracing::instrument]  // Use 
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
```

## Quick Start

範例附錄結構

```tree
.
├── api
│   ├── protos
│   └── src
├── migration
│   └── src
└── src
    └── entity
```

1. Migrate up

    ``` bash
    export DATABASE_URL="postgresql://admin:admin@localhost:5432/test"
    sea-orm-cli migrate up
    ```

2. Generate the example entities

    ``` bash
    sea-orm-cli generate entity \
        -u "postgresql://admin:admin@localhost:5432/test" \
        -o src/entity
    ```
