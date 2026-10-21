use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(ConnectionKind { Gitlab = "GITLAB", Kubernetes = "KUBERNETES" });
