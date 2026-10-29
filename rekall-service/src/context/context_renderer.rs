//! Turns loaded context records into the markdown a session reads.

use std::collections::HashSet;

use rekall_common::jstr;
use rekall_store::{ConnectionKind, DocumentContextMode, TaskStepState};

use super::{ContextCommitView, ContextRecord, ContextSkillScope, ContextSkillView, DocumentView, StepConnectionView};
use crate::component::{ComponentReferences, ProjectComponentView};
use crate::connection::ConnectionView;
use crate::step::{StepLinkView, TaskStepView};
use crate::wrapup::WrapupView;

pub const MAX_DOCUMENT_CHARACTERS: usize = 20_000;

/// A diff is the one thing here worth more room than a note: it is what the session was handed to read.
pub const MAX_DIFF_CHARACTERS: usize = 60_000;

/// How much of a reference note's opening line travels with it.
pub const REFERENCE_LINE_MAX: usize = 200;

#[derive(Clone, Copy, Debug, Default)]
pub struct ContextRenderer;

impl ContextRenderer {
    /// The whole answer to one `rekall_context` call: every record once, in the order loaded.
    pub fn render(&self, records: &[ContextRecord]) -> String {
        let mut out = String::from("# Context\n");
        let mut rendered = HashSet::new();
        for record in records {
            out.push('\n');
            out.push_str(&self.render_record(record, 2, &mut rendered));
        }
        out
    }

    pub fn render_record(&self, record: &ContextRecord, heading_level: usize, rendered: &mut HashSet<String>) -> String {
        if !rendered.insert(record.anchor.clone()) {
            return String::new();
        }
        let mut out = format!(
            "{} {}: {}\n\n- `anchor`: `{}`\n",
            "#".repeat(heading_level),
            record.kind,
            record.label,
            record.anchor
        );
        for (key, value) in &record.fields {
            out.push_str(&format!("- `{key}`: {value}\n"));
        }
        if !record.related.is_empty() {
            out.push_str("- `related`:\n");
            for anchor in &record.related {
                out.push_str(&format!("  - `{anchor}`\n"));
            }
        }
        let mut references = ComponentReferences::new(record.component_lookup.as_deref())
            .with_connections(&record.connection_lookup)
            .with_tasks(&record.task_lookup);
        out.push_str(&self.render_description(record.description.as_deref(), &mut references));
        out.push_str(&self.render_skills(&record.skills));
        out.push_str(&self.render_components(&record.components));
        out.push_str(&self.render_connections(&record.connections));
        out.push_str(&self.render_blueprint(record.blueprint.as_deref()));
        out.push_str(&self.render_steps(&record.steps, &record.step_connections, record.wrapup.as_ref(), &mut references));
        out.push_str(&self.render_commits(&record.commits));
        out.push_str(&self.render_wrapup(record.wrapup.as_ref()));
        out.push_str(&self.render_documents(&record.documents));
        for reference in &record.references {
            let nested = self.render_record(reference, heading_level + 1, rendered);
            if !nested.is_empty() {
                out.push('\n');
                out.push_str(&nested);
            }
        }
        out
    }

    pub fn render_steps(
        &self,
        all_steps: &[TaskStepView],
        connections: &[StepConnectionView],
        wrapup: Option<&WrapupView>,
        references: &mut ComponentReferences,
    ) -> String {
        let drafts = all_steps.iter().filter(|s| s.state.draft()).count();
        let steps: Vec<&TaskStepView> = all_steps.iter().filter(|s| !s.state.draft()).collect();
        if steps.is_empty() {
            return String::new();
        }
        let mut out = format!("\n<steps {}>\n", steps_attributes(&steps, drafts, wrapup));
        for step in &steps {
            let connection = connections.iter().find(|connection| connection.step_id == step.id);
            out.push_str(&render_step(step, connection, wrapup, references));
            out.push_str(&render_arrival(step, &steps));
            out.push_str(&render_routes(step, &steps));
        }
        if drafts > 0 {
            out.push_str(&format!(
                "<!-- {drafts} further step{} still in draft, not shown: promoted to the checklist \
                 in the console when it is ready to work -->\n",
                if drafts == 1 { "" } else { "s" }
            ));
        }
        out.push_str("</steps>\n");
        out
    }

