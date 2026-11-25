use rekall_core::chat::{ContextFill, TOOL_EXCERPT_MAX_CHARACTERS};
use serde_json::json;

use super::*;

const PLAIN_REPLY: &str = include_str!("../../fixtures/chat/plain_reply.jsonl");
const TOOL_CALL: &str = include_str!("../../fixtures/chat/tool_call_result.jsonl");
const DENIED: &str = include_str!("../../fixtures/chat/streaming_deltas_denied.jsonl");
const PERMISSION_ALLOW: &str = include_str!("../../fixtures/chat/permission_request_allow.jsonl");
const PERMISSION_DENY: &str = include_str!("../../fixtures/chat/permission_request_deny.jsonl");
const INTERRUPTED: &str = include_str!("../../fixtures/chat/interrupted_turn.jsonl");
const ERROR_RETRY: &str = include_str!("../../fixtures/chat/error_retry.jsonl");
const TOOL_READ: &str = include_str!("../../fixtures/chat/tool_read.jsonl");
const TOOL_WRITE: &str = include_str!("../../fixtures/chat/tool_write.jsonl");
const TOOL_EDIT: &str = include_str!("../../fixtures/chat/tool_edit.jsonl");
const TOOL_GREP: &str = include_str!("../../fixtures/chat/tool_grep.jsonl");
const TOOL_GLOB: &str = include_str!("../../fixtures/chat/tool_glob.jsonl");
const TOOL_BASH: &str = include_str!("../../fixtures/chat/tool_bash.jsonl");
const TOOL_BASH_FAILED: &str = include_str!("../../fixtures/chat/tool_bash_failed.jsonl");
const TOOL_TODO_WRITE: &str = include_str!("../../fixtures/chat/tool_todo_write.jsonl");
const TOOL_WEB_SEARCH: &str = include_str!("../../fixtures/chat/tool_web_search.jsonl");
const SUBAGENT: &str = include_str!("../../fixtures/chat/subagent_run.jsonl");
const MULTI_CALL_TURN: &str = include_str!("../../fixtures/chat/multi_call_turn.jsonl");
const LOCAL_COMMAND: &str = include_str!("../../fixtures/chat/local_command_context.jsonl");

fn parse_all(fixture: &str) -> Vec<ChatBody> {
    let mut parser = StreamParser::new();
    fixture.lines().flat_map(|line| parser.parse_line(line)).collect()
}

