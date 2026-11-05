use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::serve::Listener;
use tokio::net::{TcpListener, TcpStream};

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
                // Per-connection failures (a reset before accept) must not stop the server.
                Err(_) => tokio::time::sleep(std::time::Duration::from_millis(1)).await,
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.0.local_addr()
    }
}