    /// The skills a session invokes before it works, each with the scope that decides when.
    pub fn render_skills(&self, skills: &[ContextSkillView]) -> String {
        if skills.is_empty() {
            return String::new();
        }
        let mut out = format!(
            "\n<skills count=\"{}\">\nInvoke each skill below with the Skill tool before you start working, and follow \
             it for the whole task. One scoped to a step applies only while you work that step.\n",
            skills.len()
        );
        for skill in skills {
            let scope = match &skill.scope {
                ContextSkillScope::Task => "task".to_string(),
                ContextSkillScope::Tag(name) => format!("tag: {}", attribute(name)),
                ContextSkillScope::Step(title) => format!("step: {}", attribute(title)),
            };
            out.push_str(&format!("- `{}` ({scope})\n", skill.name));
        }
        out.push_str("</skills>\n");
        out
    }

    pub fn render_wrapup(&self, wrapup: Option<&WrapupView>) -> String {
        match wrapup {
            None => String::new(),
            Some(wrapup) => format!(
                "\n<wrapup written-by=\"{}\" updated=\"{}\">\n{}\n</wrapup>\n",
                wrapup.written_by,
                wrapup.updated_at,
                truncate(Some(&wrapup.body_markdown), MAX_DOCUMENT_CHARACTERS)
            ),
        }
    }

    pub fn render_commits(&self, commits: &[ContextCommitView]) -> String {
        if commits.is_empty() {
            return String::new();
        }
        let mut out = format!("\n<commits count=\"{}\">\n", commits.len());
        for commit in commits {
            out.push_str(&format!("<commit hash=\"{}\"", commit.commit_hash));
            if let Some(step) = &commit.step_title {
                out.push_str(&format!(" step=\"{}\"", step.replace('"', "'")));
            }
            out.push_str(">\n");
            out.push_str(&commit.comment);
            out.push('\n');
            match commit.diff.as_deref() {
                Some(diff) if !jstr::is_blank(diff) => {
                    out.push('\n');
                    out.push_str(&truncate(Some(diff), MAX_DIFF_CHARACTERS));
                    out.push('\n');
                }
                _ => out.push_str("\n(no diff was recorded for this commit; read it from the repository by its hash)\n"),
            }
            out.push_str("</commit>\n");
        }
        out.push_str("</commits>\n");
        out
    }

    pub fn render_description(&self, description: Option<&str>, references: &mut ComponentReferences) -> String {
        match description {
            Some(text) if !jstr::is_blank(text) => {
                let expanded = references.expand(text);
                format!("\n<description>\n{}\n</description>\n", truncate(Some(&expanded), MAX_DOCUMENT_CHARACTERS))
            }
            _ => String::new(),
        }
    }

    /// What the project reaches outside Rekall and how a session uses it. The credentials stay with
    /// Rekall: a session only ever names a connection by its label.
    pub fn render_connections(&self, connections: &[ConnectionView]) -> String {
        if connections.is_empty() {
            return String::new();
        }
        let mut out = format!(
            "\n<connections count=\"{}\">\nThese work only in a terminal Rekall opened on this project. Never print, echo or \
             write a credential or a kubeconfig, never call a credential helper or Rekall's credentials endpoint yourself, \
             and reach each system only the way its line says.\n",
            connections.len()
        );
        for connection in connections {
            out.push_str(&format!("- `{}` ({}): {}\n", connection.label, attribute(&connection.title), connection_usage(connection)));
        }
        out.push_str("</connections>\n");
        out
    }

    /// The project's parts: each folder a session can open to find that part, with what it is.
    pub fn render_components(&self, components: &[ProjectComponentView]) -> String {
        if components.is_empty() {
            return String::new();
        }
        let mut out = format!("\n<components count=\"{}\">\n", components.len());
        for component in components {
            out.push_str(&format!(
                "<component name=\"{}\" folder=\"{}\">\n",
                attribute(&component.name),
                attribute(&component.folder)
            ));
            if !jstr::is_blank(&component.description) {
                out.push_str(&truncate(Some(&component.description), MAX_DOCUMENT_CHARACTERS));
                out.push('\n');
            }
            out.push_str("</component>\n");
        }
        out.push_str("</components>\n");
        out
    }

