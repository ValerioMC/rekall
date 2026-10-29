//! Cleanup the caller cannot act on when it fails: a scratch file, a child process past its
//! deadline. A failure is logged rather than discarded, so a leak shows in the log; a target that
//! is already gone is not a failure.

use std::io::ErrorKind;
use std::path::Path;
use std::process::Child;

use tracing::{debug, warn};

pub fn remove_file(path: &Path) {
    if let Err(failed) = std::fs::remove_file(path) {
        if failed.kind() != ErrorKind::NotFound {
            warn!(%failed, path = %path.display(), "Could not delete a leftover file");
        }
    }
}

pub fn remove_dir_all(path: &Path) {
    if let Err(failed) = std::fs::remove_dir_all(path) {
        if failed.kind() != ErrorKind::NotFound {
            warn!(%failed, path = %path.display(), "Could not delete a leftover folder");
        }
    }
}

/// Kills the child and reaps it. A child that exited on its own in the meantime is reaped all the same.
pub fn kill(child: &mut Child) {
    if let Err(failed) = child.kill() {
        debug!(%failed, pid = child.id(), "Kill found the child already gone");
    }
    if let Err(failed) = child.wait() {
        warn!(%failed, pid = child.id(), "Could not reap a killed child");
    }
}

#[cfg(test)]
#[path = "../tests/unit/cleanup_tests.rs"]
mod tests;
