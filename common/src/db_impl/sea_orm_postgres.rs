use std::{sync::Arc, time::Duration};

use sea_orm::{ConnectOptions, TransactionTrait};
use tonic::async_trait;

use crate::database::Database;


#[derive(Debug, Clone)]
pub struct SeaOrmPostgres {
    db: Arc<sea_orm::DatabaseConnection>,
}

#[async_trait]
impl Database for SeaOrmPostgres {
    type DatabaseConnection = sea_orm::DatabaseConnection;
    type DatabaseTransaction = sea_orm::DatabaseTransaction;
    type DatabaseError = sea_orm::DbErr;

    async fn create_transaction(&self) -> Result<Self::DatabaseTransaction, Self::DatabaseError> {
        self.db.as_ref().begin().await
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

#[derive(Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Into<log::LevelFilter> for LogLevel {
    fn into(self) -> log::LevelFilter {
        match self {
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
        }
    }
}

pub struct SeaPostgresBuilder<'a> {
    db_user: &'a str,
    db_password: &'a str,
    db_host: &'a str,
    db_port: &'a str,
    db_name: &'a str,
    max_connections: u32,
    min_connections: u32,
    connect_timeout: Duration,
    idle_timeout: Duration,
    max_lifetime: Duration,
    sqlx_logging: bool,
    sqlx_logging_level: LogLevel,
}

impl<'a> Default for SeaPostgresBuilder<'a> {
    fn default() -> Self {
        Self {
            db_user: "user",
            db_password: "password",
            db_host: "localhost",
            db_port: "5432",
            db_name: "postgres",
            max_connections: 100,
            min_connections: 5,
            connect_timeout: Duration::from_secs(8),
            idle_timeout: Duration::from_secs(8),
            max_lifetime: Duration::from_secs(8),
            sqlx_logging: false,
            sqlx_logging_level: LogLevel::Info,
        }
    }
}

impl<'a> SeaPostgresBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn db_user(mut self, db_user: &'a str) -> Self {
        self.db_user = db_user;
        self
    }

    pub fn db_password(mut self, db_password: &'a str) -> Self {
        self.db_password = db_password;
        self
    }

    pub fn db_host(mut self, db_host: &'a str) -> Self {
        self.db_host = db_host;
        self
    }

    pub fn db_port(mut self, db_port: &'a str) -> Self {
        self.db_port = db_port;
        self
    }

    pub fn db_name(mut self, db_name: &'a str) -> Self {
        self.db_name = db_name;
        self
    }

    pub fn max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    pub fn min_connections(mut self, min_connections: u32) -> Self {
        self.min_connections = min_connections;
        self
    }

    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }

    pub fn idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.idle_timeout = idle_timeout;
        self
    }

    pub fn max_lifetime(mut self, max_lifetime: Duration) -> Self {
        self.max_lifetime = max_lifetime;
        self
    }

    pub fn sqlx_logging(mut self, sqlx_logging: bool) -> Self {
        self.sqlx_logging = sqlx_logging;
        self
    }

    pub fn sqlx_logging_level(mut self, sqlx_logging_level: LogLevel) -> Self {
        self.sqlx_logging_level = sqlx_logging_level;
        self
    }

    pub async fn build(&self) -> SeaOrmPostgres {
        let db_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.db_user,
            self.db_password,
            self.db_host,
            self.db_port,
            self.db_name
        );

        let mut opt = ConnectOptions::new(db_url);
        opt.max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .connect_timeout(self.connect_timeout)
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime)
            .sqlx_logging(self.sqlx_logging)
            .sqlx_logging_level(self.sqlx_logging_level.into());

        let db = sea_orm::Database::connect(opt).await.expect("connect to db failed");

        SeaOrmPostgres{
            db: Arc::new(db),
        }
    }
}

