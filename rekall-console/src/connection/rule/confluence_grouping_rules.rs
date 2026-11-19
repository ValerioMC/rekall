//! What a Confluence space becomes in a project: its home page the blueprint, its description the
//! project's, and every other page a part of the section of the concept it is about. A page is
//! placed by its labels first and by the words of its title second; one that names no concept goes
//! to `General`.

use rekall_core::connection::{ConfluencePage, ConfluenceSpace};
use rekall_store::project_section::BODY_MAX_CHARACTERS;

const DESCRIPTION_MAX_CHARACTERS: usize = 500;
const BLUEPRINT_MAX_CHARACTERS: usize = 100_000;
const TRUNCATION_NOTE: &str = "\n\n_The rest of this page is cut: open it in Confluence._";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::connection) enum Concept {
    Architecture,
    Decisions,
    Api,
    Deployment,
    Operations,
    Conventions,
    Requirements,
    Glossary,
    General,
}

impl Concept {
    const IN_ORDER: [Concept; 9] =
        [Self::Architecture, Self::Decisions, Self::Api, Self::Deployment, Self::Operations, Self::Conventions, Self::Requirements, Self::Glossary, Self::General];

    fn title(self) -> &'static str {
        match self {
            Self::Architecture => "Architecture",
            Self::Decisions => "Decisions",
            Self::Api => "API",
            Self::Deployment => "Deployment",
            Self::Operations => "Operations",
            Self::Conventions => "Conventions",
            Self::Requirements => "Requirements",
            Self::Glossary => "Glossary",
            Self::General => "General",
        }
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Architecture => &["architecture", "design", "diagram", "component", "system", "domain"],
            Self::Decisions => &["decision", "adr", "rfc", "proposal"],
            Self::Api => &["api", "endpoint", "integration", "webhook", "schema", "contract"],
            Self::Deployment => &["deploy", "release", "pipeline", "ci", "cd", "infrastructure", "environment", "kubernetes", "docker", "hosting"],
            Self::Operations => &["runbook", "operation", "monitoring", "incident", "oncall", "alert", "troubleshooting", "support", "backup"],
            Self::Conventions => &["convention", "guideline", "standard", "style", "contributing", "workflow", "process", "practice", "onboarding"],
            Self::Requirements => &["requirement", "specification", "spec", "feature", "roadmap", "story", "product", "prd"],
            Self::Glossary => &["glossary", "term", "definition", "faq"],
            Self::General => &[],
        }
    }
}

/// A section to write: its title and body, and how many pages it gathers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::connection) struct SectionDraft {
    pub title: String,
    pub body: String,
    pub pages: usize,
}

/// What a space fills the project with. Each part is none when the space holds nothing for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::connection) struct ProjectContent {
    pub description: Option<String>,
    pub blueprint: Option<String>,
    pub sections: Vec<SectionDraft>,
}

pub(in crate::connection) fn project_content(space: &ConfluenceSpace, pages: &[ConfluencePage]) -> ProjectContent {
    let home = space.home_page_id.as_deref().and_then(|id| pages.iter().find(|page| page.id == id));
    let description = space.description.clone().or_else(|| home.and_then(|page| first_paragraph(&page.markdown))).map(|text| capped(&text, DESCRIPTION_MAX_CHARACTERS));
    let blueprint = home.filter(|page| !page.markdown.trim().is_empty()).map(|page| capped(&format!("{}\n\nSource: [{}]({})", page.markdown.trim(), page.title, page.url), BLUEPRINT_MAX_CHARACTERS));
    let others: Vec<&ConfluencePage> = pages.iter().filter(|page| Some(page.id.as_str()) != home.map(|home| home.id.as_str())).collect();
    ProjectContent { description, blueprint, sections: sections_of(&others) }
}

fn sections_of(pages: &[&ConfluencePage]) -> Vec<SectionDraft> {
    Concept::IN_ORDER
        .iter()
        .filter_map(|concept| {
            let gathered: Vec<&ConfluencePage> = pages.iter().copied().filter(|page| concept_of(page) == *concept).collect();
            (!gathered.is_empty()).then(|| SectionDraft { title: concept.title().to_string(), body: section_body(&gathered), pages: gathered.len() })
        })
        .collect()
}

fn concept_of(page: &ConfluencePage) -> Concept {
    let by_label = Concept::IN_ORDER.iter().copied().find(|concept| page.labels.iter().any(|label| mentions(concept, label)));
    let by_title = || Concept::IN_ORDER.iter().copied().find(|concept| mentions(concept, &page.title));
    by_label.or_else(by_title).unwrap_or(Concept::General)
}

fn mentions(concept: &Concept, text: &str) -> bool {
    let lowered = text.to_lowercase();
    lowered.split(|character: char| !character.is_alphanumeric()).filter(|word| !word.is_empty()).any(|word| concept.keywords().iter().any(|keyword| is_word_of(word, keyword)))
}

fn is_word_of(word: &str, keyword: &str) -> bool {
    word == keyword || (keyword.len() >= 5 && word.starts_with(keyword))
}

fn section_body(pages: &[&ConfluencePage]) -> String {
    let body = pages.iter().map(|page| page_part(page)).collect::<Vec<_>>().join("\n\n");
    capped(&body, BODY_MAX_CHARACTERS)
}

fn page_part(page: &ConfluencePage) -> String {
    let heading = format!("## {}", page.title.trim());
    let link = if page.url.is_empty() { String::new() } else { format!("\n\n[Open in Confluence]({})", page.url) };
    let body = lowered_headings(page.markdown.trim());
    if body.is_empty() { format!("{heading}{link}") } else { format!("{heading}{link}\n\n{body}") }
}

/// A page's own headings go one level under the title it is listed with.
fn lowered_headings(markdown: &str) -> String {
    let mut inside_fence = false;
    markdown
        .lines()
        .map(|line| {
            if line.starts_with("```") {
                inside_fence = !inside_fence;
            }
            let is_heading = !inside_fence && line.starts_with('#') && line.trim_start_matches('#').starts_with(' ');
            if is_heading && line.chars().take_while(|character| *character == '#').count() < 6 { format!("#{line}") } else { line.to_string() }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn first_paragraph(markdown: &str) -> Option<String> {
    markdown
        .split("\n\n")
        .map(str::trim)
        .find(|block| !block.is_empty() && !block.starts_with(['#', '|', '`', '-']))
        .map(|block| block.replace('\n', " "))
}

fn capped(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let room = limit.saturating_sub(TRUNCATION_NOTE.chars().count());
    format!("{}{TRUNCATION_NOTE}", text.chars().take(room).collect::<String>())
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/rule/confluence_grouping_rules_tests.rs"]
mod tests;
