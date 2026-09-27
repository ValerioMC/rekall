use std::collections::BTreeMap;

/// Free key/value facts a node or edge carries beyond its fixed fields. Ordered, so a graph
/// written twice serialises identically.
pub type Metadata = BTreeMap<String, serde_json::Value>;
