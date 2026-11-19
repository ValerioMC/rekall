//! `090`: a component keeps a short summary apart from its description. The summary is what every
//! context carries; the description grows into a structured document read on demand. Existing
//! descriptions were short, so they become the summary too.

use super::{changeset, exec};

changeset!(ComponentSummary, "090-component-summary", |manager| {
    exec(manager, &[
        "ALTER TABLE project_component ADD COLUMN summary TEXT NOT NULL DEFAULT ''",
        "UPDATE project_component SET summary = description",
    ])
    .await
});
