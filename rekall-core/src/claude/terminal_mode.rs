use rekall_diagram::DiagramKind;

/// Longest request a generate line carries; the rest is cut so the line stays one prompt.
pub const REQUEST_MAX_CHARACTERS: usize = 2000;

/// What a terminal is opened to do, which decides the `/rk` line typed into it first. `Work`
/// loads the task and works it; `Plan` has the session propose the task's checklist as drafts;
/// `PlanFlow` has it propose a flow, steps and routes, as drafts; `Generate` has it draw `request`
/// as Mermaid sheets of the `kinds` asked for; `Explain` has it write a walkthrough of `request`;
/// `Inventory` has it record the project's dependencies; `Assess` has it judge whether `advisory`
/// really reaches the project, `AssessAll` every advisory not judged yet. Only `Work` follows a step.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TerminalMode {
    #[default]
    Work,
    Plan,
    PlanFlow,
    Generate { request: String, kinds: Vec<DiagramKind> },
    Explain { request: String },
    Inventory,
    Assess { advisory: String },
    AssessAll,
}

impl TerminalMode {
    /// The line the session opens on, for what `anchors` name.
    pub fn first_line(&self, anchors: &str) -> String {
        match self {
            TerminalMode::Work => format!("/rk {anchors}"),
            TerminalMode::Plan => format!("/rk {anchors} plan"),
            TerminalMode::PlanFlow => format!("/rk {anchors} plan flow"),
            TerminalMode::Generate { request, kinds } => {
                format!("/rk {anchors} generate{} \"{}\"", kinds_term(kinds), one_line(request))
            }
            TerminalMode::Explain { request } => format!("/rk {anchors} explain \"{}\"", one_line(request)),
            TerminalMode::Inventory => format!("/rk {anchors} inventory"),
            TerminalMode::Assess { advisory } => format!("/rk {anchors} assess advisory:{advisory}"),
            TerminalMode::AssessAll => format!("/rk {anchors} assess"),
        }
    }

    /// Whether the work is the project's as a whole: a diagram, its inventory or an assessment, the
    /// only work a terminal on a project alone has.
    pub fn works_on_a_project(&self) -> bool {
        matches!(self, TerminalMode::Generate { .. } | TerminalMode::Explain { .. } | TerminalMode::Inventory | TerminalMode::Assess { .. } | TerminalMode::AssessAll)
    }

    pub fn follows_steps(&self) -> bool {
        matches!(self, TerminalMode::Work)
    }
}

/// ` kinds:sequence,state`, or nothing when the session is left to choose.
fn kinds_term(kinds: &[DiagramKind]) -> String {
    if kinds.is_empty() {
        return String::new();
    }
    let names: Vec<&str> = kinds.iter().map(|kind| kind.name()).collect();
    format!(" kinds:{}", names.join(","))
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
            TerminalMode::PlanFlow => "PLAN_FLOW",
            TerminalMode::Generate { .. } => "GENERATE",
            TerminalMode::Explain { .. } => "EXPLAIN",
            TerminalMode::Inventory => "INVENTORY",
            TerminalMode::Assess { .. } => "ASSESS",
            TerminalMode::AssessAll => "ASSESS_ALL",
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/claude/terminal_mode_tests.rs"]
mod tests;
