use std::fs;
use std::path::{Path, PathBuf};

use rekall_common::{RekallError, Result};

/// A project's folder, the only place a diagram's code is read from. A path that climbs out of
/// it, through `..`, an absolute path or a symlink, is refused.
#[derive(Clone, Debug)]
pub struct ProjectFolder {
    root: PathBuf,
}

impl ProjectFolder {
    pub fn open(folder: &Path) -> Result<Self> {
        let root = folder
            .canonicalize()
            .map_err(|_| RekallError::not_found_msg(format!("The project folder {} is not there.", folder.display())))?;
        Ok(Self { root })
    }

    /// `file` resolved under the folder: refused unless its real path is a file still inside it.
    pub fn resolve(&self, file: &str) -> Result<PathBuf> {
        let requested = Path::new(file.trim());
        let joined = if requested.is_absolute() { requested.to_path_buf() } else { self.root.join(requested) };
        let real = joined.canonicalize().map_err(|_| missing(file))?;
        if !real.starts_with(&self.root) {
            return Err(RekallError::illegal(format!("{file} is outside the project folder.")));
        }
        if !real.is_file() {
            return Err(RekallError::illegal(format!("{file} is not a file.")));
        }
        Ok(real)
    }

    /// The file's text, lossily decoded, refused past `max_bytes`.
    pub fn read(&self, file: &str, max_bytes: u64) -> Result<String> {
        let path = self.resolve(file)?;
        let size = fs::metadata(&path).map_err(|_| missing(file))?.len();
        if size > max_bytes {
            return Err(RekallError::illegal(format!("{file} is {size} bytes, too large to read.")));
        }
        let bytes = fs::read(&path).map_err(|_| missing(file))?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

fn missing(file: &str) -> RekallError {
    RekallError::not_found_msg(format!("There is no file {file} in the project folder."))
}
