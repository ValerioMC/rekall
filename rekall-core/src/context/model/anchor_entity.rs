use rekall_kernel::{RekallError, Result};

/// What an anchor's `entity:` part names. The aliases are the ones callers have always typed,
/// English and Italian, in any letter case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorEntity {
    Company,
    Project,
    Task,
    Note,
}

impl AnchorEntity {
    const ALL: [AnchorEntity; 4] = [Self::Company, Self::Project, Self::Task, Self::Note];

    pub fn parse(text: &str) -> Result<Self> {
        match text.to_lowercase().as_str() {
            "company" | "companies" | "azienda" | "aziende" | "cliente" => Ok(Self::Company),
            "project" | "projects" | "progetto" | "progetti" => Ok(Self::Project),
            "task" | "tasks" | "issue" | "issues" | "attivita" => Ok(Self::Task),
            "note" | "notes" | "nota" | "document" => Ok(Self::Note),
            _ => Err(RekallError::unknown_anchor(format!(
                "No entity matches '{text}'. Known entities: {}",
                Self::known_names()
            ))),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Company => "company",
            Self::Project => "project",
            Self::Task => "task",
            Self::Note => "note",
        }
    }

    pub fn known_names() -> String {
        Self::ALL.map(Self::name).join(", ")
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/context/model/anchor_entity_tests.rs"]
mod tests;
