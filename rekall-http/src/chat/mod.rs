//! The routes of a chat session: `GET /api/chat/{id}/events` streams its events as Server-Sent
//! Events, and a message, a stop and the answer to a permission prompt are posted to it. Opening,
//! listing and closing one are the terminal routes, which take a `kind`. Served by
//! `rekall_console::chat`.
//!
//! Each event is a frame whose SSE `id` is its `seq`, so an `EventSource` that reconnects sends
//! `Last-Event-ID` and gets only what came after; `?after=` asks the same on the first connection.
//! A keep-alive comment is sent every 15 seconds. Frames are queued with a cap: a reader that falls
//! too far behind is dropped rather than left to back up memory. Like the terminal socket, the
//! request is only accepted from a loopback origin, or from a client that sends none.

mod controller;
mod request;
mod routes;
mod support;

pub(crate) use routes::{open_routes, routes};