fn parse(fixture: &str) -> Vec<ChatBody> {
    parse_all(fixture).into_iter().filter(|event| !matches!(event, ChatBody::SessionStarted { .. })).collect()
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

    let ChatBody::ToolUse { tool_use_id, name, input, .. } = &events[0] else { panic!("the tool call comes first") };
    assert_eq!(name, "Read");
    assert_eq!(*input, ToolInput::Read { file_path: "/work/project/hello.txt".into() });
    let ChatBody::ToolResult { tool_use_id: answered, is_error, excerpt, truncated, .. } = &events[1] else { panic!("its result follows") };
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
            ChatBody::AssistantText { message_id: "msg_a".into(), text: "one".into(), complete: false, parent_tool_use_id: None },
            ChatBody::AssistantText { message_id: "msg_b".into(), text: "two".into(), complete: false, parent_tool_use_id: None },
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

    assert_eq!(events, vec![ChatBody::tool_result("toolu_1", true, "a\nb", None, None)]);
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

#[test]
fn the_init_line_announces_the_session_with_its_model_directory_mode_and_version() {
    let events = parse_all(PLAIN_REPLY);

    assert_eq!(
        events[0],
        ChatBody::SessionStarted {
            model: "claude-haiku-5-5".into(),
            cwd: "/work/project".into(),
            permission_mode: "default".into(),
            version: "2.1.314".into(),
        }
    );
}

#[test]
fn an_init_repeated_on_every_turn_announces_the_session_once() {
    let events = parse_all(PLAIN_REPLY);

    let announcements = events.iter().filter(|event| matches!(event, ChatBody::SessionStarted { .. })).count();
    assert_eq!(announcements, 1);
}

#[test]
fn an_init_that_changed_the_model_announces_the_session_again() {
    let init = |model: &str| json!({"type": "system", "subtype": "init", "model": model, "cwd": "/w", "permissionMode": "default", "claude_code_version": "2"});

    let events = parse_lines(&[init("a"), init("a"), init("b")]);

    assert_eq!(events.len(), 2);
}

#[test]
fn a_local_command_prints_its_output_as_one_event_without_the_tags_and_no_assistant_text() {
    let events = parse(LOCAL_COMMAND);

    let outputs: Vec<&ChatBody> = events.iter().filter(|event| matches!(event, ChatBody::CommandOutput { .. })).collect();
    assert_eq!(outputs.len(), 1);
    let ChatBody::CommandOutput { command, text } = outputs[0] else { panic!("a command output") };
    assert_eq!(command, "/context");
    assert!(text.starts_with("## Context Usage"));
    assert!(!text.contains("local-command-stdout"));
    assert!(completed_texts(&events).is_empty());
}

#[test]
fn a_local_command_with_no_output_produces_nothing() {
    let events = parse_lines(&[json!({
        "type": "assistant",
        "message": {"id": "m", "content": [{"type": "text", "text": ""}]},
        "local_command_source": "<local-command-stdout></local-command-stdout>",
        "local_command_run": {"command": "compact", "args": ""}
    })]);

    assert!(events.is_empty());
}

fn tool_summary(fixture: &str) -> Option<String> {
    parse(fixture).into_iter().find_map(|event| match event {
        ChatBody::ToolResult { summary, .. } => Some(summary),
        _ => None,
    })?
}

#[test]
fn each_tool_result_says_in_one_line_what_the_tool_did() {
    assert_eq!(tool_summary(TOOL_READ).as_deref(), Some("Read 4 lines"));
    assert_eq!(tool_summary(TOOL_WRITE).as_deref(), Some("Added 3 lines, removed 0"));
    assert_eq!(tool_summary(TOOL_EDIT).as_deref(), Some("Added 1 line, removed 1"));
    assert_eq!(tool_summary(TOOL_GREP).as_deref(), Some("1 match"));
    assert_eq!(tool_summary(TOOL_GLOB).as_deref(), Some("1 file"));
    assert_eq!(tool_summary(TOOL_BASH).as_deref(), Some("exit code 0"));
}

#[test]
fn a_failed_command_says_its_exit_code() {
    assert_eq!(tool_summary(TOOL_BASH_FAILED).as_deref(), Some("exit code 3"));
}

#[test]
fn an_interrupted_command_says_so() {
    let events = parse_lines(&[json!({
        "type": "user",
        "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "t", "content": "partial"}]},
        "tool_use_result": {"stdout": "partial", "stderr": "", "interrupted": true, "isImage": false, "noOutputExpected": false}
    })]);

    assert!(matches!(&events[0], ChatBody::ToolResult { summary: Some(summary), .. } if summary == "interrupted"));
}

#[test]
fn a_result_of_a_shape_nobody_summarises_or_with_no_report_has_no_summary() {
    assert_eq!(tool_summary(TOOL_TODO_WRITE), None);
    assert_eq!(tool_summary(TOOL_WEB_SEARCH), None);
    let events = parse_lines(&[json!({"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "t", "content": "x"}]}})]);
    assert!(matches!(&events[0], ChatBody::ToolResult { summary: None, .. }));
}

#[test]
fn a_report_shared_by_several_results_is_not_taken_for_any_of_them() {
    let events = parse_lines(&[json!({
        "type": "user",
        "message": {"role": "user", "content": [
            {"type": "tool_result", "tool_use_id": "a", "content": "x"},
            {"type": "tool_result", "tool_use_id": "b", "content": "y"}
        ]},
        "tool_use_result": {"file": {"numLines": 3}}
    })]);

    assert!(events.iter().all(|event| matches!(event, ChatBody::ToolResult { summary: None, .. })));
}

#[test]
fn the_tools_the_terminal_draws_in_its_own_way_are_typed_and_not_other() {
    let todo_call = parse(TOOL_TODO_WRITE).into_iter().find(|event| matches!(event, ChatBody::ToolUse { .. })).expect("the call");
    let web_call = parse(TOOL_WEB_SEARCH).into_iter().find(|event| matches!(event, ChatBody::ToolUse { .. })).expect("the call");

    assert!(matches!(todo_call, ChatBody::ToolUse { input: ToolInput::Todos { ref items }, .. } if items.len() == 2));
    assert!(matches!(web_call, ChatBody::ToolUse { input: ToolInput::Web { ref target, .. }, .. } if target == "rust serde tutorial"));
}

fn parent_of(event: &ChatBody) -> Option<&str> {
    match event {
        ChatBody::AssistantText { parent_tool_use_id, .. }
        | ChatBody::AssistantThinking { parent_tool_use_id, .. }
        | ChatBody::ToolUse { parent_tool_use_id, .. }
        | ChatBody::ToolResult { parent_tool_use_id, .. } => parent_tool_use_id.as_deref(),
        _ => None,
    }
}

#[test]
fn what_a_subagent_does_names_the_agent_call_that_started_it_and_the_main_conversation_does_not() {
    let events = parse(SUBAGENT);
    let ChatBody::ToolUse { tool_use_id: agent_call, input: ToolInput::Agent { .. }, parent_tool_use_id: None, .. } = &events[1] else {
        panic!("the agent call comes second, in the main conversation")
    };

    let inner: Vec<&ChatBody> = events.iter().filter(|event| parent_of(event).is_some()).collect();
    assert_eq!(inner.len(), 2);
    assert!(matches!(inner[0], ChatBody::ToolUse { input: ToolInput::Web { .. }, .. }));
    assert!(matches!(inner[1], ChatBody::ToolResult { .. }));
    assert!(inner.iter().all(|event| parent_of(event) == Some(agent_call.as_str())));
    assert!(matches!(&events[0], ChatBody::AssistantText { parent_tool_use_id: None, .. }));
    assert!(matches!(events.last(), Some(ChatBody::AssistantText { parent_tool_use_id: None, .. })));
}

#[test]
fn the_agents_own_result_belongs_to_the_main_conversation() {
    let events = parse(SUBAGENT);

    let answer = events.iter().find(|event| matches!(event, ChatBody::ToolResult { parent_tool_use_id: None, .. }));
    assert!(answer.is_some());
}

#[test]
fn a_subagents_text_is_not_joined_to_the_text_of_the_main_message() {
    let assistant = |id: &str, parent: Option<&str>, text: &str| {
        json!({"type": "assistant", "parent_tool_use_id": parent, "message": {"id": id, "content": [{"type": "text", "text": text}]}})
    };

    let events = parse_lines(&[assistant("main", None, "first"), assistant("sub", Some("toolu_A"), "inner"), assistant("main", None, "second")]);

    assert_eq!(completed_texts(&events), vec!["first", "inner", "first\n\nsecond"]);
    assert_eq!(parent_of(&events[1]), Some("toolu_A"));
}

#[test]
fn streamed_deltas_of_a_subagent_carry_its_parent() {
    let events = parse_lines(&[
        json!({"type": "stream_event", "parent_tool_use_id": "toolu_A", "event": {"type": "message_start", "message": {"id": "sub"}}}),
        json!({"type": "stream_event", "parent_tool_use_id": "toolu_A", "event": {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "hi"}}}),
        json!({"type": "stream_event", "parent_tool_use_id": null, "event": {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "yo"}}}),
    ]);

    assert_eq!(parent_of(&events[0]), Some("toolu_A"));
    assert_eq!(parent_of(&events[1]), None);
}

fn finished_turn(events: &[ChatBody]) -> (Option<f64>, Option<ContextFill>) {
    events
        .iter()
        .find_map(|event| match event {
            ChatBody::TurnFinished { cost_usd, context, .. } => Some((*cost_usd, *context)),
            _ => None,
        })
        .expect("a finished turn")
}

#[test]
fn a_turn_of_many_calls_reports_its_cost_and_what_its_last_call_held() {
    let (cost, context) = finished_turn(&parse(MULTI_CALL_TURN));

    assert!((cost.expect("a cost") - 0.01015964).abs() < 1e-9);
    assert_eq!(context, Some(ContextFill { used_tokens: 2 + 37_070 + 937, window_tokens: 1_000_000 }));
}

#[test]
fn a_result_without_a_cost_or_iterations_has_neither() {
    let events = parse_lines(&[
        json!({"type": "result", "subtype": "success", "is_error": false, "usage": {"input_tokens": 1}}),
        json!({"type": "result", "subtype": "success", "is_error": false, "usage": {"iterations": []}, "modelUsage": {}}),
        json!({"type": "result", "subtype": "success", "is_error": false, "usage": {"iterations": null}}),
    ]);

    for event in &events {
        assert!(matches!(event, ChatBody::TurnFinished { cost_usd: None, context: None, .. }));
    }
    assert_eq!(events.len(), 3);
}

#[test]
fn the_window_is_the_one_of_the_model_that_wrote_most() {
    let events = parse_lines(&[json!({
        "type": "result", "subtype": "success", "is_error": false, "total_cost_usd": 0.5,
        "usage": {"iterations": [{"input_tokens": 10, "cache_read_input_tokens": 20, "cache_creation_input_tokens": 30}]},
        "modelUsage": {
            "small-model": {"outputTokens": 5, "contextWindow": 200_000},
            "main-model": {"outputTokens": 900, "contextWindow": 1_000_000}
        }
    })]);

    assert_eq!(finished_turn(&events).1, Some(ContextFill { used_tokens: 60, window_tokens: 1_000_000 }));
}

#[test]
fn a_model_that_names_no_window_leaves_the_fill_unknown() {
    let events = parse_lines(&[json!({
        "type": "result", "subtype": "success", "is_error": false,
        "usage": {"iterations": [{"input_tokens": 10}]},
        "modelUsage": {"m": {"outputTokens": 5}}
    })]);

    assert_eq!(finished_turn(&events).1, None);
}
