use crate::component::ProjectComponentView;
use crate::step::TaskStepView;
use crate::wrapup::WrapupView;

use super::{ContextCommitView, DocumentView};

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
    pub commits: Vec<ContextCommitView>,
    pub wrapup: Option<WrapupView>,
    pub components: Vec<ProjectComponentView>,
    pub blueprint: Option<String>,
    pub description: Option<String>,
}
