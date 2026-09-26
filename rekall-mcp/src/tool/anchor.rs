use std::sync::LazyLock;

use regex::Regex;

use crate::protocol::ToolError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub entity_name: Option<String>,
    pub value: String,
}

impl Anchor {
    pub fn new(entity_name: Option<&str>, value: &str) -> Self {
        Self { entity_name: entity_name.map(str::to_string), value: value.to_string() }
    }

    /// Every term in `raw`: whitespace-separated, a quoted value kept whole.
    pub fn parse_all(raw: Option<&str>) -> Result<Vec<Anchor>, ToolError> {
        // `\S*"[^"]*"|\S+`, with Java's ASCII `\S`.
        static TERM: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r#"[^ \t\n\x0B\x0C\r]*"[^"]*"|[^ \t\n\x0B\x0C\r]+"#).unwrap()
        });
        let mut anchors = Vec::new();
        for term in TERM.find_iter(raw.unwrap_or("")) {
            anchors.push(Self::parse(term.as_str())?);
        }
        if anchors.is_empty() {
            return Err(ToolError::Failure(
                "No anchor given. Pass something like `project:vega task:report-builder`.".into(),
            ));
        }
        Ok(anchors)
    }

    fn parse(term: &str) -> Result<Anchor, ToolError> {
        let separator = term.find(':');
        let quote = term.find('"');
        let qualified = match (separator, quote) {
            (Some(s), None) => Some(s),
            (Some(s), Some(q)) if s < q => Some(s),
            _ => None,
        };
        let Some(separator) = qualified else {
            return Ok(Anchor { entity_name: None, value: unquote(term) });
        };
        let entity_name = rekall_common::jstr::trim(&term[..separator]);
        let value = unquote(rekall_common::jstr::trim(&term[separator + 1..]));
        if entity_name.is_empty() || value.is_empty() {
            return Err(ToolError::Failure(format!(
                "'{term}' is not a valid anchor. Use `entity:value`, or a bare value."
            )));
        }
        Ok(Anchor { entity_name: Some(entity_name.to_string()), value })
    }

    pub fn is_qualified(&self) -> bool {
        self.entity_name.is_some()
    }

    pub fn is(&self, entity: &str) -> bool {
        self.entity_name.as_deref().is_some_and(|name| rekall_common::jstr::equals_ignore_case(name, entity))
    }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.entity_name {
            Some(entity) => write!(f, "{entity}:{}", self.value),
            None => f.write_str(&self.value),
        }
    }
}

fn unquote(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return rekall_common::jstr::trim(&value[1..value.len() - 1]).to_string();
    }
    value.to_string()
}

#[cfg(test)]
#[path = "../../tests/unit/tool/anchor_tests.rs"]
mod tests;
