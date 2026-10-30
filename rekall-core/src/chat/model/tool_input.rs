use serde::Serialize;
use serde_json::Value;

/// What a tool call was asked to do, typed for the tools the window shows in a way of their own.
/// Any other tool is `Other`, with its input as readable text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "tool", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ToolInput {
    Bash { command: String, description: Option<String> },
    Edit { file_path: String, old_string: String, new_string: String, replace_all: bool },
    Write { file_path: String, content: String },
    Read { file_path: String },
    Search { name: String, pattern: String, path: Option<String> },
    Other { name: String, text: String },
}

impl ToolInput {
    /// The input of a call to `tool` as Claude Code sends it. A call whose input does not have the
    /// fields its tool is known by is shown as `Other` rather than lost.
    pub fn of(tool: &str, input: &Value) -> Self {
        let typed = match tool {
            "Bash" => text(input, "command").map(|command| ToolInput::Bash { command, description: text(input, "description") }),
            "Edit" => edit_of(input),
            "Write" => text(input, "file_path").map(|file_path| ToolInput::Write { file_path, content: text(input, "content").unwrap_or_default() }),
            "Read" => text(input, "file_path").map(|file_path| ToolInput::Read { file_path }),
            "Grep" | "Glob" => text(input, "pattern").map(|pattern| ToolInput::Search { name: tool.to_string(), pattern, path: text(input, "path") }),
            _ => None,
        };
        typed.unwrap_or_else(|| ToolInput::Other { name: tool.to_string(), text: readable(input) })
    }
}

fn edit_of(input: &Value) -> Option<ToolInput> {
    Some(ToolInput::Edit {
        file_path: text(input, "file_path")?,
        old_string: text(input, "old_string")?,
        new_string: text(input, "new_string")?,
        replace_all: input.get("replace_all").and_then(Value::as_bool).unwrap_or(false),
    })
}

fn text(input: &Value, field: &str) -> Option<String> {
    input.get(field).and_then(Value::as_str).map(str::to_string)
}

fn readable(input: &Value) -> String {
    match input {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_default(),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/model/tool_input_tests.rs"]
mod tests;
