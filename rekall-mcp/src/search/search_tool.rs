use async_trait::async_trait;
use rekall_core::search::{SearchHit, SearchHitKind, SearchService, TERM_MIN};
use rekall_kernel::jstr;
use serde_json::Value;

use crate::tool::{told, Anchor, AnchoredProject, Arguments};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

/// Lets a session look for earlier work before redoing it. It answers with an index of anchors
/// and excerpts; the text behind a result is read with `rekall_context`.
pub struct SearchTool {
    search: SearchService,
}

impl SearchTool {
    pub fn new(search: SearchService) -> Self {
        Self { search }
    }
}

#[async_trait]
impl McpTool for SearchTool {
    fn name(&self) -> &'static str {
        "rekall_search"
    }

    fn description(&self) -> &'static str {
        texts::SEARCH
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string("term", "The phrase to find, at least three characters, matched whole and ignoring case.")
            .optional_string("anchors", "`project:vega` to search that project only. Omitted, every project is searched. A label, never a title.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let term = args.required_string("term")?;
        let project = project_of(args.optional_string("anchors").as_deref())?;
        if is_too_short(&term) {
            return Ok(too_short_message());
        }
        let hits = match &project {
            Some(label) => self.search.search_in_project(Some(&term), label).await,
            None => self.search.search(Some(&term)).await,
        }
        .map_err(|e| told(e, ""))?;
        Ok(report(&term, project.as_deref(), &hits))
    }
}

fn project_of(anchors: Option<&str>) -> Result<Option<String>, ToolError> {
    let Some(raw) = anchors else {
        return Ok(None);
    };
    Ok(Some(AnchoredProject::from(&Anchor::parse_all(Some(raw))?)?.project_label))
}

fn is_too_short(term: &str) -> bool {
    jstr::len(&jstr::collapse_whitespace(jstr::strip(term))) < TERM_MIN
}

fn too_short_message() -> String {
    format!("Nothing was searched: the term needs at least {TERM_MIN} characters.")
}

fn report(term: &str, project_label: Option<&str>, hits: &[SearchHit]) -> String {
    let scope = project_label.map_or("every project".to_string(), |label| format!("`project:{label}`"));
    if hits.is_empty() {
        return format!("Nothing in {scope} matches \"{term}\".");
    }
    let lines: Vec<String> = hits.iter().map(line_of).collect();
    format!(
        "{} result{} for \"{term}\" in {scope}. Pass an anchor to `rekall_context` to read one:\n{}",
        hits.len(),
        if hits.len() == 1 { "" } else { "s" },
        lines.join("\n")
    )
}

fn line_of(hit: &SearchHit) -> String {
    format!("- {} \"{}\": `{}`\n  {}", kind_name(hit.kind), hit.title, hit.context_anchor, hit.excerpt)
}

fn kind_name(kind: SearchHitKind) -> &'static str {
    match kind {
        SearchHitKind::Description => "description",
        SearchHitKind::Step => "step",
        SearchHitKind::Wrapup => "wrapup",
        SearchHitKind::Note => "note",
    }
}

#[cfg(test)]
#[path = "../../tests/unit/search/search_tool_tests.rs"]
mod tests;
