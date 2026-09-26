use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(WrapupAuthor { Claude = "CLAUDE", Hand = "HAND" });
