use rekall_core::chat::TOOL_EXCERPT_MAX_CHARACTERS;
use serde_json::json;

use super::*;

const PLAIN_REPLY: &str = include_str!("../../fixtures/chat/plain_reply.jsonl");
const TOOL_CALL: &str = include_str!("../../fixtures/chat/tool_call_result.jsonl");
const DENIED: &str = include_str!("../../fixtures/chat/streaming_deltas_denied.jsonl");
const PERMISSION_ALLOW: &str = include_str!("../../fixtures/chat/permission_request_allow.jsonl");
const PERMISSION_DENY: &str = include_str!("../../fixtures/chat/permission_request_deny.jsonl");
const INTERRUPTED: &str = include_str!("../../fixtures/chat/interrupted_turn.jsonl");
const ERROR_RETRY: &str = include_str!("../../fixtures/chat/error_retry.jsonl");

fn parse(fixture: &str) -> Vec<ChatBody> {
    let mut parser = StreamParser::new();
    fixture.lines().flat_map(|line| parser.parse_line(line)).collect()
}

fn parse_lines(lines: &[serde_json::Value]) -> Vec<ChatBody> {
    let mut parser = StreamParser::new();
    lines.iter().flat_map(|line| parser.parse_line(&line.to_string())).collect()
}

fn completed_texts(events: &[ChatBody]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|event| match event {
            ChatBody::AssistantText { text, complete: true, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn streamed_text(events: &[ChatBody]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            ChatBody::AssistantText { text, complete: false, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn turns(events: &[ChatBody]) -> Vec<(TurnUsage, TurnOutcome)> {
    events
        .iter()
        .filter_map(|event| match event {
            ChatBody::TurnFinished { usage, outcome, .. } => Some((*usage, *outcome)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_plain_reply_streams_its_text_then_completes_it_and_finishes_the_turn_with_its_counters() {
    let events = parse(PLAIN_REPLY);

    assert_eq!(completed_texts(&events), vec!["pong", "ping"]);
    assert_eq!(streamed_text(&events), "pongping");
    assert_eq!(
        turns(&events),
        vec![
            (TurnUsage { input_tokens: 2, cache_creation_input_tokens: 14784, cache_read_input_tokens: 9014, output_tokens: 4 }, TurnOutcome::Completed),
            (TurnUsage { input_tokens: 2, cache_creation_input_tokens: 12907, cache_read_input_tokens: 14076, output_tokens: 4 }, TurnOutcome::Completed),
        ]
    );
}

#[test]
fn every_event_of_a_message_names_the_message_it_belongs_to() {
    let events = parse(PLAIN_REPLY);

    let ids: Vec<&str> = events
        .iter()
        .filter_map(|event| match event {
            ChatBody::AssistantText { message_id, .. } => Some(message_id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(ids, vec!["msg_011CgYsvbD4G6v4N3eu4vpyn", "msg_011CgYsvbD4G6v4N3eu4vpyn", "msg_011CgYsvhEhVHmVeRGpopASS", "msg_011CgYsvhEhVHmVeRGpopASS"]);
}

#[test]
fn a_tool_call_is_typed_and_its_result_names_the_call_it_answers() {
    let events = parse(TOOL_CALL);

    let ChatBody::ToolUse { tool_use_id, name, input } = &events[0] else { panic!("the tool call comes first") };
    assert_eq!(name, "Read");
    assert_eq!(*input, ToolInput::Read { file_path: "/work/project/hello.txt".into() });
    let ChatBody::ToolResult { tool_use_id: answered, is_error, excerpt, truncated } = &events[1] else { panic!("its result follows") };
    assert_eq!(answered, tool_use_id);
    assert!(!is_error);
    assert_eq!(excerpt, "1\thello spike\n2\t");
    assert!(!truncated);
}

#[test]
fn thinking_is_announced_once_and_completed_without_a_partial_per_empty_delta() {
    let events = parse(TOOL_CALL);

    let thinking: Vec<bool> = events
        .iter()
        .filter_map(|event| match event {
            ChatBody::AssistantThinking { complete, .. } => Some(*complete),
            _ => None,
        })
        .collect();
    assert_eq!(thinking, vec![false, true]);
}

#[test]
fn text_streamed_in_pieces_ends_as_the_whole_text() {
    let events = parse(DENIED);

    let whole = completed_texts(&events);
    assert_eq!(whole.len(), 1);
    assert_eq!(streamed_text(&events), whole[0]);
}

#[test]
fn a_denied_call_comes_back_as_an_error_result() {
    let events = parse(DENIED);

    assert!(matches!(&events[0], ChatBody::ToolUse { name, input: ToolInput::Bash { command, .. }, .. } if name == "Bash" && command == "touch created.txt"));
    assert!(matches!(&events[1], ChatBody::ToolResult { is_error: true, .. }));
}

#[test]
fn a_permission_request_carries_the_tool_its_input_and_the_ids_to_answer_with() {
    for fixture in [PERMISSION_ALLOW, PERMISSION_DENY] {
        let events = parse(fixture);

        let (ChatBody::ToolUse { tool_use_id, .. }, ChatBody::PermissionRequested { request_id, tool_use_id: asked_about, name, input }) =
            (&events[0], &events[1])
        else {
            panic!("the call, then the request to run it");
        };
        assert_eq!(asked_about, tool_use_id);
        assert_eq!(name, "Bash");
        assert!(matches!(input, ToolInput::Bash { command, .. } if command.starts_with("touch created_")));
        assert_eq!(request_id.len(), 36);
    }
}

#[test]
fn an_allowed_call_runs_and_a_denied_one_comes_back_as_an_error() {
    assert!(matches!(&parse(PERMISSION_ALLOW)[2], ChatBody::ToolResult { is_error: false, .. }));
    assert!(matches!(&parse(PERMISSION_DENY)[2], ChatBody::ToolResult { is_error: true, .. }));
}

#[test]
fn an_interrupted_turn_ends_as_interrupted_and_the_next_one_is_completed() {
    let outcomes: Vec<TurnOutcome> = turns(&parse(INTERRUPTED)).into_iter().map(|(_, outcome)| outcome).collect();

    assert_eq!(outcomes, vec![TurnOutcome::Interrupted, TurnOutcome::Completed]);
}

#[test]
fn each_api_retry_is_a_notice_with_its_attempt() {
    let events = parse(ERROR_RETRY);

    assert_eq!(events.len(), 5);
    assert!(events.iter().all(|event| matches!(event, ChatBody::Notice { kind: NoticeKind::ApiRetry, .. })));
    assert!(matches!(&events[4], ChatBody::Notice { text, .. } if text.contains("Retry 5 in 10s")));
}

#[test]
fn a_line_that_cannot_be_read_becomes_a_warning_and_parsing_goes_on() {
    let mut parser = StreamParser::new();

    let garbled = parser.parse_line("{\"type\": \"assistant\", \"message\": ");
    let not_an_event = parser.parse_line("[1, 2, 3]");
    let after = parser.parse_line(&json!({"type": "conversation_reset"}).to_string());

    assert!(matches!(garbled.as_slice(), [ChatBody::Notice { kind: NoticeKind::Warning, .. }]));
    assert!(matches!(not_an_event.as_slice(), [ChatBody::Notice { kind: NoticeKind::Warning, .. }]));
    assert_eq!(after.len(), 1);
}

#[test]
fn blank_lines_and_lines_of_a_type_nobody_shows_produce_nothing() {
    let mut parser = StreamParser::new();

    assert!(parser.parse_line("").is_empty());
    assert!(parser.parse_line("   \n").is_empty());
    assert!(parser.parse_line(&json!({"type": "rate_limit_event", "rate_limit_info": {}}).to_string()).is_empty());
    assert!(parser.parse_line(&json!({"type": "system", "subtype": "init", "session_id": "s"}).to_string()).is_empty());
}

#[test]
fn deltas_belong_to_the_message_that_started_last() {
    let events = parse_lines(&[
        json!({"type": "stream_event", "event": {"type": "message_start", "message": {"id": "msg_a"}}}),
        json!({"type": "stream_event", "event": {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "one"}}}),
        json!({"type": "stream_event", "event": {"type": "message_start", "message": {"id": "msg_b"}}}),
        json!({"type": "stream_event", "event": {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "two"}}}),
    ]);

    assert_eq!(
        events,
        vec![
            ChatBody::AssistantText { message_id: "msg_a".into(), text: "one".into(), complete: false },
            ChatBody::AssistantText { message_id: "msg_b".into(), text: "two".into(), complete: false },
        ]
    );
}

#[test]
fn a_delta_before_any_message_started_is_dropped() {
    let events = parse_lines(&[json!({"type": "stream_event", "event": {"type": "content_block_delta", "delta": {"type": "text_delta", "text": "x"}}})]);

    assert!(events.is_empty());
}

#[test]
fn two_text_blocks_of_one_message_complete_as_one_text() {
    let line = |text: &str| json!({"type": "assistant", "message": {"id": "msg_a", "content": [{"type": "text", "text": text}]}});

    let events = parse_lines(&[line("first"), line("second")]);

    assert_eq!(completed_texts(&events), vec!["first", "first\n\nsecond"]);
}

#[test]
fn empty_content_and_blocks_of_an_unknown_type_produce_nothing() {
    let events = parse_lines(&[
        json!({"type": "assistant", "message": {"id": "msg_a", "content": []}}),
        json!({"type": "assistant", "message": {"id": "msg_a", "content": [{"type": "text", "text": ""}]}}),
        json!({"type": "assistant", "message": {"id": "msg_a", "content": [{"type": "server_tool_use", "id": "x"}]}}),
    ]);

    assert!(events.is_empty());
}

#[test]
fn a_huge_tool_output_is_cut_to_an_excerpt() {
    let output = "x".repeat(TOOL_EXCERPT_MAX_CHARACTERS * 2);

    let events = parse_lines(&[json!({
        "type": "user",
        "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "toolu_1", "content": output}]}
    })]);

    let [ChatBody::ToolResult { excerpt, truncated, .. }] = events.as_slice() else { panic!("one result") };
    assert_eq!(excerpt.chars().count(), TOOL_EXCERPT_MAX_CHARACTERS);
    assert!(truncated);
}

#[test]
fn a_tool_output_in_parts_is_read_as_the_text_of_its_parts() {
    let events = parse_lines(&[json!({
        "type": "user",
        "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "toolu_1", "is_error": true, "content": [{"type": "text", "text": "a"}, {"type": "image"}, {"type": "text", "text": "b"}]}]}
    })]);

    assert_eq!(events, vec![ChatBody::tool_result("toolu_1", true, "a\nb")]);
}

#[test]
fn replayed_and_plain_user_lines_are_not_events() {
    let events = parse_lines(&[
        json!({"type": "user", "isReplay": true, "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "t", "content": "old"}]}}),
        json!({"type": "user", "message": {"role": "user", "content": "[Request interrupted by user]"}}),
    ]);

    assert!(events.is_empty());
}

#[test]
fn a_result_without_counters_finishes_the_turn_with_zeros_and_an_error_fails_it() {
    let events = parse_lines(&[
        json!({"type": "result", "subtype": "success", "is_error": false}),
        json!({"type": "result", "subtype": "error_during_execution", "is_error": true, "duration_ms": 12}),
    ]);

    assert_eq!(
        turns(&events),
        vec![(TurnUsage::default(), TurnOutcome::Completed), (TurnUsage::default(), TurnOutcome::Failed)]
    );
}

#[test]
fn a_compaction_and_a_cleared_conversation_are_notices() {
    let events = parse_lines(&[
        json!({"type": "system", "subtype": "compact_boundary", "session_id": "s"}),
        json!({"type": "conversation_reset", "new_conversation_id": "c"}),
    ]);

    assert!(matches!(&events[0], ChatBody::Notice { kind: NoticeKind::Compaction, .. }));
    assert!(matches!(&events[1], ChatBody::Notice { kind: NoticeKind::Warning, .. }));
}

#[test]
fn a_control_request_other_than_a_permission_prompt_is_ignored() {
    let events = parse_lines(&[json!({"type": "control_request", "request_id": "r", "request": {"subtype": "hook_callback"}})]);

    assert!(events.is_empty());
}
