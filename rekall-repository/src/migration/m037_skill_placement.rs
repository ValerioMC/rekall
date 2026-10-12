//! `037`: which Rekall-only group each of the person's own Claude skills is shown under. The
//! skill is named by its path under `~/.claude`, which this table never checks: a skill deleted on
//! disk leaves a row that nothing reads.

use super::{changeset, exec};

changeset!(SkillPlacement, "037-skill-placement", |manager| {
    exec(manager, &[
        "CREATE TABLE skill_placement (
            skill_path VARCHAR(300) NOT NULL CONSTRAINT pk_skill_placement PRIMARY KEY,
            group_label VARCHAR(60) NOT NULL,
            placed_at TEXT NOT NULL)",
        "CREATE INDEX ix_skill_placement_group ON skill_placement (group_label)",
    ])
    .await
});
