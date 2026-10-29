# The Claude folder and token usage

## The Claude folder

The Claude screen edits the files Claude Code itself loads: `~/.claude/CLAUDE.md`, skills, commands,
agents and each project's memories. The disk side lives in `rekall-infra` (`resources/`), behind the
`ClaudeResources` port of `rekall-core`; `ClaudeFolderService` in `rekall-console` runs each call off
the async workers, and the routes are in `rekall-http/src/claude/`. The files have no
table: the disk is the record, a session may rewrite a file at any moment, and a copy in a database
would only be a second answer that drifts. The one table, `skill_placement`, holds how the person
groups their own skills, which is Rekall's own preference and no part of the files; it is written by
`SkillGroupService` in `rekall-core/src/skill/`. The usage meter (`UsageReader`, read through
`UsageMeterService`), the install check (`ClaudeCodeSetup`, through `ClaudeInstallService`) and the
terminals' credential helpers (`CredentialDesk`, through `CredentialHandOutService`) follow the same
pattern: the port in `rekall-core`, the implementation in `rekall-infra`, the thin service in
`rekall-console`, the route in `rekall-http`. Claude Code's own token (`credentials`) and the
transcript reader (`transcript`) have no port: no route reaches them, and only `rekall-infra` uses them.

| Piece | What it decides |
|---|---|
| `ResourcePath` | The only way to name a file. Parsing is the guard: a path is one of the known places (`CLAUDE.md`, `skills/`, `commands/`, `agents/`, `memory/` or `projects/<folder>/memory/`, and a plugin's `SKILL.md` under `plugins/cache/<marketplace>/<plugin>/<version>/{,.claude/}skills/<name>/`), all `.md`, with no `..`, hidden or separator-bearing segment (`.claude` is allowed only inside the plugin cache). It classifies the file (`ResourceKind`), says whether it is editable (not under `skills/synced/` or `plugins/`) and where its `MEMORY.md` is |
| `ClaudeHome` | The disk under the root: read whole (UTF-8, capped at 1 MB), head, version (modification time), write, remove. A write goes to a temporary file beside the target and is renamed over it, so a session never reads half a file; a symlink is resolved first, so the link survives. Removing a skill's last file removes its folder |
| `Frontmatter` | The `name`, `description` and `type` of a leading `---` block, read line by line (a folded description takes its indented lines; `type` may sit under `metadata`). Not a YAML parser, and not meant to become one |
| `ResourceScanner` | Builds the catalog: skills are flat files and folders holding a `SKILL.md` (a folder without one is a bucket and is searched), commands and agents nest. Every skill carries a `SkillSource`: `own`, `synced` (claude.ai; the tag comes from the bucket's `manifest.json`) or `plugin` (found through `plugins/installed_plugins.json`, so a superseded version left in the cache is not listed). A file that cannot be described is logged and left out |
| `SkillGroupService` | The only code that writes `skill_placement`: path to group label, through `repo::skill_placement`. It refuses a path that is not one of the person's own skills (synced and plugin skills are listed by their source instead), renames a group by relabelling its rows, and never touches `~/.claude`. A placement for a skill deleted on disk is left, and nothing reads it |
| `MemoryAnalyzer` | The memory review (`GET /api/claude/resources/analysis`), computed on demand and stored nowhere. `TextFingerprint` reduces a text to its distinctive words (no front matter, stopwords, short words or digits; plurals folded) and the pairs of them that sit side by side. A memory is *covered* when one `ReferenceLibrary` document (global `CLAUDE.md`, every skill, and the `CLAUDE.md` / `.claude/CLAUDE.md` of its project folder and each folder above it up to the user's home) holds at least 80% of its words and 50% of its pairs; covered memories are set aside. The rest are joined into clusters by shared words (union-find), and a cluster spanning two or more projects is a promotion candidate, aimed at `CLAUDE.md` or at a skill by its length and whether it has steps. A heuristic over wording, not meaning: it finds restatements and misses paraphrases |
| `WorkingDirectory` | A project folder's real path. Its name flattens every non-alphanumeric character to `-`, which cannot be reversed by reading it. The first `cwd` in a session transcript that flattens back to the folder's name wins; with no transcript the disk is walked from `/` for a folder that flattens to it (backtracking when `mlf` and `mlf-pf` both fit); with neither, the path is a guess (`-` as `/`) and the group is `resolved: false` and `folderExists: false` |
| `ClaudeResourceService` | The rules. A save names the version it was opened from: an existing file is rewritten only from that version, a new file only where none exists, and anything else is a 409. A new memory gets its line in the index beside it and a deleted one loses it (`memory_index`); the instructions and the indexes cannot be deleted; synced skills can be neither written nor deleted |

The console side is `ClaudeResourcesPage` (route `/claude`) over `claudeResources.store`: `ResourceRail`
(library, tabs, filter), `MemoryTree` / `MemoryGroupSection` (a project's memories, matched to a
Rekall project by repo folder, deepest folder first; `model/memory-tree.ts` nests a project under the
nearest folder above it that has memories, and only for groups whose folder is resolved and on disk),
`SkillShelves` (`model/skill-shelves.ts`: the person's skills in their groups, then a locked shelf per
read-only source, with `SkillGroupMenu` and `SkillBucketHeader` for the grouping),
`MemoryReviewPanel` with `PromotionCard` and `ObsoleteCard` (the analysis, suggestions only) and
`ResourceEditor` (header, conflict bar, markdown editor).
`useDiscardGuard` stands between unsaved text and a change of file or route. A save the server
refuses as stale keeps the draft and raises the conflict; **Overwrite it** re-reads the version now
on disk and saves the draft over it.

Sharp edges: there is no live feed, so the screen shows what the disk held when it last loaded or
saved; a project with no memory yet has no group to add one to (create it from the project's own
folder in a session); project-level skills (`<project>/.claude/skills`) and plugin commands and
agents are not listed; the review cannot tell a paraphrase from a different thought.

## Token usage

A task and each of its steps carry the tokens Claude Code generated on them, and a project the sum
of its tasks. Nothing a session sends says what it cost, and the in-app terminal is a TUI, not a
stream-json process, so the figures are read from where Claude Code already writes them: the
session transcripts under `~/.claude/projects`, where every model call carries its `usage`.

| Piece | What it decides |
|---|---|
| `TranscriptFolder` (rekall-infra) | Which transcripts belong to a project: the folder Claude Code names after the project's `repo_folder` (every non-alphanumeric character becomes `-`), and the folders of subfolders under it. A session's subagents are in `<session>/subagents/` |
| `SessionLedger` (rekall-infra) | What each call is charged to, and on which UTC day, by the line's own timestamp; a line with none takes the day it is read. `/rk` or a `rekall_context` naming one task moves the session to that task; `rekall_step` `running` to the step, `claimed` or `open` back to the task, after charging the call that moved it. A task named again without a step keeps the running step, so a wrapup written mid-step costs the step. Subagent calls go to what the session was on at their timestamp. A message spread over several lines is charged once, by its id. Calls before any Rekall work are not counted |
| `TranscriptScanner` (rekall-infra) | Reads, every `rekall.claude.transcript-scan-seconds`, each session whose files have grown since `transcript_scan` last saw them; an unchanged transcript is never read twice. A line that does not parse is skipped, and a field in a shape the reader does not know reads as empty rather than losing the line's tokens |
| `TokenUsageService` (rekall-core) | Resolves the anchors and replaces the session's `token_usage` rows whole, so a reread never double counts. A task that no longer resolves is dropped; a step that does not resolve leaves its tokens on the task, and deleting a step sets its rows' `step_id` to null for the same reason. A row is a session's spend on one task or step on one day (`spent_on`, midnight UTC), so a task's spend can be told day by day; migration `041` stamps old rows with the day they were read and clears `transcript_scan` so every transcript is read again with real days. A read that changes a task's tallies or days publishes a `token-usage` frame with all of them |

The figure the console shows is output tokens, what the model generated; input, cache write and
cache read are kept beside it and shown in the tooltip. `GET /api/token-usage` returns one tally per
task and step, `GET /api/token-usage/daily` one entry per task and day, and `useTokenUsageStore`
sums a project from its tasks in the browser, so no total is stored that could disagree with its
parts. A task's header (`TaskTitleRow`) holds the name, its status flags, `CopyAnchorButton` and the
token chip in every pane; the description pane adds `ContextSizeChip` (what `/rk` hands a session)
right of the token chip through the row's `trailing` slot, and its tag and skills line
(`TaskLabelsStrip`) holds only the tag control and the skills. Both chips share the tag and skill chips' shape (24px, `rounded-key`). `TokenTrendChip` is the chip for any set of tasks (`TaskTokenChip`
for one, `ProjectTokenChip` for a project in the navigator header); its sparkline and popover chart
(`TokenTrendPanel`, `TokenTrendChart`) are drawn from the store's days by `trendOf`, a running total
with one point per calendar day. `TaskTagControl` and `TagPicker` change a task's tag in place. The tokens follow the anchors, not the folder, and
any session counts, whether Rekall opened its terminal or not.

Sharp edges: a session started outside every project's folder is never read, and a step named by
number in an old transcript resolves against the checklist as it is when the transcript is reread.

