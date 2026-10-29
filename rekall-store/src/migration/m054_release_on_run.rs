//! `054`: a project may choose one of its graphs to release every task the console runs, and a
//! task remembers when a graph was laid on it and whether it is to be left unreleased.

use super::{changeset, exec};

changeset!(ReleaseOnRun, "054-release-on-run", |manager| {
    exec(manager, &[
        "ALTER TABLE deployment_graph ADD COLUMN release_default BOOLEAN NOT NULL DEFAULT 0",
        "ALTER TABLE task ADD COLUMN graph_laid_at TEXT",
        "ALTER TABLE task ADD COLUMN release_skipped BOOLEAN NOT NULL DEFAULT 0",
    ])
    .await
});
