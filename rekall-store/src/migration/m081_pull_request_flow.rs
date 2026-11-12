//! `081`: a project can turn on the pull request flow, off unless it says otherwise.

use super::{changeset, exec};

changeset!(PullRequestFlow, "081-pull-request-flow", |manager| {
    exec(manager, &["ALTER TABLE project ADD COLUMN pull_request_flow BOOLEAN NOT NULL DEFAULT 0"]).await
});
