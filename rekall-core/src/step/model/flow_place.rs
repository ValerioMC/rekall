use rekall_kernel::{RekallError, Result};

/// The farthest a step may be dragged from the origin, in canvas pixels either way.
const REACH: i32 = 20_000;

/// Where a step was dragged to on the flow canvas, in canvas pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlowPlace {
    pub x: i32,
    pub y: i32,
}

impl FlowPlace {
    /// A place within reach; a coordinate beyond it is a mistake in the caller, not a layout.
    pub fn new(x: i32, y: i32) -> Result<Self> {
        if x.abs() > REACH || y.abs() > REACH {
            return Err(RekallError::illegal(format!("A step cannot be placed more than {REACH} pixels from the origin.")));
        }
        Ok(Self { x, y })
    }
}
