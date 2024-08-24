use std::{any::Any, sync::Arc};

use once_cell::sync::{Lazy, OnceCell};
use tonic::async_trait;

#[async_trait]
pub trait Database: Any + Send + Sync {
    type DatabaseConnection;
    type DatabaseTransaction;
    type DatabaseError;

    async fn build() -> Result<Self,Self::DatabaseError> where Self: Sized;

    async fn create_transaction(&self) -> Result<Self::DatabaseTransaction,Self::DatabaseError>;

    async fn rollback_transaction(&self, transaction: Self::DatabaseTransaction) -> Result<(),Self::DatabaseError>;

    async fn commit_transaction(&self, transaction: Self::DatabaseTransaction) -> Result<(),Self::DatabaseError>;
}


// pub struct DatabaseManager<D:Database> {
//     db: D
// }

// impl<T:Database> DatabaseManager<T> {
//     pub async fn init_db<>(db:T) -> Result<Self,T::DatabaseError> {
//         let db  = T::build().await?;
//         Ok(Self { db })
//     }

//     pub async fn create_transaction<T: Database>(db: &T::DatabaseConnection) -> Result<T::DatabaseTransaction,T::DatabaseError> {
//         T::create_transaction(db).await
//     }

//     pub async fn rollback_transaction<T: Database>(db: &T::DatabaseConnection, transaction: T::DatabaseTransaction) -> Result<T::DatabaseTransaction,T::DatabaseError> {
//         T::rollback_transaction(db, transaction).await
//     }

//     pub async fn commit_transaction<T: Database>(db: &T::DatabaseConnection, transaction: T::DatabaseTransaction) -> Result<T::DatabaseTransaction,T::DatabaseError> {
//         T::commit_transaction(db, transaction).await
//     }
// }
