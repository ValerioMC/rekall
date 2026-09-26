use rekall_service::Services;
use tokio::sync::{oneshot, watch};

/// A running server: the port it answers on, the services of the instance currently up, and the
/// way to stop it.
pub struct Running {
    pub port: u16,
    pub(super) current: watch::Receiver<Option<(u64, Services)>>,
    pub(super) stop: Option<oneshot::Sender<()>>,
    pub(super) supervisor: Option<tokio::task::JoinHandle<()>>,
}

impl Running {
    /// The services of the instance up right now; `None` in the gap of a restart.
    pub fn services(&self) -> Option<Services> {
        self.current.borrow().as_ref().map(|(_, services)| services.clone())
    }

    /// Which start of the application is up: 1 for the first, one more after every restart.
    pub fn generation(&self) -> u64 {
        self.current.borrow().as_ref().map(|(generation, _)| *generation).unwrap_or(0)
    }

    /// Wait until a start later than `generation` is up (a restart has gone round).
    pub async fn instance_after(&mut self, generation: u64) -> Option<Services> {
        let found = self
            .current
            .wait_for(|current| current.as_ref().is_some_and(|(g, _)| *g > generation))
            .await
            .ok()?;
        found.as_ref().map(|(_, services)| services.clone())
    }

    /// Close the application and wait for it.
    pub async fn stop(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(supervisor) = self.supervisor.take() {
            let _ = supervisor.await;
        }
    }
}
