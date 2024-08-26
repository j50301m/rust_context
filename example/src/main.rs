use api::test::test_service_server::TestServiceServer;
use common::database::Database;
use kgs_tracing::{info, tracing};
use sea_orm::{sqlx, ConnectOptions, TransactionTrait};
use std::{sync::Arc, time::Duration};
use tokio;
use tonic::async_trait;

mod entity;
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_telemetry();

    let sea_db = SeaPostgres::build().await?;
    let sqlx_db = SqlxPostgres::build().await?;
    let cx = common::context::Context::current_with_value(sea_db).with_value(sqlx_db);

    tonic::transport::Server::builder()
        .layer(common::context_middleware::TestLayer::new(cx))
        .layer(kgs_tracing::middlewares::tonic::root_span_builder())
        .layer(kgs_tracing::middlewares::tonic::TracingRecord::default())
        .add_service(TestServiceServer::new(service::TestService::default()))
        .serve("127.0.0.1:12345".parse().unwrap())
        .await?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct SeaPostgres {
    db: Arc<sea_orm::DatabaseConnection>,
}

#[async_trait]
impl Database for SeaPostgres {
    type DatabaseConnection = sea_orm::DatabaseConnection;
    type DatabaseTransaction = sea_orm::DatabaseTransaction;
    type DatabaseError = sea_orm::DbErr;

    #[tracing::instrument]
    async fn build() -> Result<SeaPostgres, Self::DatabaseError> {
        let db_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            "admin", "admin", "localhost", "5432", "test",
        );

        let mut opt = ConnectOptions::new(db_url);
        opt.max_connections(100)
            .min_connections(30)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true);

        let db = sea_orm::Database::connect(opt).await?;

        Ok(SeaPostgres { db: Arc::new(db) })
    }

    #[tracing::instrument]
    async fn create_transaction(&self) -> Result<Self::DatabaseTransaction, Self::DatabaseError> {
        self.db.begin().await
    }

    #[tracing::instrument]
    async fn rollback_transaction(
        transaction: Self::DatabaseTransaction,
    ) -> Result<(), Self::DatabaseError> {
        transaction.rollback().await
    }
    #[tracing::instrument]
    async fn commit_transaction(
        transaction: Self::DatabaseTransaction,
    ) -> Result<(), Self::DatabaseError> {
        transaction.commit().await
    }
}

#[derive(Debug, Clone)]
pub struct SqlxPostgres {
    db: Arc<sqlx::PgPool>,
}
#[async_trait]
impl Database for SqlxPostgres {
    type DatabaseConnection = sqlx::PgPool;
    type DatabaseTransaction = sqlx::Transaction<'static, sqlx::Postgres>;
    type DatabaseError = sqlx::Error;

    async fn build() -> Result<SqlxPostgres, Self::DatabaseError> {
        let db_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            "admin", "admin", "localhost", "5432", "test",
        );

        let db = sqlx::PgPool::connect(&db_url).await?;

        Ok(SqlxPostgres { db: Arc::new(db) })
    }

    async fn create_transaction(&self) -> Result<Self::DatabaseTransaction, Self::DatabaseError> {
        self.db.begin().await
    }

    async fn rollback_transaction(
        transaction: Self::DatabaseTransaction,
    ) -> Result<(), Self::DatabaseError> {
        transaction.rollback().await
    }

    async fn commit_transaction(
        transaction: Self::DatabaseTransaction,
    ) -> Result<(), Self::DatabaseError> {
        transaction.commit().await
    }
}

#[tracing::instrument]
fn init_telemetry() {
    kgs_tracing::TelemetryBuilder::new("context-example")
        .enable_log("http://localhost:3100")
        .enable_metrics("http://localhost:43177")
        .enable_tracing("http://localhost:43177")
        .build();

    // start metrics system CPU and RAM
    kgs_tracing::components::base_metrics::base_metrics("context-example");
    info!("telemetry init success");
}
