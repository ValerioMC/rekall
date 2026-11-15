use std::fmt;

/// The project an anchor names: its label, and the company when two companies share that label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectRef {
    pub company: Option<String>,
    pub label: String,
}

impl ProjectRef {
    pub fn bare(label: &str) -> Self {
        Self { company: None, label: label.to_string() }
    }

    pub fn in_company(company: &str, label: &str) -> Self {
        Self { company: Some(company.to_string()), label: label.to_string() }
    }
}

impl fmt::Display for ProjectRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.company {
            Some(company) => write!(formatter, "company:{company} project:{}", self.label),
            None => write!(formatter, "project:{}", self.label),
        }
    }
}
