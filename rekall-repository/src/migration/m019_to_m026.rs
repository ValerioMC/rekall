//! `019` to `026`: commits logged against tasks, auto-commit, tags, revisions, reference notes and
//! the run queue.

use super::{changeset, exec};

changeset!(CommitReference, "019-commit-reference", |manager| {
    exec(manager, &["CREATE TABLE commit_reference (
        id TEXT NOT NULL CONSTRAINT pk_commit_reference PRIMARY KEY,
        task_id TEXT NOT NULL,
        step_id TEXT,
        commit_hash VARCHAR(40) NOT NULL,
        comment VARCHAR(200) NOT NULL,
        created_at TEXT NOT NULL,
        CONSTRAINT fk_commit_reference_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
        CONSTRAINT fk_commit_reference_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE CASCADE)"])
    .await
});

changeset!(CommitReferenceDiff, "020-commit-reference-diff", |manager| {
    exec(manager, &["ALTER TABLE commit_reference ADD COLUMN diff TEXT"]).await
});

changeset!(CommitReferenceInContext, "021-commit-reference-in-context", |manager| {
    exec(manager, &["ALTER TABLE commit_reference ADD COLUMN in_context BOOLEAN NOT NULL DEFAULT 0"]).await
});

changeset!(ProjectAutoCommit, "022-project-auto-commit", |manager| {
    exec(manager, &["ALTER TABLE project ADD COLUMN auto_commit BOOLEAN NOT NULL DEFAULT 0"]).await
});

changeset!(Tag, "023-tag", |manager| {
    exec(manager, &[
        "CREATE TABLE tag (
            id TEXT NOT NULL CONSTRAINT pk_tag PRIMARY KEY,
            name VARCHAR(60) NOT NULL CONSTRAINT uq_tag_name UNIQUE,
            icon VARCHAR(40) NOT NULL,
            color VARCHAR(40) NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL)",
        "ALTER TABLE task ADD COLUMN tag_id TEXT CONSTRAINT fk_task_tag REFERENCES tag (id) ON DELETE SET NULL",
    ])
    .await
});

changeset!(TaskRevision, "024-task-revision", |manager| {
    exec(manager, &[
        "CREATE TABLE task_revision (
            id TEXT NOT NULL CONSTRAINT pk_task_revision PRIMARY KEY,
            task_id TEXT NOT NULL,
            kind VARCHAR(20) NOT NULL,
            body_markdown VARCHAR(100000) NOT NULL,
            written_by VARCHAR(20),
            written_at TEXT,
            created_at TEXT NOT NULL,
            CONSTRAINT fk_task_revision_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_task_revision_task_kind ON task_revision (task_id, kind, created_at)",
    ])
    .await
});

changeset!(DocumentContextMode, "025-document-context-mode", |manager| {
    exec(manager, &["ALTER TABLE document ADD COLUMN context_mode VARCHAR(20) NOT NULL DEFAULT 'FULL'"]).await
});

changeset!(RunQueue, "026-run-queue", |manager| {
    exec(manager, &[
        "CREATE TABLE run_queue (
            id TEXT NOT NULL CONSTRAINT pk_run_queue PRIMARY KEY,
            state VARCHAR(20) NOT NULL,
            start_at TEXT,
            ceiling_percent INTEGER,
            skip_permissions BOOLEAN NOT NULL DEFAULT 0,
            model VARCHAR(20),
            effort VARCHAR(20),
            hold_until TEXT,
            hold_reason VARCHAR(500),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL)",
        "CREATE TABLE run_queue_item (
            id TEXT NOT NULL CONSTRAINT pk_run_queue_item PRIMARY KEY,
            task_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            state VARCHAR(20) NOT NULL,
            detail VARCHAR(500),
            started_at TEXT,
            finished_at TEXT,
            created_at TEXT NOT NULL,
            CONSTRAINT fk_run_queue_item_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_run_queue_item_position ON run_queue_item (position)",
    ])
    .await
});
