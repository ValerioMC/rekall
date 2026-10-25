use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// How severe the GitHub Advisory Database rates an advisory; its own words upper-cased.
    AdvisorySeverity { Critical = "CRITICAL", High = "HIGH", Medium = "MEDIUM", Low = "LOW", Unknown = "UNKNOWN" }
);
