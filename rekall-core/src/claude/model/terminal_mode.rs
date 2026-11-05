use rekall_diagram::DiagramKind;

/// Longest request a generate line carries; the rest is cut so the line stays one prompt.
pub const REQUEST_MAX_CHARACTERS: usize = 2000;

/// What a terminal is opened to do, which decides the `/rk` line typed into it first. `Work`
/// loads the task and works it; `Plan` has the session propose the task's checklist as drafts;
/// `PlanFlow` has it propose a flow, steps and routes, as drafts; `Replan` has it redesign the
/// task's draft and open steps, bending them to the `message` when one is given; `Revise` has it
/// evaluate the pending entries of the task's thread; `Generate` has it draw `request`
/// as Mermaid sheets of the `kinds` asked for; `Explain` has it write a walkthrough of `request`;
/// `Inventory` has it record the project's dependencies; `Assess` has it judge whether `advisory`
/// really reaches the project, `AssessAll` every advisory not judged yet; `Autoconfigure` has it
/// learn from the project's repository how to contribute and store that as the project's sections;
/// `Merge` has it bring the project's branch into the task's branch and resolve the conflicts.
/// Only `Work` follows a step.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TerminalMode {
    #[default]
    Work,
    Plan,
    PlanFlow,
    Replan { message: Option<String> },
    Revise,
    Generate { request: String, kinds: Vec<DiagramKind> },
    Explain { request: String },
    Inventory,
    Assess { advisory: String },
    AssessAll,
    Autoconfigure,
    Merge,
}

impl TerminalMode {
    /// The line the session opens on, for what `anchors` name.
    pub fn first_line(&self, anchors: &str) -> String {
        match self {
            TerminalMode::Work => format!("/rk {anchors}"),
            TerminalMode::Plan => format!("/rk {anchors} plan"),
            TerminalMode::PlanFlow => format!("/rk {anchors} plan flow"),
            TerminalMode::Replan { message: None } => format!("/rk {anchors} replan"),
            TerminalMode::Replan { message: Some(message) } => format!("/rk {anchors} replan \"{}\"", one_line(message)),
            TerminalMode::Revise => format!("/rk {anchors} revise"),
            TerminalMode::Generate { request, kinds } => {
                format!("/rk {anchors} generate{} \"{}\"", kinds_term(kinds), one_line(request))
            }
            TerminalMode::Explain { request } => format!("/rk {anchors} explain \"{}\"", one_line(request)),
            TerminalMode::Inventory => format!("/rk {anchors} inventory"),
            TerminalMode::Assess { advisory } => format!("/rk {anchors} assess advisory:{advisory}"),
            TerminalMode::AssessAll => format!("/rk {anchors} assess"),
            TerminalMode::Autoconfigure => format!("/rk {anchors} autoconfigure"),
            TerminalMode::Merge => format!("/rk {anchors} merge"),
        }
    }

    /// Whether the work is the project's as a whole: a diagram, its inventory, an assessment or its
    /// configuration, the only work a terminal on a project alone has.
    pub fn works_on_a_project(&self) -> bool {
        matches!(
            self,
            TerminalMode::Generate { .. } | TerminalMode::Explain { .. } | TerminalMode::Inventory | TerminalMode::Assess { .. } | TerminalMode::AssessAll | TerminalMode::Autoconfigure
        )
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
            TerminalMode::Replan { .. } => "REPLAN",
            TerminalMode::Revise => "REVISE",
            TerminalMode::Generate { .. } => "GENERATE",
            TerminalMode::Explain { .. } => "EXPLAIN",
            TerminalMode::Inventory => "INVENTORY",
            TerminalMode::Assess { .. } => "ASSESS",
            TerminalMode::AssessAll => "ASSESS_ALL",
            TerminalMode::Autoconfigure => "AUTOCONFIGURE",
            TerminalMode::Merge => "MERGE",
        })
    }
}

impl serde::Serialize for TerminalMode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/model/terminal_mode_tests.rs"]
mod tests;
