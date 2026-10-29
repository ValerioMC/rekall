use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// Which exposures a project-wide fix takes: the ones still a warning, or also the ones a
    /// session judged not affected.
    FixScope { Affected = "AFFECTED", All = "ALL" }
);
