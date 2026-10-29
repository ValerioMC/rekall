//! `048`: an assessment says in plain words why the project is or is not affected, beside the
//! reasoning that traces it through the code. Null on exposures judged before it existed.

use super::{changeset, exec};

changeset!(ExposureConclusion, "048-exposure-conclusion", |manager| {
    exec(manager, &["ALTER TABLE vulnerability_exposure ADD COLUMN conclusion VARCHAR(1000)"]).await
});
