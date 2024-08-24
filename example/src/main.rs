use std::{any::Any, sync::Arc, time::Duration};

use api::test::test_service_server::TestServiceServer;
use common::db_manager::Database;
use sea_orm::{sqlx, ConnectOptions, TransactionTrait};
use tokio;
use tonic::async_trait;

mod service;
mod entity;





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let sea_db = Arc::new(SeaPostgres::build().await?);

    tonic::transport::Server::builder()
        .layer(common::context_middleware::TestLayer::new(sea_db.clone()))
        .add_service(TestServiceServer::new(service::TestService::default()))
        .serve("127.0.0.1:12345".parse().unwrap())
        .await?;

    Ok(())
}

#[derive(Debug,Clone)]
pub struct SeaPostgres{
    db: Arc<sea_orm::DatabaseConnection>,
}
#[async_trait]
impl Database for SeaPostgres {
    type DatabaseConnection = sea_orm::DatabaseConnection;
    type DatabaseTransaction = sea_orm::DatabaseTransaction;
    type DatabaseError = sea_orm::DbErr;

    async fn build() -> Result<SeaPostgres, Self::DatabaseError> {
        let db_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            "admin",
            "admin",
            "localhost",
            "5432",
            "test",
        );

        let mut opt = ConnectOptions::new(db_url);
        opt.max_connections(10)
            .min_connections(2)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true);

        let db = sea_orm::Database::connect(opt).await?;

        Ok(SeaPostgres {
            db:Arc::new(db)
        })
    }

    async fn create_transaction(&self) -> Result<Self::DatabaseTransaction, Self::DatabaseError> {
        self.db.begin().await
    }

    async fn rollback_transaction(&self, transaction: Self::DatabaseTransaction) -> Result<(), Self::DatabaseError> {
        transaction.rollback().await
    }

    async fn commit_transaction(&self, transaction: Self::DatabaseTransaction) -> Result<(), Self::DatabaseError> {
        transaction.commit().await
    }
}


#[derive(Debug)]
pub struct SqlxPostgres{
    db: sqlx::PgPool,
}
#[async_trait]
impl Database for SqlxPostgres {
    type DatabaseConnection = sqlx::PgPool;
    type DatabaseTransaction = sqlx::Transaction<'static, sqlx::Postgres>;
    type DatabaseError = sqlx::Error;

    async fn build() -> Result<SqlxPostgres, Self::DatabaseError> {
        let db_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            "admin",
            "admin",
            "localhost",
            "5432",
            "test",
        );

        let db = sqlx::PgPool::connect(&db_url).await?;

        Ok(SqlxPostgres { db })
    }

    async fn create_transaction(&self) -> Result<Self::DatabaseTransaction, Self::DatabaseError> {
        self.db.begin().await
    }

    async fn rollback_transaction(&self, transaction: Self::DatabaseTransaction) -> Result<(), Self::DatabaseError> {
        transaction.rollback().await
    }

    async fn commit_transaction(&self, transaction: Self::DatabaseTransaction) -> Result<(), Self::DatabaseError> {
        transaction.commit().await
    }
}