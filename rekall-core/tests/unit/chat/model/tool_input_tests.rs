use serde_json::json;

use super::ToolInput;
use crate::chat::{TodoItem, TodoStatus};

#[test]
fn a_bash_call_keeps_its_command_and_description() {
    let input = ToolInput::of("Bash", &json!({"command": "ls -l", "description": "List files"}));

    assert_eq!(input, ToolInput::Bash { command: "ls -l".into(), description: Some("List files".into()) });
}

#[test]
fn an_edit_carries_both_sides_of_the_change() {
    let input = ToolInput::of("Edit", &json!({"file_path": "a.rs", "old_string": "x", "new_string": "y", "replace_all": true}));

    assert_eq!(input, ToolInput::Edit { file_path: "a.rs".into(), old_string: "x".into(), new_string: "y".into(), replace_all: true });
}

#[test]
fn a_write_a_read_and_a_search_are_typed() {
    assert_eq!(ToolInput::of("Write", &json!({"file_path": "a", "content": "c"})), ToolInput::Write { file_path: "a".into(), content: "c".into() });
    assert_eq!(ToolInput::of("Read", &json!({"file_path": "a"})), ToolInput::Read { file_path: "a".into() });
    assert_eq!(
        ToolInput::of("Grep", &json!({"pattern": "todo", "path": "src"})),
        ToolInput::Search { name: "Grep".into(), pattern: "todo".into(), path: Some("src".into()) }
    );
    assert_eq!(
        ToolInput::of("Glob", &json!({"pattern": "*.rs"})),
        ToolInput::Search { name: "Glob".into(), pattern: "*.rs".into(), path: None }
    );
}

#[test]
fn an_unknown_tool_is_shown_as_readable_text() {
    let input = ToolInput::of("NotebookEdit", &json!({"notebook_path": "a.ipynb"}));

    let ToolInput::Other { name, text } = input else { panic!("an other tool") };
    assert_eq!(name, "NotebookEdit");
    assert!(text.contains("a.ipynb"));
}

#[test]
fn a_known_tool_without_its_fields_falls_back_to_text() {
    let input = ToolInput::of("Bash", &json!({"unexpected": 1}));

    assert!(matches!(input, ToolInput::Other { ref name, .. } if name == "Bash"));
}

#[test]
fn a_tool_input_is_written_with_the_tool_as_its_tag() {
    let input = ToolInput::Read { file_path: "a".into() };

    assert_eq!(serde_json::to_value(&input).unwrap(), json!({"tool": "read", "filePath": "a"}));
}

#[test]
fn a_todo_list_carries_each_item_with_its_status() {
    let input = ToolInput::of(
        "TodoWrite",
        &json!({"todos": [
            {"content": "Fetch the page", "status": "in_progress", "activeForm": "Fetching the page"},
            {"content": "Answer", "status": "pending", "activeForm": "Answering"},
            {"content": "Done already", "status": "completed", "activeForm": "Finishing"}
        ]}),
    );

    assert_eq!(
        input,
        ToolInput::Todos {
            items: vec![
                TodoItem { content: "Fetch the page".into(), status: TodoStatus::InProgress },
                TodoItem { content: "Answer".into(), status: TodoStatus::Pending },
                TodoItem { content: "Done already".into(), status: TodoStatus::Completed },
            ]
        }
    );
}

#[test]
fn an_agent_is_named_by_either_of_the_names_claude_code_has_used_for_it() {
    let arguments = json!({"description": "Fetch the title", "subagent_type": "web-fetch", "prompt": "Fetch it"});
    let expected = ToolInput::Agent { description: "Fetch the title".into(), subagent_type: Some("web-fetch".into()), prompt: "Fetch it".into() };

    assert_eq!(ToolInput::of("Agent", &arguments), expected);
    assert_eq!(ToolInput::of("Task", &arguments), expected);
}

#[test]
fn a_web_call_carries_the_url_or_the_query() {
    assert_eq!(
        ToolInput::of("WebFetch", &json!({"url": "https://example.com", "prompt": "title"})),
        ToolInput::Web { name: "WebFetch".into(), target: "https://example.com".into() }
    );
    assert_eq!(
        ToolInput::of("WebSearch", &json!({"query": "rust serde", "mode": "standard"})),
        ToolInput::Web { name: "WebSearch".into(), target: "rust serde".into() }
    );
}

#[test]
fn an_mcp_call_is_split_into_its_server_and_tool_and_keeps_its_arguments_readable() {
    let input = ToolInput::of("mcp__plugin_figma_figma__authenticate", &json!({"anchors": "project:vega"}));

    let ToolInput::Mcp { server, tool_name, text } = input else { panic!("an mcp call") };
    assert_eq!(server, "plugin_figma_figma");
    assert_eq!(tool_name, "authenticate");
    assert!(text.contains("project:vega"));
}

#[test]
fn a_typed_tool_missing_its_fields_stays_other() {
    for (tool, arguments) in [
        ("TodoWrite", json!({"todos": [{"content": "x", "status": "unheard-of"}]})),
        ("TodoWrite", json!({})),
        ("Agent", json!({"description": "no prompt"})),
        ("WebFetch", json!({"prompt": "no url"})),
        ("WebSearch", json!({})),
        ("mcp__serveronly", json!({})),
    ] {
        assert!(matches!(ToolInput::of(tool, &arguments), ToolInput::Other { .. }), "{tool} should stay Other");
    }
}

#[test]
fn the_new_variants_are_written_with_the_tool_as_their_tag() {
    let todos = ToolInput::Todos { items: vec![TodoItem { content: "x".into(), status: TodoStatus::InProgress }] };
    let mcp = ToolInput::Mcp { server: "s".into(), tool_name: "t".into(), text: "{}".into() };

    assert_eq!(serde_json::to_value(&todos).unwrap(), json!({"tool": "todos", "items": [{"content": "x", "status": "inProgress"}]}));
    assert_eq!(serde_json::to_value(&mcp).unwrap(), json!({"tool": "mcp", "server": "s", "toolName": "t", "text": "{}"}));
}
