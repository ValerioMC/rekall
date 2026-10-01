//! `029`: a step keeps the passes it was sent back from, so feedback never overwrites the brief.

use super::{changeset, exec};

changeset!(TaskStepPasses, "029-task-step-passes", |manager| {
    exec(manager, &["ALTER TABLE task_step ADD COLUMN passes_json TEXT NOT NULL DEFAULT '[]'"]).await
});
