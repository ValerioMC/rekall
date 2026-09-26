use rekall_common::jstr;

/// What git answered. `stdout` is untouched (a porcelain status line starts with a meaningful
/// space); `output()` and `error()` are the stripped reads.
#[derive(Clone, Debug)]
pub struct GitResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl GitResult {
    pub fn ok(&self) -> bool {
        self.exit_code == 0
    }

    pub fn output(&self) -> &str {
        jstr::strip(&self.stdout)
    }

    pub fn error(&self) -> &str {
        jstr::strip(&self.stderr)
    }
}
