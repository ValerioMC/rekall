use super::{changeset, exec};

changeset!(RunQueueClaudeOptions, "099-run-queue-claude-options", |manager| {
    exec(manager, &[
        "ALTER TABLE run_queue_item ADD COLUMN claude_options_configured BOOLEAN NOT NULL DEFAULT 0",
        "ALTER TABLE run_queue_item ADD COLUMN claude_model VARCHAR(150)",
        "ALTER TABLE run_queue_item ADD COLUMN claude_effort VARCHAR(150)",
    ]).await
});
