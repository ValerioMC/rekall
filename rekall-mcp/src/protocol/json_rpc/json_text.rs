use serde_json::Value;

/// A string as itself, a number or boolean as its text, anything else empty.
pub fn as_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// The field as text, absent when it is missing or null.
pub fn text_at(node: Option<&Value>, field: &str) -> Option<String> {
    node.and_then(|n| n.get(field)).filter(|v| !v.is_null()).map(as_text)
}
