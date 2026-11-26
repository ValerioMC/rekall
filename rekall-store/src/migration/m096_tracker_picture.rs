//! `096`: the pictures already uploaded to a task manager, so a push sends each one only once.
//! A picture is named without `/api/images/`: a row here is no use of it.

use super::{changeset, exec};

changeset!(TrackerPicture, "096-tracker-picture", |manager| {
    exec(manager, &[
        "CREATE TABLE tracker_picture (
            task_id TEXT NOT NULL,
            destination VARCHAR(1000) NOT NULL,
            picture_path VARCHAR(1000) NOT NULL,
            remote_reference VARCHAR(2000) NOT NULL,
            created_at TEXT NOT NULL,
            CONSTRAINT pk_tracker_picture PRIMARY KEY (task_id, destination, picture_path),
            CONSTRAINT fk_tracker_picture_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
    ])
    .await
});
