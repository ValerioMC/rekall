use serde::Serialize;

/// Why a backup was taken; it is the last part of the file's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BackupReason {
    Auto,
    Manual,
    BeforeRestore,
    Uploaded,
}

impl BackupReason {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
            Self::BeforeRestore => "before-restore",
            Self::Uploaded => "uploaded",
        }
    }

    pub(super) fn of_slug(slug: &str) -> Option<Self> {
        [Self::Auto, Self::Manual, Self::BeforeRestore, Self::Uploaded].into_iter().find(|r| r.slug() == slug)
    }
}
