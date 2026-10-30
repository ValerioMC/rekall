//! The chat view's side of the machine: reading what a headless `claude` prints. `StreamParser`
//! turns each stdout line of `claude -p --output-format stream-json` into the chat events it means;
//! the `wire` types are the shapes of the lines on both pipes and go no further. `chat_command` and
//! `chat_process` start the headless `claude` and carry its lines in and out.

mod chat_command;
mod chat_process;
mod chat_session;
mod chat_session_manager;
mod event_journal;
mod stream_parser;
mod wire;

pub use chat_command::chat_command;
pub use chat_process::{spawn_chat_process, ChatProcess};
pub use chat_session_manager::ChatSessionManager;
pub use stream_parser::StreamParser;
