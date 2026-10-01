//! Where a note lives: everywhere, with one company, or with one project. The scope owns the
//! note, so taking it off its last task leaves it there instead of deleting it, and every task
//! it sits on has to be inside it. Sessions still read a note only through the tasks it is on.

use rekall_common::Id;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "UPPERCASE")]
pub enum NoteScope {
    Global,
    Company { id: Id },
    Project { id: Id },
}

impl NoteScope {
    /// The scope the two nullable columns spell. A project wins over a company: the columns are
    /// written so that at most one is set, and the project is the narrower claim.
    pub fn of_columns(company_id: Option<Id>, project_id: Option<Id>) -> Self {
        match (project_id, company_id) {
            (Some(id), _) => Self::Project { id },
            (None, Some(id)) => Self::Company { id },
            (None, None) => Self::Global,
        }
    }

    /// `(scope_company_id, scope_project_id)` as stored.
    pub fn columns(self) -> (Option<Id>, Option<Id>) {
        match self {
            Self::Global => (None, None),
            Self::Company { id } => (Some(id), None),
            Self::Project { id } => (None, Some(id)),
        }
    }

    /// Whether a task in `project_id`, owned by `company_id`, may carry a note of this scope.
    pub fn admits(self, company_id: Id, project_id: Id) -> bool {
        match self {
            Self::Global => true,
            Self::Company { id } => id == company_id,
            Self::Project { id } => id == project_id,
        }
    }

    /// The narrowest scope holding every `(company, project)` given: their one project, else
    /// their one company, else global. No places at all is global.
    pub fn narrowest(places: &[(Id, Id)]) -> Self {
        let Some(&(first_company, first_project)) = places.first() else {
            return Self::Global;
        };
        if places.iter().all(|&(_, project)| project == first_project) {
            Self::Project { id: first_project }
        } else if places.iter().all(|&(company, _)| company == first_company) {
            Self::Company { id: first_company }
        } else {
            Self::Global
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/note_scope_tests.rs"]
mod tests;
