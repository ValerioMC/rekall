//! A transaction, and the events it will announce once it commits.

use rekall_kernel::{RekallError, Result};
use sea_orm::{DatabaseTransaction, SqliteTransactionMode, TransactionOptions, TransactionTrait};

use crate::{Ctx, DomainEvent, EventBus, PublishedEvent};

pub struct Tx {
    txn: DatabaseTransaction,
    events: Vec<PublishedEvent>,
    bus: EventBus,
}

impl Tx {
    /// A deferred transaction, which in WAL mode reads one consistent snapshot and never waits
    /// for a writer.
    pub async fn read(ctx: &Ctx) -> Result<Self> {
        Self::begin(ctx, SqliteTransactionMode::Deferred).await
    }

    /// An immediate transaction, which takes the write lock up front (waiting for another writer
    /// to finish) so that a read made inside it cannot be invalidated by a write committed before
    /// this one's first write.
    pub async fn write(ctx: &Ctx) -> Result<Self> {
        Self::begin(ctx, SqliteTransactionMode::Immediate).await
    }

    async fn begin(ctx: &Ctx, mode: SqliteTransactionMode) -> Result<Self> {
        let txn = ctx
            .db
            .begin_with_options(TransactionOptions {
                sqlite_transaction_mode: Some(mode),
                ..Default::default()
            })
            .await
            .map_err(RekallError::from)?;
        Ok(Self { txn, events: Vec::new(), bus: ctx.events.clone() })
    }

    pub fn db(&self) -> &DatabaseTransaction {
        &self.txn
    }

    /// Record an event to announce after the commit.
    pub fn publish(&mut self, event: impl DomainEvent) {
        self.events.push(std::sync::Arc::new(event));
    }

    pub async fn commit(self) -> Result<()> {
        self.txn.commit().await.map_err(RekallError::from)?;
        for event in self.events {
            self.bus.announce(event);
        }
        Ok(())
    }

    pub async fn rollback(self) -> Result<()> {
        self.txn.rollback().await.map_err(RekallError::from)
    }

    /// Commit on success, roll back on failure, and hand the result through.
    pub async fn finish<T>(self, result: Result<T>) -> Result<T> {
        match result {
            Ok(value) => {
                self.commit().await?;
                Ok(value)
            }
            Err(error) => {
                if let Err(rollback_failure) = self.rollback().await {
                    tracing::error!(%rollback_failure, cause = %error, "Rollback failed");
                }
                Err(error)
            }
        }
    }
}

/// Runs `body` in a write transaction of its own, committed on `Ok` and rolled back on `Err`.
#[macro_export]
macro_rules! in_write {
    ($ctx:expr, |$tx:ident| $body:expr) => {{
        #[allow(unused_mut)]
        let mut $tx = $crate::Tx::write($ctx).await?;
        let result: rekall_kernel::Result<_> = async { $body }.await;
        $tx.finish(result).await
    }};
}

/// Runs `body` in a read transaction of its own.
#[macro_export]
macro_rules! in_read {
    ($ctx:expr, |$tx:ident| $body:expr) => {{
        #[allow(unused_mut)]
        let mut $tx = $crate::Tx::read($ctx).await?;
        let result: rekall_kernel::Result<_> = async { $body }.await;
        $tx.finish(result).await
    }};
}
