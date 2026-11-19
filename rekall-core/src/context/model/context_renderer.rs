//! Turns loaded context records into the markdown a session reads. The sections live in
//! `context/render/`, one file each; this composes them.

use std::collections::HashSet;

use crate::component::ComponentReferences;
use crate::context::ContextRecord;

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
        out.push_str(&self.render_sections(&record.sections));
        out.push_str(&self.render_steps(&record.steps, &record.step_connections, record.wrapup.as_ref(), &mut references));
        out.push_str(&self.render_commits(&record.commits));
        out.push_str(&self.render_wrapup(record.wrapup.as_ref(), &record.wrapup_freshness));
        out.push_str(&self.render_documents(&record.documents));
        out.push_str(&self.render_deviations(&record.deviations));
        out.push_str(&self.render_thread(&record.thread));
        for reference in &record.references {
            let nested = self.render_record(reference, heading_level + 1, rendered);
            if !nested.is_empty() {
                out.push('\n');
                out.push_str(&nested);
            }
        }
        out
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/context/model/context_renderer_deviation_tests.rs"]
mod deviation_tests;

#[cfg(test)]
#[path = "../../../tests/unit/context/model/context_renderer_thread_tests.rs"]
mod thread_tests;

#[cfg(test)]
#[path = "../../../tests/unit/context/model/context_renderer_flow_tests.rs"]
mod flow_tests;

#[cfg(test)]
#[path = "../../../tests/unit/context/model/context_renderer_skills_tests.rs"]
mod skills_tests;

#[cfg(test)]
#[path = "../../../tests/unit/context/model/context_renderer_connections_tests.rs"]
mod connections_tests;

#[cfg(test)]
#[path = "../../../tests/unit/context/model/context_renderer_sessions_tests.rs"]
mod sessions_tests;

#[cfg(test)]
#[path = "../../../tests/unit/context/model/context_renderer_sections_tests.rs"]
mod sections_tests;

#[cfg(test)]
#[path = "../../../tests/unit/context/model/context_renderer_freshness_tests.rs"]
mod freshness_tests;
