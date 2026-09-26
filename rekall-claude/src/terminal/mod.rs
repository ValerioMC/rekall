//! Open, list and close in-app terminals. Bytes do not travel here: a pane connects to the
//! socket at `/api/terminal/{id}/io` once the terminal exists.

mod open_terminal_request;
mod terminal_controller;
mod terminal_mode;

pub use open_terminal_request::OpenTerminalRequest;
pub use terminal_controller::routes;
pub use terminal_mode::TerminalMode;
