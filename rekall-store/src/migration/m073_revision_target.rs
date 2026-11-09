//! `073`: a thread entry is only ever a revision of the description, applied to the work
//! (`IMPLEMENTATION`) or to the plan (`PLAN`). Feedback written apart from the description is
//! dropped, a revision takes `PLAN` on a task with steps and `IMPLEMENTATION` on one without, and
//! `follows_seq` goes.

use super::{changeset, exec};

changeset!(RevisionTarget, "073-revision-target", |manager| {
    exec(manager, &[
        "DELETE FROM task_feedback WHERE quoted_markdown IS NULL",
        "UPDATE task_feedback SET kind = CASE
            WHEN EXISTS (SELECT 1 FROM task_step WHERE task_step.task_id = task_feedback.task_id) THEN 'PLAN'
            ELSE 'IMPLEMENTATION' END
            WHERE kind = 'REVISION'",
        "ALTER TABLE task_feedback DROP COLUMN follows_seq",
    ])
    .await
});
