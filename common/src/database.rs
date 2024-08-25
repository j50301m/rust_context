use std::any::Any;

use tonic::async_trait;

use crate::context::Context;

#[async_trait]
pub trait Database: Any + Send + Sync {
    type DatabaseConnection;
    type DatabaseTransaction: Any + Send + Sync;
    type DatabaseError;

    async fn build() -> Result<Self, Self::DatabaseError>
    where
        Self: Sized;

    async fn create_transaction(&self) -> Result<Self::DatabaseTransaction, Self::DatabaseError>;

    async fn rollback_transaction(
        &self,
        transaction: Self::DatabaseTransaction,
    ) -> Result<(), Self::DatabaseError>;

    async fn commit_transaction(
        &self,
        transaction: Self::DatabaseTransaction,
    ) -> Result<(), Self::DatabaseError>;

    async fn create_transaction_in_context(
        &self,
        context: Context,
    ) -> Result<Context, Self::DatabaseError> {
        let txn = self.create_transaction().await?;
        Ok(context.with_value(txn))
    }

    async fn rollback_transaction_in_context(
        &self,
        mut context: Context,
    ) -> Result<Context, Self::DatabaseError> {
        if let Some(txn) = context.try_move_out::<Self::DatabaseTransaction>() {
            self.rollback_transaction(txn).await?;
        }
        Ok(context)
    }

    async fn commit_transaction_in_context(
        &self,
        mut context: Context,
    ) -> Result<Context, Self::DatabaseError> {
        if let Some(txn) = context.try_move_out::<Self::DatabaseTransaction>() {
            self.commit_transaction(txn).await?;
        }
        Ok(context)
    }
}
