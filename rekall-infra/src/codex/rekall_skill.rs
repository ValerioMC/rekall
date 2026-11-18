use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use rekall_kernel::{InternalFailure, RekallError, Result};
use tempfile::NamedTempFile;
use tracing::warn;

use crate::installer::{PACKAGED_COMMAND, PACKAGED_MODES};

const SKILL_HEADER: &str = "---\nname: rekall\ndescription: Load and work a Rekall task or project by anchor, with modes for planning, claiming steps, writing notes and wrapups, and generating diagrams.\n---\n\n";
const CODEX_ARGUMENTS: &str = "The argument line is the text supplied when the user invokes this skill. Use it unchanged. References to `/rk` below denote the same workflow. Invoke assigned skills using the Codex skills available to the session and report unavailable ones.";

pub(super) struct RekallSkill {
    folder: PathBuf,
}

impl RekallSkill {
    pub(super) fn under(codex_home: &Path) -> Self {
        Self { folder: codex_home.join("skills/rekall") }
    }

    pub(super) fn installed(&self) -> bool {
        self.files().iter().all(|(file, expected)| match fs::read_to_string(file) {
            Ok(found) => &found == expected,
            Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => false,
            Err(failure) => {
                warn!(%failure, file = %file.display(), "Could not read the installed Rekall skill");
                false
            }
        })
    }

    pub(super) fn install(&self) -> Result<()> {
        for (file, contents) in self.files() {
            write_skill_file(&file, &contents).map_err(|failure| RekallError::internal(
                InternalFailure::Io,
                format!("Could not install the Rekall skill at {}: {failure}", file.display()),
            ))?;
        }
        Ok(())
    }

    fn files(&self) -> Vec<(PathBuf, String)> {
        let modes = PACKAGED_MODES.iter().map(|(name, text)| (self.folder.join("references").join(name), (*text).to_owned()));
        modes.chain(std::iter::once((self.folder.join("SKILL.md"), skill_document()))).collect()
    }
}

fn skill_document() -> String {
    let body = PACKAGED_COMMAND.splitn(3, "---").nth(2).expect("the packaged /rk command has frontmatter");
    let body = body.replace("The arguments are:\n\n$ARGUMENTS", CODEX_ARGUMENTS)
        .replace("read its file with Read", "read its file")
        .replace(
            "The files are in `~/.claude/rk/`; if one is not there, read it from `.claude/rk/` in the working\ndirectory.",
            "Read the files from `references/` beside this SKILL.md.",
        )
        .replace("Claude skills assigned", "skills assigned")
        .replace("Invoke each one with the Skill tool", "Load each available skill")
        .replace("pass the text inside the quotes as the\n  Skill tool's `args`", "use the text inside the quotes as the skill arguments");
    format!("{SKILL_HEADER}{}", body.trim_start())
}

fn write_skill_file(file: &Path, contents: &str) -> std::io::Result<()> {
    let directory = file.parent().expect("a skill file lies under the skill directory");
    fs::create_dir_all(directory)?;
    let mut temporary = NamedTempFile::new_in(directory)?;
    temporary.write_all(contents.as_bytes())?;
    temporary.persist(file)?;
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/codex/rekall_skill_tests.rs"]
mod tests;
