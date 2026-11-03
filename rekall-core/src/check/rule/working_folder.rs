//! Where a check runs: the project's folder, or a folder inside it.

use std::path::{Component, Path, PathBuf};

use rekall_kernel::{jstr, RekallError, Result};

/// `relative` under `project_folder`, or the project folder itself when there is none. A folder that
/// is absolute or climbs out with `..` is refused as the caller's mistake.
pub fn working_folder(project_folder: &Path, relative: Option<&str>) -> Result<PathBuf> {
    let Some(relative) = relative.filter(|text| !jstr::is_blank(text)) else {
        return Ok(project_folder.to_path_buf());
    };
    let relative = Path::new(relative);
    let stays_inside = relative.components().all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
    if !stays_inside {
        return Err(RekallError::illegal(format!("The folder '{}' is not inside the project's folder.", relative.display())));
    }
    Ok(project_folder.join(relative))
}

#[cfg(test)]
#[path = "../../../tests/unit/check/rule/working_folder_tests.rs"]
mod tests;
