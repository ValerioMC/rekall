use crate::component::ProjectComponentView;
use crate::connection::ConnectionView;
use crate::step::TaskStepView;
use crate::wrapup::WrapupView;

use super::{ContextCommitView, ContextSkillView, DocumentView};

/// One record of a loaded context, materialised so it renders with no database behind it.
///
/// `fields` keeps the order the fields were added in, so a context renders the same way every time.
#[derive(Clone, Debug, Default)]
pub struct ContextRecord {
    pub kind: String,
    pub label: String,
    pub anchor: String,
    pub fields: Vec<(String, String)>,
    pub references: Vec<ContextRecord>,
    pub related: Vec<String>,
    pub documents: Vec<DocumentView>,
    pub steps: Vec<TaskStepView>,
    pub skills: Vec<ContextSkillView>,
    pub commits: Vec<ContextCommitView>,
    pub wrapup: Option<WrapupView>,
    pub components: Vec<ProjectComponentView>,
    /// The project's components, for resolving `[[Name]]` in a task's text only; `None` leaves text untouched.
    pub component_lookup: Option<Vec<ProjectComponentView>>,
    /// What the project reaches outside Rekall; never a credential.
    pub connections: Vec<ConnectionView>,
    pub blueprint: Option<String>,
    pub description: Option<String>,
}
