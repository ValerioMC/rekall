//! `053`: a visit keeps what the session reported when it left the step, so the step the flow goes
//! to next reads what failed instead of relying on a session's memory.

use super::{changeset, exec};

changeset!(VisitReport, "053-visit-report", |manager| {
    exec(manager, &["ALTER TABLE step_visit ADD COLUMN report TEXT"]).await
});
