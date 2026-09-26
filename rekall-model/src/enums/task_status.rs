use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(TaskStatus { Todo = "TODO", InProgress = "IN_PROGRESS", Blocked = "BLOCKED", Done = "DONE" });
