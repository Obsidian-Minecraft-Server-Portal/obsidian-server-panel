//! Convenience wrapper around sqlx transactions.

use crate::error::DatabaseError;
use crate::types::Pool;

#[cfg(feature = "sqlite")]
type Inner = sqlx::Transaction<'static, sqlx::Sqlite>;
#[cfg(feature = "mysql")]
type Inner = sqlx::Transaction<'static, sqlx::MySql>;
#[cfg(feature = "postgres")]
type Inner = sqlx::Transaction<'static, sqlx::Postgres>;

/// A database transaction.
///
/// Created from a [`Pool`] via [`Transaction::begin()`].
/// Must be explicitly committed with [`Transaction::commit()`];
/// if dropped without committing, the transaction is rolled back
/// automatically.
///
/// # Example
/// ```ignore
/// let mut tx = Transaction::begin(pool).await?;
///
/// sqlx::query("INSERT INTO items (name) VALUES (?)")
///     .bind("widget")
///     .execute(&mut **tx) // deref twice: Transaction -> sqlx::Transaction -> Connection
///     .await?;
///
/// tx.commit().await?;
/// ```
pub struct Transaction {
    inner: Inner,
}

impl Transaction {
    /// Begin a new transaction on the given pool.
    pub async fn begin(pool: &Pool) -> Result<Self, DatabaseError> {
        let tx = pool.begin().await?;
        Ok(Self { inner: tx })
    }

    /// Commit the transaction.
    ///
    /// If this is not called, the transaction is rolled back on drop.
    pub async fn commit(self) -> Result<(), DatabaseError> {
        self.inner
            .commit()
            .await
            .map_err(DatabaseError::TransactionCommitFailed)
    }

    /// Explicitly roll back the transaction.
    ///
    /// This is called automatically on drop, but calling it explicitly
    /// allows you to handle rollback errors.
    pub async fn rollback(self) -> Result<(), DatabaseError> {
        self.inner.rollback().await?;
        Ok(())
    }

}

impl std::ops::Deref for Transaction {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Transaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
