use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// What a session concluded after reading the code: whether the vulnerable code is reached.
    ExposureVerdict { Affected = "AFFECTED", NotAffected = "NOT_AFFECTED" }
);
