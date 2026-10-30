//! What every kind of `claude` session manager shares: the checklist marker a session moves while
//! it works a task, the bus that announces every end, the loop that sweeps idle sessions, and what
//! starting one takes from the machine.

mod claude_options;
mod ended_bus;
mod idle_sweep;
mod session_hosts;
mod session_launcher;
mod work_marker;

pub use claude_options::ClaudeOptions;
pub use ended_bus::EndedBus;
pub use idle_sweep::spawn_idle_sweep;
pub use session_hosts::SessionHosts;
pub use session_launcher::{require_folder, SessionLauncher};
pub use work_marker::WorkMarker;
