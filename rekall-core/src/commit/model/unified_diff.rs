use std::{ops::Range, sync::LazyLock};

use regex::Regex;

use crate::commit::view::{ChangeKind, ChangedFileMetadata, ChangedFileView, DiffHunkView, DiffLineKind, DiffLineView};

static HUNK_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@ ?(.*)$").expect("a valid pattern"));

/// Reads the patch git prints (`git show`, `git diff`) into files, hunks and numbered lines.
/// A patch cut in the middle of a hunk still yields the lines read up to the cut.
pub struct UnifiedDiff;

impl UnifiedDiff {
    pub fn parse(patch: &str) -> Vec<ChangedFileView> {
        Self::scan(patch, true).into_iter().map(ParsedFile::into_view).collect()
    }

    pub(in crate::commit) fn index(patch: &str) -> Vec<DiffFileSpan> {
        Self::scan(patch, false).into_iter().map(|file| file.index).collect()
    }

    fn scan(patch: &str, read_hunks: bool) -> Vec<ParsedFile> {
        let mut reader = PatchReader { read_hunks, ..PatchReader::default() };
        let mut offset = 0;
        for raw_line in patch.split_inclusive('\n') {
            let line = raw_line.strip_suffix('\n').map_or(raw_line, |line| line.strip_suffix('\r').unwrap_or(line));
            reader.feed(line, offset);
            offset += raw_line.len();
        }
        reader.close_file(offset);
        reader.files
    }
}

pub(in crate::commit) struct DiffFileSpan {
    pub metadata: ChangedFileMetadata,
    pub bytes: Range<usize>,
}

struct ParsedFile {
    index: DiffFileSpan,
    hunks: Vec<DiffHunkView>,
}

impl ParsedFile {
    fn into_view(self) -> ChangedFileView {
        let ChangedFileMetadata { path, old_path, kind, binary, additions, deletions } = self.index.metadata;
        ChangedFileView { path, old_path, kind, binary, additions, deletions, hunks: self.hunks }
    }
}

#[derive(Default)]
struct PatchReader {
    read_hunks: bool,
    files: Vec<ParsedFile>,
    file: Option<FileDraft>,
    hunk: Option<HunkDraft>,
}

struct FileDraft {
    offset: usize,
    header_path: String,
    path: Option<String>,
    old_path: Option<String>,
    kind: ChangeKind,
    binary: bool,
    additions: u32,
    deletions: u32,
    hunks: Vec<DiffHunkView>,
}

struct HunkDraft {
    view: Option<DiffHunkView>,
    old_remaining: u32,
    new_remaining: u32,
    next_old: u32,
    next_new: u32,
}

impl PatchReader {
    fn feed(&mut self, line: &str, offset: usize) {
        if self.hunk.as_ref().is_some_and(HunkDraft::is_open) {
            self.read_hunk_line(line);
            return;
        }
        self.close_hunk();
        if let Some(rest) = line.strip_prefix("diff --git ") {
            self.close_file(offset);
            self.file = Some(FileDraft::new(rest, offset));
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
        let (kind, text) = match line.as_bytes().first() {
            Some(b'+') => (DiffLineKind::Added, &line[1..]),
            Some(b'-') => (DiffLineKind::Removed, &line[1..]),
            Some(b' ') => (DiffLineKind::Context, &line[1..]),
            _ => (DiffLineKind::Context, line),
        };
        draft.push(kind, text);
        if let Some(file) = self.file.as_mut() {
            file.additions += u32::from(kind == DiffLineKind::Added);
            file.deletions += u32::from(kind == DiffLineKind::Removed);
        }
    }

    fn start_hunk(&mut self, line: &str) {
        let Some(captures) = HUNK_HEADER.captures(line) else { return };
        let number = |index: usize, default: u32| captures.get(index).and_then(|m| m.as_str().parse().ok()).unwrap_or(default);
        let (old_start, old_lines) = (number(1, 0), number(2, 1));
        let (new_start, new_lines) = (number(3, 0), number(4, 1));
        let view = self.read_hunks.then(|| {
            let context = captures.get(5).map_or("", |m| m.as_str()).to_string();
            DiffHunkView { old_start, old_lines, new_start, new_lines, context, lines: Vec::new() }
        });
        self.hunk = Some(HunkDraft {
            view,
            old_remaining: old_lines,
            new_remaining: new_lines,
            next_old: old_start,
            next_new: new_start,
        });
    }

    fn close_hunk(&mut self) {
        if let (Some(view), Some(file)) = (self.hunk.take().and_then(|draft| draft.view), self.file.as_mut()) {
            file.hunks.push(view);
        }
    }

    fn close_file(&mut self, offset: usize) {
        self.close_hunk();
        if let Some(file) = self.file.take() {
            self.files.push(file.finish(offset));
        }
    }
}

impl HunkDraft {
    fn is_open(&self) -> bool {
        self.old_remaining > 0 || self.new_remaining > 0
    }

    fn push(&mut self, kind: DiffLineKind, text: &str) {
        let consumes_old = kind != DiffLineKind::Added;
        let consumes_new = kind != DiffLineKind::Removed;
        self.old_remaining = self.old_remaining.saturating_sub(u32::from(consumes_old));
        self.new_remaining = self.new_remaining.saturating_sub(u32::from(consumes_new));
        if let Some(view) = self.view.as_mut() {
            let old_number = consumes_old.then_some(self.next_old);
            let new_number = consumes_new.then_some(self.next_new);
            self.next_old += u32::from(consumes_old);
            self.next_new += u32::from(consumes_new);
            view.lines.push(DiffLineView { kind, old_number, new_number, text: text.to_string() });
        }
    }
}

impl FileDraft {
    fn new(header: &str, offset: usize) -> Self {
        Self {
            offset,
            header_path: path_of_header(header),
            path: None,
            old_path: None,
            kind: ChangeKind::Modified,
            binary: false,
            additions: 0,
            deletions: 0,
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

    fn finish(self, end: usize) -> ParsedFile {
        let metadata = ChangedFileMetadata {
            path: self.path.unwrap_or(self.header_path),
            old_path: self.old_path,
            kind: self.kind,
            binary: self.binary,
            additions: self.additions,
            deletions: self.deletions,
        };
        let index = DiffFileSpan { metadata, bytes: self.offset..end };
        ParsedFile { index, hunks: self.hunks }
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
