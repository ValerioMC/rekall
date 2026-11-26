use std::sync::{Mutex, MutexGuard};

use rekall_core::picture::{PictureCleanupState, PictureCleanupStreamEvent};
use rekall_core::EventBus;
use rekall_kernel::{RekallError, Result};

use crate::picture::{PictureCleanupStatusView, UnusedPictureView};

/// The status of the one cleanup, held in memory, so a restart goes back to idle. A frame is
/// published whenever the state, the whole percentage or the count changes.
pub(in crate::picture) struct CleanupBoard {
    status: Mutex<PictureCleanupStatusView>,
    events: EventBus,
}

impl CleanupBoard {
    pub(in crate::picture) fn new(events: EventBus) -> Self {
        Self { status: Mutex::default(), events }
    }

    pub(in crate::picture) fn status(&self) -> PictureCleanupStatusView {
        self.locked().clone()
    }

    pub(in crate::picture) fn show(&self, next: PictureCleanupStatusView) {
        self.replace(&mut self.locked(), next);
    }

    /// Takes the board for a new search, which a run in progress or a list not yet answered refuses.
    pub(in crate::picture) fn begin_search(&self) -> Result<()> {
        let mut status = self.locked();
        match *status {
            PictureCleanupStatusView::Searching { .. } | PictureCleanupStatusView::Deleting { .. } => {
                Err(RekallError::conflict("A cleanup is already running."))
            }
            PictureCleanupStatusView::Ready { .. } => Err(RekallError::conflict("Delete or discard the unused pictures already listed first.")),
            _ => {
                self.replace(&mut status, PictureCleanupStatusView::Searching { percentage: 0 });
                Ok(())
            }
        }
    }

    /// Moves on from a ready list to `next` and hands the list over, so only one caller gets it.
    pub(in crate::picture) fn answer_list(&self, next: PictureCleanupStatusView) -> Result<Vec<UnusedPictureView>> {
        let mut status = self.locked();
        let PictureCleanupStatusView::Ready { pictures } = &*status else {
            return Err(RekallError::conflict("There is no list of unused pictures."));
        };
        let listed = pictures.clone();
        self.replace(&mut status, next);
        Ok(listed)
    }

    /// Published under the lock, so frames arrive in the order the status changed.
    fn replace(&self, status: &mut PictureCleanupStatusView, next: PictureCleanupStatusView) {
        let frame = frame_of(&next);
        if frame != frame_of(status) {
            self.events.publish(frame);
        }
        *status = next;
    }

    fn locked(&self) -> MutexGuard<'_, PictureCleanupStatusView> {
        self.status.lock().expect("nothing panics while holding the cleanup status")
    }
}

fn frame_of(status: &PictureCleanupStatusView) -> PictureCleanupStreamEvent {
    let (state, percentage, count) = match status {
        PictureCleanupStatusView::Idle => (PictureCleanupState::Idle, 0, 0),
        PictureCleanupStatusView::Searching { percentage } => (PictureCleanupState::Searching, *percentage, 0),
        PictureCleanupStatusView::Ready { pictures } => (PictureCleanupState::Ready, 100, pictures.len()),
        PictureCleanupStatusView::Deleting { percentage } => (PictureCleanupState::Deleting, *percentage, 0),
        PictureCleanupStatusView::Finished { removed, .. } => (PictureCleanupState::Finished, 100, *removed),
        PictureCleanupStatusView::Failed { .. } => (PictureCleanupState::Failed, 0, 0),
    };
    PictureCleanupStreamEvent { state, percentage, count }
}
