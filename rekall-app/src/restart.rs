//! `ApplicationRestarter`: switching or restoring a database restarts the application in the same
//! process, so the next start opens what the registry now names. The supervisor in `crate::server`
//! owns the loop; handlers hold a `Restarter` that asks it to go round once more.

use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// A hook run around the moment nothing holds the database.
pub type Hook = Box<dyn FnOnce() -> Result<(), String> + Send>;

/// What the supervisor is asked to do: close with `before_close` still able to reach the
/// database, then run `after_close` before the next start (to swap its file).
pub struct RestartRequest {
    pub before_close: Hook,
    pub after_close: Hook,
}

/// The wait before closing, so the response that asked for the restart gets out first.
pub const RESTART_DELAY: Duration = Duration::from_millis(400);

#[derive(Clone, Default)]
pub struct Restarter {
    requests: Option<mpsc::UnboundedSender<RestartRequest>>,
}

impl Restarter {
    /// One that the supervisor listens to.
    pub fn channel() -> (Self, mpsc::UnboundedReceiver<RestartRequest>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (Self { requests: Some(sender) }, receiver)
    }

    /// One nothing listens to: an application that was not started by the supervisor, as a test.
    pub fn disabled() -> Self {
        Self { requests: None }
    }

    /// Whether a restart can be scheduled at all.
    pub fn can_restart(&self) -> bool {
        self.requests.as_ref().is_some_and(|sender| !sender.is_closed())
    }

    pub fn restart(&self) -> bool {
        self.restart_with(Box::new(|| Ok(())), Box::new(|| Ok(())))
    }

    /// Returns whether a restart was scheduled; false where nothing can restart.
    pub fn restart_with(&self, before_close: Hook, after_close: Hook) -> bool {
        let Some(sender) = &self.requests else {
            warn!("Restart requested before the application registered itself; ignoring (test context?)");
            return false;
        };
        sender.send(RestartRequest { before_close, after_close }).is_ok()
    }
}

/// A hook that fails is logged and the restart goes on, so the application always comes back.
pub fn run_hook(when: &str, hook: Hook) {
    if let Err(failed) = hook() {
        error!("The restart hook {when} failed; restarting anyway: {failed}");
    }
}

pub fn announce() {
    info!("Restarting to pick up a database change");
}
