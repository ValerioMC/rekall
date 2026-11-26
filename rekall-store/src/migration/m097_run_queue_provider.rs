use super::{changeset, exec};

changeset!(RunQueueProvider, "097-run-queue-provider", |manager| {
    exec(manager, &[
        "ALTER TABLE run_queue_item ADD COLUMN provider VARCHAR(20) NOT NULL DEFAULT 'CLAUDE' CHECK (provider IN ('CLAUDE', 'CODEX'))",
        "ALTER TABLE run_queue_item ADD COLUMN codex_model VARCHAR(150)",
        "ALTER TABLE run_queue_item ADD COLUMN codex_effort VARCHAR(150)",
        "ALTER TABLE run_queue_item ADD COLUMN codex_skip_permissions BOOLEAN NOT NULL DEFAULT 0",
    ]).await
});
