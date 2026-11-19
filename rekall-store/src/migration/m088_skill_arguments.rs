//! `088`: a skill assigned to a task, a step or a tag may carry the arguments it is invoked with.
//! Rows already there keep no arguments.

use super::{changeset, exec};

changeset!(SkillArguments, "088-skill-arguments", |manager| {
    exec(manager, &[
        "ALTER TABLE task_skill ADD COLUMN arguments VARCHAR(300)",
        "ALTER TABLE tag_skill ADD COLUMN arguments VARCHAR(300)",
    ])
    .await
});
