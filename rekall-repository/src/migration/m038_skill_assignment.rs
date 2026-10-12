//! `038`: the Claude skills assigned to a task, to one of its steps, or to a tag. A skill is kept
//! by name and never checked against `~/.claude`: one deleted on disk leaves a row that a session
//! simply cannot invoke. Deleting the task, step or tag deletes its rows.

use super::{changeset, exec};

changeset!(SkillAssignment, "038-skill-assignment", |manager| {
    exec(manager, &[
        "CREATE TABLE task_skill (
            id TEXT NOT NULL CONSTRAINT pk_task_skill PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_id TEXT,
            skill_name VARCHAR(120) NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            CONSTRAINT fk_task_skill_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_task_skill_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE CASCADE)",
        "CREATE UNIQUE INDEX ux_task_skill_task ON task_skill (task_id, skill_name) WHERE step_id IS NULL",
        "CREATE UNIQUE INDEX ux_task_skill_step ON task_skill (step_id, skill_name) WHERE step_id IS NOT NULL",
        "CREATE INDEX ix_task_skill_task ON task_skill (task_id)",
        "CREATE TABLE tag_skill (
            id TEXT NOT NULL CONSTRAINT pk_tag_skill PRIMARY KEY,
            tag_id TEXT NOT NULL,
            skill_name VARCHAR(120) NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            CONSTRAINT fk_tag_skill_tag FOREIGN KEY (tag_id) REFERENCES tag (id) ON DELETE CASCADE)",
        "CREATE UNIQUE INDEX ux_tag_skill ON tag_skill (tag_id, skill_name)",
    ])
    .await
});
