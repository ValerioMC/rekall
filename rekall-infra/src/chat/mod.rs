//! The chat view's side of the machine: reading what a headless `claude` prints. `StreamParser`
//! turns each stdout line of `claude -p --output-format stream-json` into the chat events it means;
//! the `wire` types are the shapes of those lines and go no further.

mod stream_parser;
mod wire;

pub use stream_parser::StreamParser;
