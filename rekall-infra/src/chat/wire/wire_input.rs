use serde::Serialize;

/// One line written to the stdin of a headless `claude`.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WireInput<'a> {
    User { message: WireUserText<'a> },
    ControlRequest { request_id: &'a str, request: WireInterrupt },
    ControlResponse { response: WireAnswer<'a> },
}

#[derive(Serialize)]
struct WireUserText<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Serialize)]
#[serde(tag = "subtype", rename_all = "snake_case")]
enum WireInterrupt {
    Interrupt,
}

#[derive(Serialize)]
struct WireAnswer<'a> {
    subtype: &'static str,
    request_id: &'a str,
    response: WireDecision,
}

#[derive(Serialize)]
#[serde(tag = "behavior", rename_all = "snake_case")]
enum WireDecision {
    Allow,
    Deny { message: &'static str },
}

const DENIED_MESSAGE: &str = "The user denied this tool call.";

/// A user message, which `claude` queues if a turn is running.
pub(in crate::chat) fn user_message_line(text: &str) -> String {
    line(&WireInput::User { message: WireUserText { role: "user", content: text } })
}

/// Ends the running turn and leaves the process answering the next message. SIGINT ends the
/// process itself.
pub(in crate::chat) fn interrupt_line(request_id: &str) -> String {
    line(&WireInput::ControlRequest { request_id, request: WireInterrupt::Interrupt })
}

/// The answer to a `can_use_tool` request: the call runs as asked, or the model is told it was refused.
pub(in crate::chat) fn permission_answer_line(request_id: &str, allow: bool) -> String {
    let response = if allow { WireDecision::Allow } else { WireDecision::Deny { message: DENIED_MESSAGE } };
    line(&WireInput::ControlResponse { response: WireAnswer { subtype: "success", request_id, response } })
}

fn line(input: &WireInput) -> String {
    serde_json::to_string(input).expect("a message of plain strings always serialises")
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/wire/wire_input_tests.rs"]
mod tests;
