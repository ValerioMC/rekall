//! The byte pipe for one open terminal pane, Rekall's one WebSocket. Binary frames are stdin,
//! `{"resize":[cols,rows]}` text frames set the window size, `{"in":"<base64>"}` is stdin too,
//! PTY output returns as binary frames, and a `{"type":"ended"}` frame is sent on exit.
//!
//! A browser does not apply CORS to a WebSocket, so the handshake is only accepted from a
//! loopback origin (or from a client that sends none). Outbound frames are queued with a cap: a
//! pane that stops reading is dropped rather than left to back up memory.

mod outbound;
mod socket_controller;
mod socket_listener;

pub use socket_controller::{routes, is_allowed_origin};
