//! The routes of the in-app terminals: open, list and close one, served by
//! `rekall_console::terminal`, and the socket at `/api/terminal/{id}/io` that pipes its bytes once
//! it exists.
//!
//! The socket is Rekall's one WebSocket. Binary frames are stdin, `{"resize":[cols,rows]}` text
//! frames set the window size, `{"in":"<base64>"}` is stdin too, PTY output returns as binary
//! frames, and a `{"type":"ended"}` frame is sent on exit. A browser does not apply CORS to a
//! WebSocket, so the handshake is only accepted from a loopback origin (or from a client that sends
//! none). Outbound frames are queued with a cap: a pane that stops reading is dropped rather than
//! left to back up memory.

mod controller;
mod request;
mod support;
mod routes;

pub use request::open_terminal_request::OpenTerminalRequest;
pub use controller::socket_controller::is_allowed_origin;
pub use request::terminal_mode_name::TerminalModeName;
pub(crate) use routes::{routes, socket_routes};
