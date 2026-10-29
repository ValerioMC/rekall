//! `036`: the tokens Claude Code sessions spent on each task and step, read from their
//! transcripts, and how far each transcript has been read. A deleted step leaves its tokens on
//! the task rather than taking them with it.

use super::{changeset, exec};

changeset!(TokenUsage, "036-token-usage", |manager| {
    exec(manager, &[
        "CREATE TABLE token_usage (
            id TEXT NOT NULL CONSTRAINT pk_token_usage PRIMARY KEY,
            session_id VARCHAR(100) NOT NULL,
            task_id TEXT NOT NULL,
            step_id TEXT,
            input_tokens INTEGER NOT NULL,
            output_tokens INTEGER NOT NULL,
            cache_creation_tokens INTEGER NOT NULL,
            cache_read_tokens INTEGER NOT NULL,
            recorded_at TEXT NOT NULL,
            CONSTRAINT fk_token_usage_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_token_usage_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE SET NULL)",
        "CREATE INDEX ix_token_usage_session ON token_usage (session_id)",
        "CREATE INDEX ix_token_usage_task ON token_usage (task_id)",
        "CREATE TABLE transcript_scan (
            session_id VARCHAR(100) NOT NULL CONSTRAINT pk_transcript_scan PRIMARY KEY,
            scanned_bytes INTEGER NOT NULL,
            scanned_at TEXT NOT NULL)",
    ])
    .await
});
