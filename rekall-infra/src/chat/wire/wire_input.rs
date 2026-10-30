use serde::Serialize;

/// One line written to the stdin of a headless `claude`.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WireInput<'a> {
    User { message: WireUserText<'a> },
    ControlRequest { request_id: &'a str, request: WireInterrupt },
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

/// A user message, which `claude` queues if a turn is running.
pub(in crate::chat) fn user_message_line(text: &str) -> String {
    line(&WireInput::User { message: WireUserText { role: "user", content: text } })
}

/// Ends the running turn and leaves the process answering the next message. SIGINT ends the
/// process itself.
pub(in crate::chat) fn interrupt_line(request_id: &str) -> String {
    line(&WireInput::ControlRequest { request_id, request: WireInterrupt::Interrupt })
}

fn line(input: &WireInput) -> String {
    serde_json::to_string(input).expect("a message of plain strings always serialises")
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/wire/wire_input_tests.rs"]
mod tests;
