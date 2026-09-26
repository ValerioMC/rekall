use serde::Deserialize;


/// What a terminal is opened to do, which decides the `/rk` line typed into it first. `Work`
/// loads the task and works it; `Plan` has the session propose the task's checklist as drafts
/// and build nothing, so a plan needs no session already running on the task.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TerminalMode {
    #[default]
    Work,
    Plan,
}

impl TerminalMode {
    /// The line the session opens on, for a task anchored by `anchors`.
    pub fn first_line(self, anchors: &str) -> String {
        match self {
            TerminalMode::Work => format!("/rk {anchors}"),
            TerminalMode::Plan => format!("/rk {anchors} plan"),
        }
    }
}

impl std::fmt::Display for TerminalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TerminalMode::Work => "WORK",
            TerminalMode::Plan => "PLAN",
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/terminal/terminal_mode_tests.rs"]
mod tests;
