//! `065`: how many tasks the run queue works at once. One keeps the queue as it was.

use super::{changeset, exec};

changeset!(ParallelSessions, "065-parallel-sessions", |manager| {
    exec(manager, &["ALTER TABLE run_queue ADD COLUMN parallel_sessions INTEGER NOT NULL DEFAULT 1"]).await
});
