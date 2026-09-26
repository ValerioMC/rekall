use axum::extract::ws::Message;

pub(super) enum Outbound {
    Frame(Message),
    Close(u16, String),
}
