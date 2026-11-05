//! `069`: the thread under a task's description. Every task whose review line is `OPEN` and still
//! carries a note written with the old Send back gets it as a `PENDING` `IMPLEMENTATION` entry, so
//! that note is not lost. `review_note` itself is dropped later, by `070`.

use super::{changeset, exec};

changeset!(TaskFeedback, "069-task-feedback", |manager| {
    exec(manager, &[
        "CREATE TABLE task_feedback (
            id TEXT NOT NULL CONSTRAINT pk_task_feedback PRIMARY KEY,
            task_id TEXT NOT NULL,
            seq INTEGER NOT NULL,
            kind VARCHAR(20) NOT NULL,
            state VARCHAR(20) NOT NULL,
            quoted_markdown VARCHAR(100000),
            body_markdown VARCHAR(100000) NOT NULL,
            reply_markdown VARCHAR(2000),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            applied_at TEXT,
            CONSTRAINT fk_task_feedback_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT uq_task_feedback_seq UNIQUE (task_id, seq))",
        "INSERT INTO task_feedback (id, task_id, seq, kind, state, body_markdown, created_at, updated_at)
            SELECT lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-a'
                   || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))),
                   id, 1, 'IMPLEMENTATION', 'PENDING', review_note, updated_at, updated_at
            FROM task WHERE review_note IS NOT NULL AND review_state = 'OPEN'",
    ])
    .await
});
