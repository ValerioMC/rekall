use serde_json::json;

use super::ToolInput;

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
    let input = ToolInput::of("mcp__rekall__rekall_context", &json!({"anchors": "project:vega"}));

    let ToolInput::Other { name, text } = input else { panic!("an other tool") };
    assert_eq!(name, "mcp__rekall__rekall_context");
    assert!(text.contains("project:vega"));
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
