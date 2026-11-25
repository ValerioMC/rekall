use rekall_export::ExportPart;
use rekall_store::{project_component, project_section};

const BLUEPRINT_HEADING: &str = "Blueprint";

/// The items one export holds, already in document order: the blueprint (when it has text), the
/// pages in the project's order and the components by position.
#[derive(Clone, Debug)]
pub struct ExportSelection {
    pub blueprint_markdown: Option<String>,
    pub pages: Vec<project_section::Model>,
    pub components: Vec<project_component::Model>,
}

impl ExportSelection {
    pub fn is_empty(&self) -> bool {
        self.blueprint_markdown.is_none() && self.pages.is_empty() && self.components.is_empty()
    }

    pub fn parts(&self) -> Vec<ExportPart> {
        let blueprint = self.blueprint_markdown.iter().map(|markdown| ExportPart { heading: BLUEPRINT_HEADING.to_string(), markdown: markdown.clone() });
        let pages = self.pages.iter().map(|page| ExportPart { heading: page.title.clone(), markdown: page.body_markdown.clone() });
        let components = self.components.iter().map(component_part);
        blueprint.chain(pages).chain(components).collect()
    }

    /// The title of the only item, or `None` when the export holds several.
    pub fn only_title(&self) -> Option<String> {
        match (&self.blueprint_markdown, self.pages.as_slice(), self.components.as_slice()) {
            (Some(_), [], []) => Some(BLUEPRINT_HEADING.to_string()),
            (None, [page], []) => Some(page.title.clone()),
            (None, [], [component]) => Some(component.name.clone()),
            _ => None,
        }
    }

    pub fn summary(&self) -> String {
        self.only_title().unwrap_or_else(|| self.counts())
    }

    fn counts(&self) -> String {
        let blueprint = self.blueprint_markdown.iter().map(|_| BLUEPRINT_HEADING.to_string());
        let pages = counted(self.pages.len(), "page");
        let components = counted(self.components.len(), "component");
        blueprint.chain(pages).chain(components).collect::<Vec<_>>().join(", ")
    }
}

fn component_part(component: &project_component::Model) -> ExportPart {
    let summary = if component.summary.trim().is_empty() { String::new() } else { format!(" — {}", component.summary.trim()) };
    ExportPart { heading: component.name.clone(), markdown: format!("`{}`{summary}\n\n{}", component.folder, component.description) }
}

fn counted(count: usize, noun: &str) -> Option<String> {
    match count {
        0 => None,
        1 => Some(format!("1 {noun}")),
        _ => Some(format!("{count} {noun}s")),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/export/rule/export_selection_tests.rs"]
mod tests;
