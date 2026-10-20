use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::WalkthroughMessage;

/// How a sequence sheet plays as an animated walkthrough: which participants live outside the
/// project's code, and what the reader is told at each message, keyed `#n` like a message link.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Walkthrough {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub messages: BTreeMap<String, WalkthroughMessage>,
}
