//! `070`: the note a person typed when sending a stepless task back is gone: corrections now go in
//! the task's thread. `069` already copied every note still waiting on an open task into it.

use super::{changeset, exec};

changeset!(DropReviewNote, "070-drop-review-note", |manager| {
    exec(manager, &["ALTER TABLE task DROP COLUMN review_note"]).await
});
