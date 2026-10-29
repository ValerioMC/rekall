use axum::extract::ws::Message;

pub(in crate::terminal) enum Outbound {
    Frame(Message),
    Close(u16, String),
}
