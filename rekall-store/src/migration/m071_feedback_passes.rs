//! `071`: a thread entry can be taken into a numbered pass (`pass`, `taken_at`) and can follow an
//! earlier entry (`follows_seq`). Existing rows keep none of them: an applied entry without a pass is
//! history from before passes.

use super::{changeset, exec};

changeset!(FeedbackPasses, "071-feedback-passes", |manager| {
    exec(manager, &[
        "ALTER TABLE task_feedback ADD COLUMN pass INTEGER",
        "ALTER TABLE task_feedback ADD COLUMN taken_at TEXT",
        "ALTER TABLE task_feedback ADD COLUMN follows_seq INTEGER",
    ])
    .await
});
