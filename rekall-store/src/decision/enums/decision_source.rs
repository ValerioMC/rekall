use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// Where a decision was taken: a terminal session, a chat session, or the person editing the record by hand.
    DecisionSource {
        Terminal = "TERMINAL",
        Chat = "CHAT",
        Person = "PERSON",
    }
);
