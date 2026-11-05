use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::serve::Listener;
use tokio::net::{TcpListener, TcpStream};
use tracing::warn;

/// How long accepting waits after a failure, so a socket that keeps failing is not spun on.
const ACCEPT_BACKOFF: Duration = Duration::from_millis(50);

/// The one listening socket, handed to each generation of the server. A cloned OS socket
/// (`try_clone`) never accepts on Windows, so the same `TcpListener` is shared instead.
pub struct SharedListener(pub Arc<TcpListener>);

impl Listener for SharedListener {
    type Io = TcpStream;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            match self.0.accept().await {
                Ok(accepted) => return accepted,
                // A connection reset before it was accepted must not stop the server.
                Err(failure) => {
                    warn!(%failure, "Could not accept a connection");
                    tokio::time::sleep(ACCEPT_BACKOFF).await;
                }
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.0.local_addr()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/server/shared_listener_tests.rs"]
mod tests;
