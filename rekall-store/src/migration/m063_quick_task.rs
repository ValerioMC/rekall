//! `063`: a task started from one line of text (`quick`) and one accepted without a review
//! (`auto_accepted`). Both default to false, so every existing task stays a plain one.

use super::{changeset, exec};

changeset!(QuickTask, "063-quick-task", |manager| {
    exec(manager, &[
        "ALTER TABLE task ADD COLUMN quick BOOLEAN NOT NULL DEFAULT 0",
        "ALTER TABLE task ADD COLUMN auto_accepted BOOLEAN NOT NULL DEFAULT 0",
    ])
    .await
});
