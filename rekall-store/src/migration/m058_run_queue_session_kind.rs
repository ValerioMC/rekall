//! `058`: the run queue chooses whether the sessions it opens are terminals or chat sessions.

use super::{changeset, exec};

changeset!(RunQueueSessionKind, "058-run-queue-session-kind", |manager| {
    exec(manager, &["ALTER TABLE run_queue ADD COLUMN session_kind VARCHAR(20) NOT NULL DEFAULT 'TERMINAL'"]).await
});
