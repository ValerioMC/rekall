use std::sync::LazyLock;

use regex::Regex;

use crate::commit::view::{ChangeKind, ChangedFileView, DiffHunkView, DiffLineKind, DiffLineView};

static HUNK_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@ ?(.*)$").expect("a valid pattern"));

/// Reads the patch git prints (`git show`, `git diff`) into files, hunks and numbered lines.
/// A patch cut in the middle of a hunk still yields the lines read up to the cut.
pub struct UnifiedDiff;

impl UnifiedDiff {
    pub fn parse(patch: &str) -> Vec<ChangedFileView> {
        let mut reader = PatchReader::default();
        for line in patch.lines() {
            reader.feed(line);
        }
        reader.finish()
    }
}

#[derive(Default)]
struct PatchReader {
    files: Vec<ChangedFileView>,
    file: Option<FileDraft>,
    hunk: Option<HunkDraft>,
}

struct FileDraft {
    header_path: String,
    path: Option<String>,
    old_path: Option<String>,
    kind: ChangeKind,
    binary: bool,
    hunks: Vec<DiffHunkView>,
}

struct HunkDraft {
    hunk: DiffHunkView,
    old_remaining: u32,
    new_remaining: u32,
    next_old: u32,
    next_new: u32,
}

impl PatchReader {
    fn feed(&mut self, line: &str) {
        if self.hunk.as_ref().is_some_and(HunkDraft::is_open) {
            self.read_hunk_line(line);
            return;
        }
        self.close_hunk();
        if let Some(rest) = line.strip_prefix("diff --git ") {
            self.start_file(rest);
        } else if line.starts_with("@@") {
            self.start_hunk(line);
        } else if let Some(file) = self.file.as_mut() {
            file.read_header(line);
        }
    }

    fn read_hunk_line(&mut self, line: &str) {
        let Some(draft) = self.hunk.as_mut() else { return };
        if line.starts_with('\\') {
            return;
        }
        let (kind, text) = match line.chars().next() {
            Some('+') => (DiffLineKind::Added, &line[1..]),
            Some('-') => (DiffLineKind::Removed, &line[1..]),
            Some(' ') => (DiffLineKind::Context, &line[1..]),
            _ => (DiffLineKind::Context, line),
        };
        draft.push(kind, text);
    }

    fn start_file(&mut self, header: &str) {
        self.close_file();
        self.file = Some(FileDraft::new(header));
    }

    fn start_hunk(&mut self, line: &str) {
        let Some(captures) = HUNK_HEADER.captures(line) else { return };
        let number = |index: usize, default: u32| captures.get(index).and_then(|m| m.as_str().parse().ok()).unwrap_or(default);
        let (old_start, old_lines) = (number(1, 0), number(2, 1));
        let (new_start, new_lines) = (number(3, 0), number(4, 1));
        let context = captures.get(5).map_or("", |m| m.as_str()).to_string();
        self.hunk = Some(HunkDraft {
            hunk: DiffHunkView { old_start, old_lines, new_start, new_lines, context, lines: Vec::new() },
            old_remaining: old_lines,
            new_remaining: new_lines,
            next_old: old_start,
            next_new: new_start,
        });
    }

    fn close_hunk(&mut self) {
        if let (Some(draft), Some(file)) = (self.hunk.take(), self.file.as_mut()) {
            file.hunks.push(draft.hunk);
        }
    }

    fn close_file(&mut self) {
        self.close_hunk();
        if let Some(file) = self.file.take() {
            self.files.push(file.finish());
        }
    }

    fn finish(mut self) -> Vec<ChangedFileView> {
        self.close_file();
        self.files
    }
}

impl HunkDraft {
    fn is_open(&self) -> bool {
        self.old_remaining > 0 || self.new_remaining > 0
    }

    fn push(&mut self, kind: DiffLineKind, text: &str) {
        let (old_number, new_number) = match kind {
            DiffLineKind::Context => (Some(self.take_old()), Some(self.take_new())),
            DiffLineKind::Removed => (Some(self.take_old()), None),
            DiffLineKind::Added => (None, Some(self.take_new())),
        };
        self.hunk.lines.push(DiffLineView { kind, old_number, new_number, text: text.to_string() });
    }

    fn take_old(&mut self) -> u32 {
        let number = self.next_old;
        self.next_old += 1;
        self.old_remaining = self.old_remaining.saturating_sub(1);
        number
    }

    fn take_new(&mut self) -> u32 {
        let number = self.next_new;
        self.next_new += 1;
        self.new_remaining = self.new_remaining.saturating_sub(1);
        number
    }
}

impl FileDraft {
    fn new(header: &str) -> Self {
        Self {
            header_path: path_of_header(header),
            path: None,
            old_path: None,
            kind: ChangeKind::Modified,
            binary: false,
            hunks: Vec::new(),
        }
    }

    fn read_header(&mut self, line: &str) {
        if line.starts_with("new file mode") {
            self.kind = ChangeKind::Added;
        } else if line.starts_with("deleted file mode") {
            self.kind = ChangeKind::Deleted;
        } else if let Some(from) = line.strip_prefix("rename from ") {
            self.kind = ChangeKind::Renamed;
            self.old_path = Some(from.to_string());
        } else if let Some(to) = line.strip_prefix("rename to ") {
            self.path = Some(to.to_string());
        } else if let Some(target) = line.strip_prefix("+++ b/") {
            self.path = Some(target.to_string());
        } else if let Some(source) = line.strip_prefix("--- a/") {
            self.path.get_or_insert_with(|| source.to_string());
        } else if line.starts_with("Binary files") || line.starts_with("GIT binary patch") {
            self.binary = true;
        }
    }

    fn finish(self) -> ChangedFileView {
        let additions = self.hunks.iter().flat_map(|h| &h.lines).filter(|l| l.kind == DiffLineKind::Added).count() as u32;
        let deletions = self.hunks.iter().flat_map(|h| &h.lines).filter(|l| l.kind == DiffLineKind::Removed).count() as u32;
        ChangedFileView {
            path: self.path.unwrap_or(self.header_path),
            old_path: self.old_path,
            kind: self.kind,
            binary: self.binary,
            additions,
            deletions,
            hunks: self.hunks,
        }
    }
}

/// The path of `a/x b/x` when no header names it: the part after the last ` b/`.
fn path_of_header(header: &str) -> String {
    match header.rfind(" b/") {
        Some(at) => header[at + 3..].to_string(),
        None => header.to_string(),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/commit/model/unified_diff_tests.rs"]
mod tests;
