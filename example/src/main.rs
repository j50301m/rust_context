use api::test::test_service_server::TestServiceServer;
use common::{context::Context, database::Database};
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

    let db = common::db_impl::SeaPostgresBuilder::default()
        .db_name("test")
        .db_password("admin")
        .db_user("admin")
        .build().await;
    
    let cx = Context::current().with_value(db);

    tonic::transport::Server::builder()
        .layer(common::context_middleware::TestLayer::new(cx))
        .layer(kgs_tracing::middlewares::tonic::root_span_builder())
        .layer(kgs_tracing::middlewares::tonic::TracingRecord::default())
        .add_service(TestServiceServer::new(service::TestService::default()))
        .serve("127.0.0.1:12345".parse().unwrap())
        .await?;

    Ok(())
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
