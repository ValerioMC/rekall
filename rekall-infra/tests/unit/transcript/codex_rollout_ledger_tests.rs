use rekall_core::token::TokenCount;
use serde_json::json;

use super::*;

fn line(ledger: &mut CodexRolloutLedger, kind: &str, payload: serde_json::Value) {
    ledger.read_line(&json!({ "timestamp": "2026-11-26T15:10:35Z", "type": kind, "payload": payload }).to_string());
}

fn usage(ledger: &mut CodexRolloutLedger, input: u64, cached: u64, write: u64, output: u64) {
    line(ledger, "event_msg", json!({ "type": "token_count", "info": { "total_token_usage": {
        "input_tokens": input, "cached_input_tokens": cached, "cache_write_input_tokens": write,
        "output_tokens": output, "reasoning_output_tokens": 3, "total_tokens": input + output
    } } }));
}

#[test]
fn cumulative_counts_follow_a_task_and_its_step_without_counting_replayed_events() {
    let mut ledger = CodexRolloutLedger::default();
    usage(&mut ledger, 10, 0, 0, 2);
    line(&mut ledger, "response_item", json!({
        "type": "message", "role": "user",
        "content": [{ "type": "input_text", "text": "Use the $rekall skill with this exact argument line: task:VE-1" }]
    }));
    line(&mut ledger, "response_item", json!({
        "type": "function_call", "call_id": "step-2", "name": "mcp__rekall__rekall_step",
        "arguments": r#"{"anchors":"task:VE-1","state":"running","step":"2"}"#
    }));
    line(&mut ledger, "response_item", json!({ "type": "function_call_output", "call_id": "step-2", "output": [
        { "type": "input_text", "text": "Wall time: 0.002 seconds\nOutput:" },
        { "type": "input_text", "text": "Step 2 \"Work\" on `task:VE-1` is now `running`." }
    ] }));
    usage(&mut ledger, 110, 30, 10, 12);
    usage(&mut ledger, 110, 30, 10, 12);
    usage(&mut ledger, 200, 60, 10, 24);
    usage(&mut ledger, 110, 30, 10, 12);
    line(&mut ledger, "response_item", json!({
        "type": "function_call", "call_id": "claim", "name": "rekall_step", "namespace": "mcp__rekall",
        "arguments": r#"{"anchors":"task:VE-1","state":"claimed","step":"2"}"#
    }));
    line(&mut ledger, "response_item", json!({ "type": "function_call_output", "call_id": "claim", "output": { "success": false } }));
    usage(&mut ledger, 210, 70, 10, 25);

    let spent = ledger.spent();
    assert_eq!(spent.len(), 2);
    assert_eq!(spent[0].unit, WorkUnit::task(None, "VE-1"));
    assert_eq!(spent[0].tokens, TokenCount { input: 60, cache_read: 30, cache_creation: 10, output: 10 });
    assert_eq!(spent[1].unit.step.as_deref(), Some("2"));
    assert_eq!(spent[1].tokens, TokenCount { input: 60, cache_read: 40, cache_creation: 0, output: 13 });
}

#[test]
fn missing_or_unattributed_usage_produces_no_reading() {
    let mut ledger = CodexRolloutLedger::default();
    ledger.read_line("incomplete record");
    line(&mut ledger, "event_msg", json!({ "type": "token_count", "info": null }));
    usage(&mut ledger, 5, 0, 0, 1);
    assert!(ledger.spent().is_empty());
}
