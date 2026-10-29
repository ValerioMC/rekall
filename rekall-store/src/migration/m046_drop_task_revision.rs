//! `046`: earlier versions of a wrapup or a description are no longer kept, so the table that held
//! them goes with every revision in it.

use super::{changeset, exec};

changeset!(DropTaskRevision, "046-drop-task-revision", |manager| {
    exec(manager, &["DROP INDEX ix_task_revision_task_kind", "DROP TABLE task_revision"]).await
});
