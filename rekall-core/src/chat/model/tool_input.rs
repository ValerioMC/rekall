use serde::Serialize;
use serde_json::Value;

use crate::chat::{TodoItem, TodoStatus};

const MCP_PREFIX: &str = "mcp__";
const MCP_SEPARATOR: &str = "__";

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
    Todos { items: Vec<TodoItem> },
    Agent { description: String, subagent_type: Option<String>, prompt: String },
    Web { name: String, target: String },
    Mcp { server: String, tool_name: String, text: String },
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
            "TodoWrite" => todos_of(input),
            "Task" | "Agent" => agent_of(input),
            "WebFetch" => web_of(tool, input, "url"),
            "WebSearch" => web_of(tool, input, "query"),
            _ => mcp_of(tool, input),
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

fn todos_of(input: &Value) -> Option<ToolInput> {
    let items = input.get("todos")?.as_array()?.iter().map(todo_of).collect::<Option<Vec<_>>>()?;
    Some(ToolInput::Todos { items })
}

fn todo_of(item: &Value) -> Option<TodoItem> {
    Some(TodoItem { content: text(item, "content")?, status: TodoStatus::parse(&text(item, "status")?)? })
}

fn agent_of(input: &Value) -> Option<ToolInput> {
    Some(ToolInput::Agent { description: text(input, "description")?, subagent_type: text(input, "subagent_type"), prompt: text(input, "prompt")? })
}

fn web_of(tool: &str, input: &Value, target_field: &str) -> Option<ToolInput> {
    text(input, target_field).map(|target| ToolInput::Web { name: tool.to_string(), target })
}

fn mcp_of(tool: &str, input: &Value) -> Option<ToolInput> {
    let (server, tool_name) = tool.strip_prefix(MCP_PREFIX)?.split_once(MCP_SEPARATOR)?;
    Some(ToolInput::Mcp { server: server.to_string(), tool_name: tool_name.to_string(), text: readable(input) })
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
