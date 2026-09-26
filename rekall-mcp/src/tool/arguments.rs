use serde_json::Value;

use crate::protocol::json_rpc_text;
use crate::protocol::ToolError;

/// A tool's arguments, read the way `Arguments` read the `JsonNode`.
pub struct Arguments<'a> {
    node: Option<&'a Value>,
}

impl<'a> Arguments<'a> {
    pub fn of(node: Option<&'a Value>) -> Self {
        Self { node }
    }

    /// Present, not null and not blank, or an `IllegalArgumentException`.
    pub fn required_string(&self, name: &str) -> Result<String, ToolError> {
        match self.text(name) {
            Some(value) if !rekall_common::jstr::is_blank(&value) => Ok(value),
            _ => Err(ToolError::Illegal(format!("'{name}' is required"))),
        }
    }

    /// Absent, null and blank all read as absent.
    pub fn optional_string(&self, name: &str) -> Option<String> {
        self.text(name).filter(|value| !rekall_common::jstr::is_blank(value))
    }

    /// True only for `true` or `"true"`, any case; absent, null and anything else read as false.
    pub fn flag(&self, name: &str) -> bool {
        self.text(name).is_some_and(|value| rekall_common::jstr::strip(&value).eq_ignore_ascii_case("true"))
    }

    fn text(&self, name: &str) -> Option<String> {
        json_rpc_text(self.node, name)
    }
}
