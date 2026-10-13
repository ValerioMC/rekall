//! `041`: each token row says on which day its tokens were generated, so a task's spend can be
//! charted day by day. Existing rows take the day they were read, and every transcript is read
//! again so they are replaced by rows that carry the real days.

use super::{changeset, exec};

changeset!(TokenUsageDay, "041-token-usage-day", |manager| {
    exec(manager, &[
        "ALTER TABLE token_usage ADD COLUMN spent_on TEXT NOT NULL DEFAULT '1970-01-01T00:00:00.000000Z'",
        "UPDATE token_usage SET spent_on = substr(recorded_at, 1, 10) || 'T00:00:00.000000Z'",
        "DELETE FROM transcript_scan",
    ])
    .await
});