    pub fn render_blueprint(&self, blueprint: Option<&str>) -> String {
        match blueprint {
            Some(text) if !jstr::is_blank(text) => {
                format!("\n<blueprint>\n{}\n</blueprint>\n", truncate(Some(text), MAX_DOCUMENT_CHARACTERS))
            }
            _ => String::new(),
        }
    }

    pub fn render_documents(&self, documents: &[DocumentView]) -> String {
        let mut out = String::new();
        for document in documents {
            if document.context_mode == DocumentContextMode::Reference {
                out.push_str(&self.render_reference(document));
            } else {
                out.push_str(&format!(
                    "\n<document title=\"{}\" kind=\"{}\">\n{}\n</document>\n",
                    document.title,
                    document.kind,
                    truncate(Some(&document.body_markdown), MAX_DOCUMENT_CHARACTERS)
                ));
            }
        }
        out
    }

    /// A reference note: its title, its opening line so the session can judge whether it
    /// applies, and the anchor that loads the rest.
    pub fn render_reference(&self, document: &DocumentView) -> String {
        format!(
            "\n<document title=\"{}\" kind=\"{}\" anchor=\"{}\" loaded=\"on request\">\n{}\n\
             Not included. If the work needs it, load it with `rekall_context` and the anchor `{}`.\n\
             </document>\n",
            document.title,
            document.kind,
            document.anchor,
            opening_line(&document.body_markdown),
            document.anchor
        )
    }
}

/// A step sent back after review: what each earlier pass worked from, then the feedback this
/// pass acts on, so a session builds on its work instead of reading the feedback as a new brief.
fn render_passes(step: &TaskStepView, references: &mut ComponentReferences) -> String {
    let mut out = String::from("Sent back after review. Previously you have executed this:\n");
    for (at, pass) in step.passes.iter().enumerate() {
        out.push_str(&format!("\n<previous-pass number=\"{}\" sent-back=\"{}\">\n", at + 1, pass.sent_back_at));
        match pass.detail_markdown.as_deref().filter(|body| !jstr::is_blank(body)) {
            Some(body) => out.push_str(&truncate(Some(&references.expand(body)), MAX_DOCUMENT_CHARACTERS)),
            None => out.push_str("(no detail beyond the title)"),
        }
        out.push_str("\n</previous-pass>\n");
    }
    out.push_str("\nBut the user's feedback is this, and it is what this pass has to do:\n\n<feedback>\n");
    match step.body_markdown.as_deref().filter(|body| !jstr::is_blank(body)) {
        Some(body) => out.push_str(&truncate(Some(&references.expand(body)), MAX_DOCUMENT_CHARACTERS)),
        None => out.push_str("(not written yet: ask the user what the pass should change)"),
    }
    out.push_str("\n</feedback>");
    out
}

fn is_unwritten(step: &TaskStepView, wrapup: Option<&WrapupView>) -> bool {
    if !step.state.complete() {
        return false;
    }
    let Some(completed) = step.completed_at() else {
        return false;
    };
    match wrapup {
        None => true,
        Some(wrapup) => completed.is_after(&wrapup.updated_at),
    }
}

