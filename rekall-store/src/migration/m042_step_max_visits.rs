//! `042`: a step may cap how many times a flow enters it, so a loop running unattended stops
//! instead of going round forever. Null sets no cap.

use super::{changeset, exec};

changeset!(StepMaxVisits, "042-step-max-visits", |manager| {
    exec(manager, &["ALTER TABLE task_step ADD COLUMN max_visits INTEGER"]).await
});
