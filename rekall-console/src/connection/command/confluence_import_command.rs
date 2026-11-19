
/// Whether a re-import rewrites the texts and sections a person already has: without it, only
/// what the project lacks is filled.
#[derive(Clone, Copy, Debug, Default)]
pub struct ConfluenceImportCommand {
    pub replace: bool,
}
