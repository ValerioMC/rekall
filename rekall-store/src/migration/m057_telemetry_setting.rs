//! `057`: where Claude Code's OpenTelemetry traces are sent, kept as a single row that is absent
//! until a person first saves it.

use super::{changeset, exec};

changeset!(TelemetrySetting, "057-telemetry-setting", |manager| {
    exec(manager, &[
        "CREATE TABLE telemetry_setting (
            id TEXT NOT NULL CONSTRAINT pk_telemetry_setting PRIMARY KEY,
            enabled BOOLEAN NOT NULL DEFAULT 0,
            host VARCHAR(300) NOT NULL,
            public_key VARCHAR(200) NOT NULL,
            log_content BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL)",
    ])
    .await
});
