//! `074`: a task in `DONE` is accepted as a whole. Its open, running or claimed steps become done,
//! and a task without a checklist takes the accepted review line. Drafts and epics stay as they are.

use super::{changeset, exec};

changeset!(AcceptDoneTasks, "074-accept-done-tasks", |manager| {
    exec(manager, &[
        "UPDATE task_step SET state = 'DONE', done_at = COALESCE(done_at, updated_at)
            WHERE state IN ('OPEN', 'RUNNING', 'CLAIMED')
            AND task_id IN (SELECT id FROM task WHERE status = 'DONE' AND kind = 'TASK')",
        "UPDATE task SET review_state = 'DONE',
                claimed_at = COALESCE(claimed_at, updated_at),
                accepted_at = COALESCE(accepted_at, updated_at)
            WHERE status = 'DONE' AND kind = 'TASK' AND review_state <> 'DONE'
            AND NOT EXISTS (SELECT 1 FROM task_step WHERE task_step.task_id = task.id AND task_step.state <> 'DRAFT')",
    ])
    .await
});
