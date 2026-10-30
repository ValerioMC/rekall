use serde_json::json;

use super::*;

fn parsed(line: &str) -> serde_json::Value {
    serde_json::from_str(line).expect("the line is json")
}

#[test]
fn a_user_message_is_one_line_with_the_text_as_its_content() {
    let written = user_message_line("say \"hi\"\nthen stop");
    assert!(!written.contains('\n'));
    assert_eq!(parsed(&written), json!({"type": "user", "message": {"role": "user", "content": "say \"hi\"\nthen stop"}}));
}

#[test]
fn an_interrupt_is_the_control_request_that_keeps_the_process_alive() {
    assert_eq!(
        parsed(&interrupt_line("int-3")),
        json!({"type": "control_request", "request_id": "int-3", "request": {"subtype": "interrupt"}})
    );
}

#[test]
fn allowing_a_call_answers_its_request_with_allow() {
    assert_eq!(
        parsed(&permission_answer_line("req-1", true)),
        json!({"type": "control_response", "response": {"subtype": "success", "request_id": "req-1", "response": {"behavior": "allow"}}})
    );
}

#[test]
fn denying_a_call_tells_the_model_it_was_refused() {
    assert_eq!(
        parsed(&permission_answer_line("req-2", false)),
        json!({"type": "control_response", "response": {
            "subtype": "success", "request_id": "req-2", "response": {"behavior": "deny", "message": "The user denied this tool call."}
        }})
    );
}
