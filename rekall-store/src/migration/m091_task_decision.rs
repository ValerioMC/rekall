//! `091`: the decisions a person and a session settle about a task before it runs. A decision is one
//! question with its answer, optionally about one step; its options, conversation lines and later
//! details hang from it. Every row goes with its task.

use super::{changeset, exec};

changeset!(TaskDecision, "091-task-decision", |manager| {
    exec(manager, &[
        "CREATE TABLE task_decision (
            id TEXT NOT NULL CONSTRAINT pk_task_decision PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_number INTEGER,
            key VARCHAR(100) NOT NULL,
            topic VARCHAR(200),
            question VARCHAR(500) NOT NULL,
            why VARCHAR(2000),
            state VARCHAR(20) NOT NULL,
            answer VARCHAR(4000),
            previous_answer VARCHAR(4000),
            changed_at TEXT,
            source VARCHAR(20) NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_task_decision_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT uq_task_decision_key UNIQUE (task_id, key),
            CONSTRAINT ck_task_decision_state CHECK (state IN ('OPEN', 'ANSWERED', 'ON_HOLD')),
            CONSTRAINT ck_task_decision_source CHECK (source IN ('TERMINAL', 'CHAT', 'PERSON')))",
        "CREATE TABLE task_decision_option (
            id TEXT NOT NULL CONSTRAINT pk_task_decision_option PRIMARY KEY,
            decision_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            label VARCHAR(200) NOT NULL,
            effect VARCHAR(1000),
            recommended BOOLEAN NOT NULL DEFAULT 0,
            CONSTRAINT fk_task_decision_option_decision FOREIGN KEY (decision_id) REFERENCES task_decision (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_task_decision_option_decision_id ON task_decision_option (decision_id, position)",
        "CREATE TABLE task_decision_line (
            id TEXT NOT NULL CONSTRAINT pk_task_decision_line PRIMARY KEY,
            decision_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            speaker VARCHAR(20) NOT NULL,
            text VARCHAR(4000) NOT NULL,
            spoken_at TEXT NOT NULL,
            CONSTRAINT fk_task_decision_line_decision FOREIGN KEY (decision_id) REFERENCES task_decision (id) ON DELETE CASCADE,
            CONSTRAINT ck_task_decision_line_speaker CHECK (speaker IN ('CLAUDE', 'PERSON')))",
        "CREATE INDEX ix_task_decision_line_decision_id ON task_decision_line (decision_id, position)",
        "CREATE TABLE task_decision_detail (
            id TEXT NOT NULL CONSTRAINT pk_task_decision_detail PRIMARY KEY,
            decision_id TEXT NOT NULL,
            text VARCHAR(2000) NOT NULL,
            added_at TEXT NOT NULL,
            CONSTRAINT fk_task_decision_detail_decision FOREIGN KEY (decision_id) REFERENCES task_decision (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_task_decision_detail_decision_id ON task_decision_detail (decision_id, added_at)",
    ])
    .await
});
