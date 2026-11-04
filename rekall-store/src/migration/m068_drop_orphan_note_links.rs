//! `068`: `067` moved notes into `task_deviation` and left their `document_task` links behind,
//! because migrations run with foreign keys off and the cascade never fired. Links whose note
//! is gone are removed, which repairs a file that already ran the first version of `067`.

use super::{changeset, exec};

changeset!(DropOrphanNoteLinks, "068-drop-orphan-note-links", |manager| {
    exec(manager, &["DELETE FROM document_task WHERE document_id NOT IN (SELECT id FROM document)"]).await
});
