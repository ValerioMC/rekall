//! `011` to `018`: the persisted Claude sessions that were later dropped for the PTY terminal,
//! the task-scoped review line, the wrapup directive that came and went, and draft steps.

use super::{changeset, exec, rebuild};

changeset!(ClaudeSession, "011-claude-session", |manager| {
    exec(manager, &[
        "CREATE TABLE claude_session (
            id TEXT NOT NULL CONSTRAINT pk_claude_session PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_id TEXT,
            anchors VARCHAR(300) NOT NULL,
            working_dir VARCHAR(1000) NOT NULL,
            cli_session_id VARCHAR(200),
            status VARCHAR(20) NOT NULL DEFAULT 'STARTING',
            skip_permissions BOOLEAN NOT NULL DEFAULT 0,
            detail VARCHAR(2000),
            exit_code INTEGER,
            last_activity_at TEXT NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_claude_session_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
        "CREATE INDEX idx_claude_session_task ON claude_session (task_id)",
        "CREATE INDEX idx_claude_session_status ON claude_session (status)",
        "CREATE TABLE claude_message (
            id TEXT NOT NULL CONSTRAINT pk_claude_message PRIMARY KEY,
            session_id TEXT NOT NULL,
            seq INTEGER NOT NULL,
            role VARCHAR(16) NOT NULL,
            content VARCHAR(1000000),
            tool_name VARCHAR(120),
            meta VARCHAR(8000),
            created_at TEXT NOT NULL,
            CONSTRAINT fk_claude_message_session FOREIGN KEY (session_id) REFERENCES claude_session (id) ON DELETE CASCADE)",
        "CREATE INDEX idx_claude_message_session ON claude_message (session_id, seq)",
    ])
    .await
});

changeset!(ClaudeSessionModel, "012-claude-session-model", |manager| {
    exec(manager, &["ALTER TABLE claude_session ADD COLUMN model VARCHAR(60)"]).await
});

changeset!(ClaudeSessionEffort, "013-claude-session-effort", |manager| {
    exec(manager, &["ALTER TABLE claude_session ADD COLUMN effort VARCHAR(10)"]).await
});

changeset!(TaskReviewState, "014-task-review-state", |manager| {
    exec(manager, &[
        "ALTER TABLE task ADD COLUMN review_state VARCHAR(20) NOT NULL DEFAULT 'OPEN'",
        "ALTER TABLE task ADD COLUMN claimed_at TEXT",
        "ALTER TABLE task ADD COLUMN accepted_at TEXT",
        "ALTER TABLE task ADD COLUMN review_note VARCHAR(2000)",
    ])
    .await
});

changeset!(TaskWrapupDirective, "015-task-wrapup-directive", |manager| {
    exec(manager, &[
        "ALTER TABLE task ADD COLUMN auto_wrapup BOOLEAN NOT NULL DEFAULT 0",
        "ALTER TABLE task ADD COLUMN wrapup_directive VARCHAR(2000)",
    ])
    .await
});

// dropDefaultValue + addDefaultValue: a column default is part of the table definition in SQLite.
changeset!(TaskStepDraft, "016-task-step-draft", |manager| {
    rebuild(
        manager,
        "task_step",
        "id TEXT NOT NULL CONSTRAINT pk_task_step PRIMARY KEY,
         task_id TEXT NOT NULL,
         title VARCHAR(200) NOT NULL,
         body_markdown VARCHAR(20000),
         done_at TEXT,
         position INTEGER NOT NULL,
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         state VARCHAR(20) NOT NULL DEFAULT 'DRAFT',
         running_at TEXT,
         claimed_at TEXT,
         CONSTRAINT fk_task_step_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE",
        "id, task_id, title, body_markdown, done_at, position, created_at, updated_at, state, running_at, claimed_at",
        &["CREATE INDEX idx_task_step_task ON task_step (task_id, position)"],
    )
    .await
});

changeset!(DropClaudeSession, "017-drop-claude-session", |manager| {
    exec(manager, &["DROP TABLE claude_message", "DROP TABLE claude_session"]).await
});

changeset!(DropTaskWrapupDirective, "018-drop-task-wrapup-directive", |manager| {
    exec(manager, &[
        "ALTER TABLE task DROP COLUMN auto_wrapup",
        "ALTER TABLE task DROP COLUMN wrapup_directive",
    ])
    .await
});
