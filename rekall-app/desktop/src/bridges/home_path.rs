use std::path::PathBuf;

pub(super) fn expand_tilde(path: &str) -> PathBuf {
    match path.strip_prefix('~') {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => {
            PathBuf::from(format!("{}{rest}", dirs::home_dir().unwrap_or_default().to_string_lossy()))
        }
        _ => PathBuf::from(path),
    }
}
