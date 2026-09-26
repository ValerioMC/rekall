use crate::step::TaskStepView;
use crate::wrapup::WrapupView;

use super::{ContextCommitView, DocumentView};

/// One record of a loaded context, materialised so it renders with no database behind it.
///
/// `fields` keeps the order the fields were added in. The Java record copied them into
/// `Map.copyOf`, whose iteration order is unspecified (and differs between JVM runs); insertion
/// order is the one stable order among those it could print, and the one the code wrote them in.
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
    pub blueprint: Option<String>,
    pub description: Option<String>,
}