/// How a session reaches one connection, or why it cannot.
fn connection_usage(connection: &ConnectionView) -> String {
    if !connection.credential_set {
        return "no credential is stored for it. Tell the person instead of looking for another way in".into();
    }
    match connection.kind {
        ConnectionKind::Kubernetes => {
            let namespace = connection.target.as_deref().map_or_else(|| " in its default namespace".to_string(), |namespace| format!(", namespace `{namespace}`"));
            format!("Kubernetes, `kubectl --context {}`{namespace}", connection.label)
        }
        ConnectionKind::Gitlab => {
            let instance = connection.instance_url.as_deref().unwrap_or("its GitLab instance");
            let project = connection.target.as_deref().map_or_else(String::new, |path| format!(", project `{path}` (`:project` in an API path)"));
            format!("GitLab at {instance}{project}. `git push` to it over HTTPS uses the configured credential helper; call its API with `rekall-gitlab --connection {} <METHOD> <path>`", connection.label)
        }
        ConnectionKind::Jira => {
            let site = connection.instance_url.as_deref().unwrap_or("its Jira site");
            let project = connection.target.as_deref().map_or_else(String::new, |key| format!(", project `{key}` (`:project` in an API path)"));
            format!(
                "Jira at {site}{project}. Read and create its issues with `rekall-jira --connection {} <METHOD> <path> [curl options]`, \
                 a path under `/rest/api/2` such as `POST /issue` with the JSON body as `-d`; search is `GET /search/jql?jql=…` on Jira Cloud \
                 (`*.atlassian.net`) and `GET /search?jql=…` on Data Center",
                connection.label
            )
        }
    }
}

fn attribute(value: &str) -> String {
    value.replace('"', "'")
}

fn indent(body: &str) -> String {
    jstr::lines(body)
        .into_iter()
        .map(|line| if jstr::is_blank(line) { String::new() } else { format!("  {line}") })
        .collect::<Vec<_>>()
        .join("\n")
}

fn opening_line(body: &str) -> String {
    let first = jstr::lines(body)
        .into_iter()
        .map(jstr::strip)
        .find(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("```"))
        .unwrap_or("");
    if jstr::len(first) <= REFERENCE_LINE_MAX {
        first.to_string()
    } else {
        format!("{}…", jstr::strip_trailing(jstr::prefix(first, REFERENCE_LINE_MAX)))
    }
}

fn truncate(body: Option<&str>, limit: usize) -> String {
    let Some(body) = body else {
        return String::new();
    };
    let length = jstr::len(body);
    if length <= limit {
        return body.to_string();
    }
    format!("{}\n\n[truncated: {limit} of {length} characters shown]", jstr::prefix(body, limit))
}

/// The checklist's counts; one that is zero is left out, except done and open.
fn steps_attributes(steps: &[&TaskStepView], drafts: usize, wrapup: Option<&WrapupView>) -> String {
    let done = steps.iter().filter(|s| s.state.complete()).count();
    let running = steps.iter().filter(|s| s.state.running()).count();
    let awaiting = steps.iter().filter(|s| s.state == TaskStepState::Claimed).count();
    let unwritten = steps.iter().filter(|s| is_unwritten(s, wrapup)).count();
    let mut attributes = format!("done=\"{done}\" open=\"{}\"", steps.len() - done);
    if is_flow(steps) {
        attributes.push_str(&flow_attributes(steps));
    }
    let optional = [("running", running), ("awaiting-review", awaiting), ("finished-since-wrapup", unwritten), ("draft", drafts)];
    for (name, count) in optional {
        if count > 0 {
            attributes.push_str(&format!(" {name}=\"{count}\""));
        }
    }
    attributes
}

/// A closed step is its title alone, unless the wrapup predates it: then, like an open one, it carries its detail.
fn render_step(
    step: &TaskStepView,
    connection: Option<&StepConnectionView>,
    wrapup: Option<&WrapupView>,
    references: &mut ComponentReferences,
) -> String {
    let unwritten = is_unwritten(step, wrapup);
    let mut out = String::from(if step.state.complete() { "- [x] " } else { "- [ ] " });
    out.push_str(&step.title);
    if step.is_current() {
        out.push_str(&format!("  (current, {})", step.visit_label()));
    } else if let Some(max) = step.max_visits {
        out.push_str(&format!("  (entered {} of at most {max} times)", step.visits.len()));
    }
    if step.state.running() {
        out.push_str("  (in progress)");
    } else if step.state == TaskStepState::Claimed {
        out.push_str("  (claimed, waiting for the console to accept it)");
    }
    if unwritten {
        out.push_str("  (finished since the wrapup was written)");
    }
    out.push('\n');
    if step.state.complete() && !unwritten {
        return out;
    }
    if let Some(connection) = connection {
        out.push_str(&render_step_connection(connection));
    }
    if !step.passes.is_empty() {
        out.push_str(&indent(&render_passes(step, references)));
        out.push('\n');
    } else if let Some(body) = step.body_markdown.as_deref().filter(|body| !jstr::is_blank(body)) {
        out.push_str(&indent(&truncate(Some(&references.expand(body)), MAX_DOCUMENT_CHARACTERS)));
        out.push('\n');
    }
    out
}

