//! `087`: a commit names the component whose repository it was read from. No component means the
//! project's own folder, which is where every commit logged before this one came from.

use super::{changeset, exec};

changeset!(CommitReferenceComponent, "087-commit-reference-component", |manager| {
    exec(manager, &[
        "ALTER TABLE commit_reference ADD COLUMN component_id TEXT REFERENCES project_component (id) ON DELETE SET NULL",
        "CREATE INDEX ix_commit_reference_component ON commit_reference (component_id)",
    ])
    .await
});
