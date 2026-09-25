//! `GET /api/export`: the whole database as a zip of folders, `company/project/task/note.md`. A
//! note on several tasks is written under each, and `MANIFEST.md` says which files are copies of
//! one note, so the tree stays readable without Rekall. A task's wrapup is `WRAPUP.md`, its
//! checklist `STEPS.md`. Folder names are labels, sanitised so none can become a path.

use std::collections::{BTreeMap, HashSet};
use std::io::{Cursor, Write};
use std::sync::LazyLock;

use regex::Regex;
use rekall_common::{jstr, RekallError, Result};
use rekall_model::{company, project, task, task_step, WrapupAuthor};
use rekall_repository::repository as repo;
use rekall_service::{in_read, Services, Tx};
use tracing::info;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const SUMMARY_LENGTH: usize = 180;

#[derive(Clone)]
pub struct ExportService {
    services: Services,
}

struct TaskEntry {
    task: task::Model,
    wrapup: Option<rekall_model::wrapup::Model>,
    steps: Vec<task_step::Model>,
    documents: Vec<(rekall_model::document::Model, usize)>,
}

struct ProjectEntry {
    project: project::Model,
    tasks: Vec<TaskEntry>,
}

struct CompanyEntry {
    company: company::Model,
    projects: Vec<ProjectEntry>,
}

impl ExportService {
    pub fn new(services: Services) -> Self {
        Self { services }
    }

    pub async fn archive(&self) -> Result<Vec<u8>> {
        in_read!(&self.services.ctx, |tx| {
            let all = load(&tx).await?;
            let bytes = build(&all).map_err(|e| RekallError::internal("UncheckedIOException", format!("Could not build the export archive: {e}")))?;
            info!("Exported {} company/companies as a {} byte archive", all.len(), bytes.len());
            Ok::<_, RekallError>(bytes)
        })
    }

    pub fn file_name_for_today(&self) -> String {
        format!("rekall-{}.zip", today())
    }
}

fn today() -> String {
    chrono::Local::now().date_naive().format("%Y-%m-%d").to_string()
}

async fn load(tx: &Tx) -> Result<Vec<CompanyEntry>> {
    let db = tx.db();
    let mut out = Vec::new();
    for company in repo::company::find_all_by_order_by_name_asc(db).await? {
        let mut projects = Vec::new();
        for project in repo::project::find_by_company_id_order_by_label_asc(db, company.id).await? {
            let mut tasks = Vec::new();
            for task in repo::task::find_by_project_id_order_by_label_asc(db, project.id).await? {
                let mut documents = Vec::new();
                for document in repo::document::documents_of_task(db, task.id).await? {
                    let shared_by = repo::document::links_of_document(db, document.id).await?.len();
                    documents.push((document, shared_by));
                }
                tasks.push(TaskEntry {
                    wrapup: repo::wrapup::find_by_task_id(db, task.id).await?,
                    steps: repo::task_step::find_by_task_id_order_by_position_asc(db, task.id).await?,
                    documents,
                    task,
                });
            }
            projects.push(ProjectEntry { project, tasks });
        }
        out.push(CompanyEntry { company, projects });
    }
    Ok(out)
}

fn build(all: &[CompanyEntry]) -> zip::result::ZipResult<Vec<u8>> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut shared: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for company in all {
        let company_dir = safe(&company.company.name);
        zip.add_directory(format!("{company_dir}/"), options)?;
        for project in &company.projects {
            let project_dir = format!("{company_dir}/{}", safe(&project.project.label));
            zip.add_directory(format!("{project_dir}/"), options)?;
            for entry in &project.tasks {
                let task_dir = format!("{project_dir}/{}", safe(&entry.task.label));
                zip.add_directory(format!("{task_dir}/"), options)?;
                let mut used = HashSet::new();
                if let Some(wrapup) = &entry.wrapup {
                    used.insert("WRAPUP.md".to_string());
                    write(&mut zip, &format!("{task_dir}/WRAPUP.md"), &wrapup.body_markdown, options)?;
                }
                if !entry.steps.is_empty() {
                    used.insert("STEPS.md".to_string());
                    write(&mut zip, &format!("{task_dir}/STEPS.md"), &checklist(&entry.steps), options)?;
                }
                for (document, shared_by) in &entry.documents {
                    let path = format!("{task_dir}/{}", unique(&mut used, file_name(&document.title)));
                    write(&mut zip, &path, &document.body_markdown, options)?;
                    if *shared_by > 1 {
                        shared.entry(document.title.clone()).or_default().push(path);
                    }
                }
            }
        }
    }
    write(&mut zip, "MANIFEST.md", &manifest(all, &shared), options)?;
    Ok(zip.finish()?.into_inner())
}

fn write(zip: &mut ZipWriter<Cursor<Vec<u8>>>, path: &str, body: &str, options: SimpleFileOptions) -> zip::result::ZipResult<()> {
    zip.start_file(path, options)?;
    zip.write_all(body.as_bytes())?;
    Ok(())
}

