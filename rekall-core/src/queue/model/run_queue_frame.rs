use crate::StreamEvent;
use serde::Serialize;

use crate::queue::RunQueueView;

/// The run queue as the console receives it, under the `run-queue` frame.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct RunQueueFrame(pub RunQueueView);

impl StreamEvent for RunQueueFrame {
    const FRAME: &'static str = "run-queue";
}
