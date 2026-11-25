use serde::{Deserialize, Serialize};

/// The one concrete case a diagram follows, so a reader knows what situation it shows before the
/// first box: who does what and where, the values involved, and what they see at the end.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scenario {
    pub situation: String,
    pub example: String,
    pub outcome: String,
}

impl Scenario {
    /// Each field with its name, in the order a reader meets them.
    pub fn fields(&self) -> [(&'static str, &str); 3] {
        [("situation", &self.situation), ("example", &self.example), ("outcome", &self.outcome)]
    }
}