/// On the current step, what the step the flow came from reported as it left, so a pass sent here
/// to fix something works from what was seen rather than from a session's memory.
fn render_arrival(step: &TaskStepView, steps: &[&TaskStepView]) -> String {
    let Some(open) = step.visits.iter().find(|visit| visit.is_open()) else {
        return String::new();
    };
    let Some((from, left)) = steps
        .iter()
        .find_map(|candidate| candidate.visits.iter().find(|visit| visit.ordinal == open.ordinal - 1).map(|visit| (candidate, visit)))
    else {
        return String::new();
    };
    let Some(report) = left.report.as_deref() else {
        return String::new();
    };
    let outcome = left.outcome.as_deref().unwrap_or("its only route");
    let arrival = format!(
        "The flow came here from step {number} \"{}\", which ended `{outcome}` and reported this. It is what this pass works from:\n\n\
         <arrived-from step=\"{number}\" outcome=\"{outcome}\">\n{}\n</arrived-from>",
        from.title,
        truncate(Some(report), MAX_DOCUMENT_CHARACTERS),
        number = from.position + 1
    );
    format!("{}\n", indent(&arrival))
}

/// The connection a step works through. One the project no longer binds has no line in
/// `<connections>`, so the session is told to report it rather than reach it some other way.
fn render_step_connection(connection: &StepConnectionView) -> String {
    if !connection.bound_as.is_empty() {
        let labels = connection.bound_as.iter().map(|label| format!("`{label}`")).collect::<Vec<_>>().join(" or ");
        return format!("  Connection: {labels}, reached as its line in `<connections>` says.\n");
    }
    format!(
        "  Connection: `{}`, which this project no longer uses, so `<connections>` does not list it. Report it; \
         do not reach the system another way.\n",
        connection.label
    )
}

fn is_flow(steps: &[&TaskStepView]) -> bool {
    steps.iter().any(|step| !step.links.is_empty())
}

/// `flow`, the step the work stands on (one-based, as `rekall_step` takes it) and how many moves it took.
fn flow_attributes(steps: &[&TaskStepView]) -> String {
    let moves: usize = steps.iter().map(|step| step.visits.len()).sum();
    let mut attributes = String::from(" flow=\"true\"");
    if let Some(current) = steps.iter().find(|step| step.is_current()) {
        attributes.push_str(&format!(" current=\"{}\"", current.position + 1));
    }
    attributes.push_str(&format!(" moves=\"{moves}\""));
    attributes
}

/// A step's routes, each naming its target by number and title. Finished steps get them too: a
/// route can bring the work back.
fn render_routes(step: &TaskStepView, steps: &[&TaskStepView]) -> String {
    if step.links.is_empty() {
        return String::new();
    }
    let routes: Vec<String> = step.links.iter().filter_map(|link| render_route(link, steps)).collect();
    if routes.is_empty() {
        return String::new();
    }
    format!("  Routes: {}\n", routes.join("; "))
}

/// One route: `outcome` -> a step by number and title, or the end of the flow. A route to a step
/// that is gone is left out.
fn render_route(link: &StepLinkView, steps: &[&TaskStepView]) -> Option<String> {
    let Some(to_step_id) = link.to_step_id else {
        return Some(format!("`{}` -> end of the flow", link.outcome));
    };
    let target = steps.iter().find(|candidate| candidate.id == to_step_id)?;
    Some(format!("`{}` -> step {} \"{}\"", link.outcome, target.position + 1, attribute(&target.title)))
}

#[cfg(test)]
#[path = "../../tests/unit/context/context_renderer_flow_tests.rs"]
mod flow_tests;

#[cfg(test)]
#[path = "../../tests/unit/context/context_renderer_skills_tests.rs"]
mod skills_tests;

#[cfg(test)]
#[path = "../../tests/unit/context/context_renderer_connections_tests.rs"]
mod connections_tests;
