//! What every kind of `claude` session manager shares: the checklist marker a session moves while
//! it works a task, the bus that announces every end, and the loop that sweeps idle sessions.

mod ended_bus;
mod idle_sweep;
mod work_marker;

pub use ended_bus::EndedBus;
pub use idle_sweep::spawn_idle_sweep;
pub use work_marker::WorkMarker;