fn manifest(all: &[CompanyEntry], shared: &BTreeMap<String, Vec<String>>) -> String {
    let mut out = format!(
        "# Rekall export\n\n{}\n\nOne folder per company, then per project, then per task, one file per note.\n",
        today()
    );
    for company in all {
        let c = &company.company;
        out.push_str(&format!("\n## {}  `company:{}`\n", c.name, c.name));
        if let Some(description) = c.description.as_deref().filter(|d| !jstr::is_blank(d)) {
            out.push_str(&format!("\n{}\n", summary(description)));
        }
        for project in &company.projects {
            let p = &project.project;
            out.push_str(&format!("\n### {}  `project:{}`\n\n- status: {}\n", p.title, p.label, p.status));
            if let Some(description) = p.description.as_deref().filter(|d| !jstr::is_blank(d)) {
                out.push_str(&format!("- {}\n", summary(description)));
            }
            for entry in &project.tasks {
                let t = &entry.task;
                out.push_str(&format!(
                    "\n#### {}  `project:{} task:{}`\n\n- status: {}\n",
                    t.title, p.label, t.label, t.status
                ));
                if let Some(description) = t.description.as_deref().filter(|d| !jstr::is_blank(d)) {
                    out.push_str(&format!("- {}\n", summary(description)));
                }
                if let Some(wrapup) = &entry.wrapup {
                    out.push_str(&format!(
                        "- `WRAPUP.md`: the state of the implementation, written {}\n",
                        if wrapup.written_by == WrapupAuthor::Claude { "by Claude" } else { "by hand" }
                    ));
                }
                if !entry.steps.is_empty() {
                    let done = entry.steps.iter().filter(|s| s.is_done()).count();
                    let checklist = entry.steps.iter().filter(|s| !s.state.draft()).count();
                    out.push_str(&format!("- `STEPS.md`: {done} of {checklist} steps done\n"));
                }
                if entry.documents.is_empty() {
                    out.push_str("- no notes\n");
                }
            }
        }
    }
    if !shared.is_empty() {
        out.push_str(
            "\n---\n\n## Notes that appear more than once\n\n\
             A note can be attached to several tasks. The tree has no way to say that, so it is written under each of \
             them. These files are copies of one note: editing them apart is how the folder tree used to drift, and \
             why this application exists.\n\n",
        );
        for (title, paths) in shared {
            out.push_str(&format!("- **{title}**\n"));
            for path in paths {
                out.push_str(&format!("  - `{path}`\n"));
            }
        }
    }
    out
}

fn checklist(steps: &[task_step::Model]) -> String {
    let mut out = String::from("# Steps\n");
    for step in steps {
        out.push('\n');
        out.push_str(if step.is_done() { "- [x] " } else { "- [ ] " });
        out.push_str(&step.title);
        if step.state.draft() {
            out.push_str("  (draft)");
        }
        out.push('\n');
        if let Some(body) = step.body_markdown.as_deref().filter(|b| !jstr::is_blank(b)) {
            out.push('\n');
            out.push_str(body);
            out.push('\n');
        }
    }
    out
}

fn summary(description: &str) -> String {
    let flat_owned = jstr::collapse_whitespace(&description.replace('\n', " "));
    let flat = jstr::trim(&flat_owned);
    if jstr::len(flat) <= SUMMARY_LENGTH {
        flat.to_string()
    } else {
        format!("{}\u{2026}", jstr::trim(jstr::prefix(flat, SUMMARY_LENGTH)))
    }
}

/// A name made into one folder or file name: anything outside `A-Za-z0-9 ._-` becomes `-`, a run
/// of dots becomes `-` (so `..` never survives), whitespace becomes `-`, runs of `-` fold, and
/// dots or dashes at either end go.
pub fn safe(name: &str) -> String {
    static OTHER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^A-Za-z0-9 ._-]").unwrap());
    static DOTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.{2,}").unwrap());
    static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t\n\x0B\x0C\r]+").unwrap());
    static DASHES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-{2,}").unwrap());
    static EDGES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[-.]+|[-.]+$").unwrap());
    let cleaned = OTHER.replace_all(jstr::trim(name), "-");
    let cleaned = DOTS.replace_all(&cleaned, "-");
    let cleaned = SPACES.replace_all(&cleaned, "-");
    let cleaned = DASHES.replace_all(&cleaned, "-");
    let cleaned = EDGES.replace_all(&cleaned, "").into_owned();
    if jstr::is_blank(&cleaned) {
        "untitled".into()
    } else {
        cleaned
    }
}

fn file_name(title: &str) -> String {
    let base = safe(title);
    if base.to_lowercase().ends_with(".md") {
        base
    } else {
        format!("{base}.md")
    }
}

fn unique(used: &mut HashSet<String>, name: String) -> String {
    if used.insert(name.clone()) {
        return name;
    }
    let stem = &name[..name.len() - 3];
    let mut suffix = 2;
    loop {
        let candidate = format!("{stem}-{suffix}.md");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_label_full_of_separators_cannot_escape_its_folder() {
        assert_eq!(safe("../../etc"), "etc");
        assert_eq!(safe("a/../b"), "a-b");
        assert_eq!(safe("   "), "untitled");
        assert_eq!(file_name("CONTEXT.md"), "CONTEXT.md");
        assert_eq!(file_name("Cluster access"), "Cluster-access.md");
    }

    #[test]
    fn a_name_used_twice_in_one_folder_gets_a_number() {
        let mut used = HashSet::new();
        assert_eq!(unique(&mut used, "a.md".into()), "a.md");
        assert_eq!(unique(&mut used, "a.md".into()), "a-2.md");
        assert_eq!(unique(&mut used, "a.md".into()), "a-3.md");
    }
}
