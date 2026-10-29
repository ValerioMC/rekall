use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use rekall_kernel::{Instant, InternalFailure, RekallError, Result};
use tracing::warn;

use super::{ResourceKind, ResourcePath};

/// Past this a file is not something to read into an editor.
pub const MAX_BYTES: u64 = 1024 * 1024;

/// Claude Code's home folder (`~/.claude`) as the console reaches it: files by `ResourcePath`,
/// read whole, written in one step, removed. Symlinks are followed, so a `CLAUDE.md` kept in a
/// dotfiles repository is edited where it really lives.
#[derive(Clone, Debug)]
pub struct ClaudeHome {
    root: PathBuf,
}

impl ClaudeHome {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn absolute(&self, path: &ResourcePath) -> PathBuf {
        self.root.join(path.as_str())
    }

    pub fn metadata(&self, path: &ResourcePath) -> Result<fs::Metadata> {
        fs::metadata(self.absolute(path)).map_err(|failure| failed(path, &failure))
    }

    /// When the file was last written, or none when it is not there.
    pub fn modified_at(&self, path: &ResourcePath) -> Result<Option<Instant>> {
        match self.metadata(path) {
            Ok(metadata) => Ok(Some(modified_of(&metadata))),
            Err(RekallError::NotFound(_)) => Ok(None),
            Err(other) => Err(other),
        }
    }

    pub fn read(&self, path: &ResourcePath) -> Result<String> {
        let size = self.metadata(path)?.len();
        if size > MAX_BYTES {
            return Err(RekallError::illegal(format!("{} is {size} bytes, too large to edit here.", path.as_str())));
        }
        let bytes = fs::read(self.absolute(path)).map_err(|failure| failed(path, &failure))?;
        String::from_utf8(bytes).map_err(|_| RekallError::illegal(format!("{} is not text.", path.as_str())))
    }

    /// The first `limit` bytes, cut where a character ends. Enough for a front matter block.
    pub fn head(&self, path: &ResourcePath, limit: u64) -> Result<String> {
        let mut bytes = Vec::new();
        fs::File::open(self.absolute(path))
            .and_then(|file| file.take(limit).read_to_end(&mut bytes))
            .map_err(|failure| failed(path, &failure))?;
        Ok(match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(cut) => String::from_utf8_lossy(&cut.as_bytes()[..cut.utf8_error().valid_up_to()]).into_owned(),
        })
    }

    /// Writes beside the target and renames over it, so a reader never sees half a file. A symlink
    /// is resolved first, or the rename would replace the link.
    pub fn write(&self, path: &ResourcePath, body: &str) -> Result<()> {
        let absolute = self.absolute(path);
        let target = fs::canonicalize(&absolute).unwrap_or(absolute);
        let folder = target.parent().ok_or_else(|| RekallError::illegal(format!("{} has no folder.", path.as_str())))?;
        fs::create_dir_all(folder).map_err(|failure| failed(path, &failure))?;
        let mut staged = tempfile::NamedTempFile::new_in(folder).map_err(|failure| failed(path, &failure))?;
        staged.write_all(body.as_bytes()).map_err(|failure| failed(path, &failure))?;
        if let Ok(existing) = fs::metadata(&target) {
            fs::set_permissions(staged.path(), existing.permissions()).map_err(|failure| failed(path, &failure))?;
        }
        staged.persist(&target).map_err(|failure| failed(path, &failure.error))?;
        Ok(())
    }

    /// Removes the file; a skill's folder goes with it once nothing else is in it.
    pub fn remove(&self, path: &ResourcePath) -> Result<()> {
        let absolute = self.absolute(path);
        fs::remove_file(&absolute).map_err(|failure| failed(path, &failure))?;
        if path.kind() == ResourceKind::Skill {
            self.prune_empty_folder_of(&absolute);
        }
        Ok(())
    }

    fn prune_empty_folder_of(&self, removed: &Path) {
        let Some(folder) = removed.parent().filter(|folder| *folder != self.root.join("skills")) else { return };
        let empty = fs::read_dir(folder).is_ok_and(|mut entries| entries.next().is_none());
        if empty {
            if let Err(failure) = fs::remove_dir(folder) {
                warn!(%failure, folder = %folder.display(), "Could not remove the emptied skill folder");
            }
        }
    }
}

pub(super) fn modified_of(metadata: &fs::Metadata) -> Instant {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|elapsed| i64::try_from(elapsed.as_millis()).ok())
        .and_then(Instant::from_epoch_millis)
        .unwrap_or(Instant::EPOCH)
}

fn failed(path: &ResourcePath, failure: &std::io::Error) -> RekallError {
    match failure.kind() {
        ErrorKind::NotFound => RekallError::not_found_msg(format!("There is no {} in the Claude folder.", path.as_str())),
        _ => RekallError::internal(InternalFailure::Io, format!("{}: {failure}", path.as_str())),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/resources/claude_home_tests.rs"]
mod tests;
