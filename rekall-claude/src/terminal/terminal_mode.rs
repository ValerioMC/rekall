use serde::Deserialize;

/// Longest request a generate line carries; the rest is cut so the line stays one prompt.
pub const REQUEST_MAX_CHARACTERS: usize = 2000;

/// What a terminal is opened to do, which decides the `/rk` line typed into it first. `Work`
/// loads the task and works it; `Plan` has the session propose the task's checklist as drafts
/// and build nothing; `Generate` has it describe the code as a Semantic Graph diagram answering
/// `request`. Only `Work` follows a step.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TerminalMode {
    #[default]
    Work,
    Plan,
    Generate { request: String },
}

/// The mode's name as the console sends it; a generate request travels beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TerminalModeName {
    Work,
    Plan,
    Generate,
}

impl TerminalMode {
    /// The line the session opens on, for a task anchored by `anchors`.
    pub fn first_line(&self, anchors: &str) -> String {
        match self {
            TerminalMode::Work => format!("/rk {anchors}"),
            TerminalMode::Plan => format!("/rk {anchors} plan"),
            TerminalMode::Generate { request } => format!("/rk {anchors} generate \"{}\"", one_line(request)),
        }
    }

    pub fn follows_steps(&self) -> bool {
        matches!(self, TerminalMode::Work)
    }
}

/// The request as one quoted prompt line: whitespace collapsed, so a newline cannot submit it
/// early, and double quotes turned single, so it cannot close its own quoting.
fn one_line(request: &str) -> String {
    let collapsed = request.split_whitespace().collect::<Vec<_>>().join(" ").replace('"', "'");
    collapsed.chars().take(REQUEST_MAX_CHARACTERS).collect()
}

impl std::fmt::Display for TerminalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TerminalMode::Work => "WORK",
            TerminalMode::Plan => "PLAN",
            TerminalMode::Generate { .. } => "GENERATE",
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/terminal/terminal_mode_tests.rs"]
mod tests;
