use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(Speaker { Claude = "CLAUDE", Person = "PERSON" });
