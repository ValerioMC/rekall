use std::sync::Arc;

use rekall_common::Instant;
use sea_orm::DatabaseConnection;

use crate::{EventBus, Tx};

/// What a clock-driven rule reads the time from. The application reads the system clock; a test
/// can fix it, the way the Java services took a `java.time.Clock`.
pub type Clock = Arc<dyn Fn() -> Instant + Send + Sync>;

pub fn system_clock() -> Clock {
    Arc::new(Instant::now)
}

/// The database and the event bus every service reads and writes through.
#[derive(Clone)]
pub struct Ctx {
    pub db: DatabaseConnection,
    pub events: EventBus,
    pub clock: Clock,
}

impl Ctx {
    pub fn new(db: DatabaseConnection, events: EventBus) -> Self {
        Self { db, events, clock: system_clock() }
    }

    pub fn now(&self) -> Instant {
        (self.clock)()
    }

    pub async fn read(&self) -> rekall_common::Result<Tx> {
        Tx::read(self).await
    }

    pub async fn write(&self) -> rekall_common::Result<Tx> {
        Tx::write(self).await
    }
}
