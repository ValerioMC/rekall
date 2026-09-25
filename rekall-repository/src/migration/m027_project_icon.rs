//! `027`: a project picks its own icon key, independent of its identity colour.

use super::{changeset, exec};

changeset!(ProjectIcon, "027-project-icon", |manager| {
    exec(manager, &["ALTER TABLE project ADD COLUMN icon VARCHAR(40) NOT NULL DEFAULT 'folder'"]).await
});
