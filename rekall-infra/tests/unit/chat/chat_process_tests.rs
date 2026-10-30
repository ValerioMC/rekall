use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rekall_core::chat::NoticeKind;
use tempfile::TempDir;

use super::super::chat_command::chat_command;
use super::super::stream_parser::StreamParser;
use super::*;
use crate::session::ClaudeOptions;

const FAKE_CLAUDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fake_claude.sh");
const PLAIN_REPLY: &str = include_str!("../../fixtures/chat/plain_reply.jsonl");
const WAIT: Duration = Duration::from_secs(10);

struct Fake {
    process: ChatProcess,
    events: mpsc::Receiver<ChatBody>,
    received: PathBuf,
    _folder: TempDir,
}

fn start(first_message: &str, behaviour: &[(&str, &str)]) -> Fake {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let fixture = folder.path().join("replay.jsonl");
    std::fs::write(&fixture, PLAIN_REPLY).expect("the fixture is written");
    let received = folder.path().join("received.txt");
    let mut environment = HashMap::from([
        ("PATH".to_string(), "/usr/bin:/bin".to_string()),
        ("FAKE_CLAUDE_FIXTURE".to_string(), fixture.to_string_lossy().into_owned()),
        ("FAKE_CLAUDE_RECEIVED".to_string(), received.to_string_lossy().into_owned()),
    ]);
    environment.extend(behaviour.iter().map(|(key, value)| (key.to_string(), value.to_string())));
    let command = chat_command(Path::new(FAKE_CLAUDE), &ClaudeOptions::chosen(true, None, None), environment, folder.path());
    let (process, events) = spawn_chat_process(Id::random(), command, first_message).expect("the fake claude starts");
    Fake { process, events, received, _folder: folder }
}

fn expected_events() -> Vec<ChatBody> {
    let mut parser = StreamParser::new();
    PLAIN_REPLY.lines().flat_map(|line| parser.parse_line(line)).collect()
}

fn is_turn_end(body: &ChatBody) -> bool {
    matches!(body, ChatBody::TurnFinished { .. })
}

async fn events_until_turns_end(events: &mut mpsc::Receiver<ChatBody>, turns: usize) -> Vec<ChatBody> {
    let mut seen = Vec::new();
    while seen.iter().filter(|body| is_turn_end(body)).count() < turns {
        let body = tokio::time::timeout(WAIT, events.recv()).await.expect("an event arrives in time").expect("the stream is open");
        seen.push(body);
    }
    seen
}

async fn events_until_closed(events: &mut mpsc::Receiver<ChatBody>) -> Vec<ChatBody> {
    let mut seen = Vec::new();
    while let Some(body) = tokio::time::timeout(WAIT, events.recv()).await.expect("the stream closes in time") {
        seen.push(body);
    }
    seen
}

async fn received_lines(path: &Path, count: usize) -> Vec<serde_json::Value> {
    for _ in 0..200 {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let lines: Vec<serde_json::Value> = text.lines().filter_map(|line| serde_json::from_str(line).ok()).collect();
        if lines.len() >= count {
            return lines;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("the fake claude did not receive {count} lines");
}

fn message_text(line: &serde_json::Value) -> &str {
    line["message"]["content"].as_str().expect("a user message carries text")
}

#[tokio::test]
async fn two_messages_in_give_the_events_of_two_turns_out() {
    let mut fake = start("/rk project:x task:y", &[]);
    fake.process.send("and now the second").expect("the process accepts a message");

    let events = events_until_turns_end(&mut fake.events, 2).await;

    assert_eq!(events, expected_events());
    let lines = received_lines(&fake.received, 2).await;
    assert_eq!(message_text(&lines[0]), "/rk project:x task:y");
    assert_eq!(message_text(&lines[1]), "and now the second");
}

#[tokio::test]
async fn an_interrupt_is_written_as_the_control_request() {
    let fake = start("hello", &[]);

    fake.process.interrupt().expect("the process accepts an interrupt");

    let lines = received_lines(&fake.received, 2).await;
    assert_eq!(lines[1]["type"], "control_request");
    assert_eq!(lines[1]["request"]["subtype"], "interrupt");
    assert!(fake.process.alive());
}

#[tokio::test]
async fn a_permission_answer_is_written_as_the_control_response_for_its_request() {
    let fake = start("hello", &[]);

    fake.process.answer_permission("req-9", false).expect("the process accepts an answer");

    let lines = received_lines(&fake.received, 2).await;
    assert_eq!(lines[1]["type"], "control_response");
    assert_eq!(lines[1]["response"]["request_id"], "req-9");
    assert_eq!(lines[1]["response"]["response"]["behavior"], "deny");
}

#[tokio::test]
async fn terminate_ends_a_process_that_is_waiting_for_input() {
    let fake = start("hello", &[]);
    received_lines(&fake.received, 1).await;

    fake.process.terminate();

    assert!(fake.process.wait_for(WAIT).await);
    assert!(!fake.process.alive());
}

#[tokio::test]
async fn kill_ends_a_process_that_is_waiting_for_input() {
    let fake = start("hello", &[]);
    received_lines(&fake.received, 1).await;

    fake.process.kill();

    assert!(fake.process.wait_for(WAIT).await);
    assert_eq!(fake.process.exit_code(), Some(-1));
}

#[tokio::test]
async fn a_process_that_exits_with_an_error_closes_the_stream_and_reports_its_code() {
    let mut fake = start("hello", &[("FAKE_CLAUDE_EXIT_CODE", "3")]);

    let events = events_until_closed(&mut fake.events).await;

    assert!(fake.process.wait_for(WAIT).await);
    assert_eq!(fake.process.exit_code(), Some(3));
    assert_eq!(events.iter().filter(|body| is_turn_end(body)).count(), 1);
}

#[tokio::test]
async fn a_message_to_an_ended_process_is_a_conflict() {
    let fake = start("hello", &[("FAKE_CLAUDE_EXIT_CODE", "0")]);
    assert!(fake.process.wait_for(WAIT).await);

    let refused = fake.process.send("anyone there?").expect_err("the process is gone");

    assert!(refused.to_string().contains("closed"), "{refused}");
}

#[tokio::test]
async fn output_that_stops_mid_line_becomes_a_warning_and_the_stream_still_ends() {
    let mut fake = start("hello", &[("FAKE_CLAUDE_CUT_MID_LINE", "1")]);

    let events = events_until_closed(&mut fake.events).await;

    let last = events.last().expect("the stream has events");
    assert!(matches!(last, ChatBody::Notice { kind: NoticeKind::Warning, .. }), "{last:?}");
    assert!(fake.process.wait_for(WAIT).await);
}

#[tokio::test]
async fn a_binary_that_is_not_there_is_a_conflict() {
    let command = chat_command(Path::new("/nonexistent/claude"), &ClaudeOptions::default(), HashMap::new(), Path::new("/tmp"));

    let refused = spawn_chat_process(Id::random(), command, "hello").err().expect("nothing to start");

    assert!(refused.to_string().contains("Could not start claude"), "{refused}");
}
