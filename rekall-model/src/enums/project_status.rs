use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(ProjectStatus { Active = "ACTIVE", Paused = "PAUSED", Done = "DONE" });
