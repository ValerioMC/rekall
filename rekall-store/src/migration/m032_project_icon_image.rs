//! `032`: the picture a project wears in place of its icon key. A table of its own, not a column on
//! `project`, so the image bytes are read only when an icon is asked for and never ride along with
//! every project query.

use super::{changeset, exec};

changeset!(ProjectIconImage, "032-project-icon-image", |manager| {
    exec(manager, &[
        "CREATE TABLE project_icon_image (
            project_id TEXT NOT NULL CONSTRAINT pk_project_icon_image PRIMARY KEY,
            png BLOB NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_project_icon_image_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE)",
    ])
    .await
});
