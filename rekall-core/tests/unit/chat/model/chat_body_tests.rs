use super::*;
use crate::chat::ChatEvent;

#[test]
fn a_short_output_is_kept_whole() {
    let ChatBody::ToolResult { excerpt, truncated, is_error, .. } = ChatBody::tool_result("toolu_1", false, "done") else {
        panic!("a tool result");
    };

    assert_eq!(excerpt, "done");
    assert!(!truncated);
    assert!(!is_error);
}

#[test]
fn a_long_output_is_cut_at_the_limit_and_flagged() {
    let output = "é".repeat(TOOL_EXCERPT_MAX_CHARACTERS + 10);

    let ChatBody::ToolResult { excerpt, truncated, .. } = ChatBody::tool_result("toolu_1", true, &output) else {
        panic!("a tool result");
    };

    assert_eq!(excerpt.chars().count(), TOOL_EXCERPT_MAX_CHARACTERS);
    assert!(truncated);
}

#[test]
fn an_event_is_written_flat_with_its_type_and_camel_case_fields() {
    let event = ChatEvent { seq: 7, body: ChatBody::AssistantText { message_id: "msg_1".into(), text: "hi".into(), complete: false } };

    assert_eq!(
        serde_json::to_value(&event).unwrap(),
        serde_json::json!({"seq": 7, "type": "assistantText", "messageId": "msg_1", "text": "hi", "complete": false})
    );
}

#[test]
fn a_turn_is_written_with_its_counters_and_outcome() {
    let usage = TurnUsage { input_tokens: 2, cache_creation_input_tokens: 3, cache_read_input_tokens: 4, output_tokens: 5 };
    let event = ChatEvent { seq: 1, body: ChatBody::TurnFinished { usage, duration_ms: 90, outcome: TurnOutcome::Interrupted } };

    assert_eq!(
        serde_json::to_value(&event).unwrap(),
        serde_json::json!({
            "seq": 1, "type": "turnFinished", "durationMs": 90, "outcome": "interrupted",
            "usage": {"inputTokens": 2, "cacheCreationInputTokens": 3, "cacheReadInputTokens": 4, "outputTokens": 5}
        })
    );
}
