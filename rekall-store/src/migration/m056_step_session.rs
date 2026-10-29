//! `056`: a step may open a fresh session instead of joining the one running; false keeps a
//! task's steps in one session.

use super::{changeset, exec};

changeset!(StepSession, "056-step-session", |manager| {
    exec(manager, &["ALTER TABLE task_step ADD COLUMN starts_session BOOLEAN NOT NULL DEFAULT 0"]).await
});
