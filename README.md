# Rekall

Rekall stores your projects, tasks and markdown notes, and loads one task in full into a Claude Code session on a single command.

Projects, tasks and notes live in a local app. `/rk project:vega task:report-builder` loads that task, its project, its checklist and every attached note. `/rk project:vega task:report-builder wrapup` records what the implementation is now, so the next session starts from the current state instead of reading the code back.

Source-available, not open source. See [License](#license).

## Requirements

| Tool  | Version |
|-------|---------|
| Java  | 25      |
| Maven | 3.9+    |
| Node  | 22+     |
| pnpm  | 9+      |

The database is an H2 file under `./data`. No database server, Docker or cluster.

## Run

```bash
make run     # build the frontend, then start on http://localhost:47355
make build   # the above, plus the packaged jar
make ui      # the frontend alone
make reset   # delete the database file (no undo)
make console # H2 shell on the database
```

`make run` and `make build` build the UI first. The frontend compiles to `rekall-ui/dist` (git-ignored) and `rekall-app` copies it into the jar under `static/` at package time. Packaging without a built UI fails with instructions to run `make ui`.

| Service | Address                      |
|---------|------------------------------|
| UI      | `http://localhost:47355`     |
| MCP     | `http://localhost:47355/mcp` |

Port 47355 is fixed because the MCP endpoint is registered with Claude Code by URL, so a clash breaks the registration rather than moving the app. `SERVER_PORT` overrides it for the server, the `.app` launcher and the tests.

## macOS application

A disk image for Apple Silicon is published on every commit to `main`.

| | |
|---|---|
| Download | [Rekall-macos-arm64.dmg](https://github.com/ValerioMC/rekall/releases/download/latest/Rekall-macos-arm64.dmg) |
| Needs | macOS 13 or later, Apple Silicon |
| Install | Drag Rekall onto Applications, then run the `xattr` command below once |

The image tracks the head of `main` and changes under the link without notice. A fixed build is a `v*` tag. It builds on a `macos-14` runner from `.github/workflows/release.yml` and `scripts/macos-bundle.sh`. There is no Intel, Windows or Linux bundle; the jar on the same release runs on any platform with a JVM 25.

The published image is the **jvm** flavour. The GraalVM (native) build reaches 5.9 GB of heap on a 7 GB runner and the watchdog aborts it after about 30 minutes, so it builds only locally.

### Build locally

```bash
make dmg-native   # GraalVM binary in the bundle. Needs GraalVM as JAVA_HOME
make dmg-jvm      # jar plus a bundled Java runtime. Any JDK 25
```

Both need the Xcode Command Line Tools for `swiftc`. Both write `dist/Rekall-<version>-<flavour>-<arch>.dmg` and install it into `/Applications` on the build machine, replacing an existing `Rekall.app` and relaunching a running copy. `REKALL_INSTALL=0 make dmg-jvm` stops at the disk image.

| | `dmg-native` | `dmg-jvm` |
|---|---|---|
| Payload | `Contents/Resources/rekall-app` | `Contents/runtime` plus `rekall-app.jar` |
| Disk image | 91 MB | 99 MB |
| First screen | under a second | about three seconds |
| Build | GraalVM, 4 to 8 minutes | any JDK 25, under a minute |

### Running the app

The bundle executable is `packaging/macos/Launcher.swift`. It shows a splash screen while the server boots, then loads the console once port 47355 answers. Quitting sends `SIGTERM` so the H2 file closes cleanly. Server output goes to `~/Library/Logs/Rekall/server.log` (View > Open Server Log).

The app uses the same port, the same `~/.rekall/config.json` and the same MCP endpoint as `make run`. If something is already listening on 47355, the app attaches to it instead of starting a second server. The folder icon in the database field opens the system folder chooser, which a browser tab cannot do; in a browser that field stays a typed input.

The bundle is signed ad hoc, which is enough for the machine that built it. A disk image downloaded through a browser on another machine is quarantined and needs one command before it opens:

```bash
xattr -dr com.apple.quarantine /Applications/Rekall.app
```

## Connect Claude Code

**Settings > Claude Code** in the app registers the MCP server for every folder and installs the `/rk` command in one click. It also repairs a registration that points at the wrong port, carries an older command, or is shadowed in one folder by a registration made without `--scope user`.

From a terminal:

```bash
make mcp-add    # claude mcp add --scope user --transport http rekall http://localhost:47355/mcp
make mcp-check  # verify the endpoint answers
cp .claude/commands/rk.md ~/.claude/commands/rk.md
```

`--scope user` registers the server for every directory. Without it, `/rk` works only in the directory `claude mcp add` ran from. Restart any open session, then:

```
/rk project:vega task:report-builder
```

**Open in terminal**, on a task or a project, hands the session to your own terminal app in the project's folder with `/rk` already running. Set the folder in the **Folder** field on the project page. The terminal is iTerm2 when installed, Terminal.app otherwise. A switch in **Settings > Claude Code** adds `--dangerously-skip-permissions`; it is stored on the machine, not in the database. This button works only inside Rekall.app.

## Terminal in Rekall

**Run here**, next to **Open in terminal** on the description and steps panes, or `c` from any task, runs a real terminal inside the app. The backend starts the interactive `claude` in a pseudo-terminal (pty4j) in the project's folder, types `/rk <anchors>` as the first line, and hands it to you; the pane renders it with `xterm.js` and the bytes travel both ways over one WebSocket at `/api/terminal/{id}/io`. Because it is the same binary run the same way as in your own terminal, its prompt caching, context compaction and `/context` read-outs behave identically, and a permission prompt actually renders and can be answered. Works in a plain browser, not only Rekall.app.

One terminal per task, the way one terminal window is. A second open on a task that already has one just refocuses it; opening on a different step moves the checklist marker (the old step back to open, the new one to running) without touching the process. **Restart** kills the `claude` process and starts a fresh one on the same task; **Close** ends it. Opening on a step marks that step `RUNNING` while the terminal is on it; on a task with no checklist the review line goes `RUNNING` instead, and both are released when the terminal closes. Nothing is persisted: a restart of Rekall clears every terminal and releases any step it left running, and reopening the pane replays a bounded scrollback so it repaints.

Live work sits in one bar in the bottom right corner of every screen: a segment for running timers (with the newest timer's clock) and a segment for live terminals, each shown only while it has something to count. A segment opens its sheet above the bar, one sheet at a time; pressing the other segment switches, and pressing the same one again, Escape, or a click anywhere else closes it. From the sheet a row jumps to its task (the terminal one also opens the terminal pane), stops the timer or terminal, and, for a terminal, logs the latest commit against the task and step it was opened on. The bar reports the room it takes so nothing else in that corner sits under it, whichever segments are present; the terminal segment steps aside while the terminal pane is on screen, and the bar leaves entirely when nothing is live.

**Settings > Claude Code** picks the model and effort a new terminal starts with, and the **skip permissions** switch:

- **Model** (`Account default`, `Sonnet`, `Fable`, `Opus`, `Haiku`): anything but the default adds `--model <alias>`. Each alias is Claude Code's own name for the latest model of that family, so no version is pinned.
- **Reasoning effort** (`Account default`, `Low`, `Medium`, `High`, `Extra-high`, `Max`): anything but the default adds `--effort <level>`.
- **Skip permissions** adds `--dangerously-skip-permissions`; with it off, an interactive permission prompt in the terminal is yours to answer.

All three are stored on the machine, not in the database. It works in both the jvm and native macOS bundles: the pty4j/JNA GraalVM metadata is committed under `rekall-app/src/main/resources/META-INF/native-image/`.

The top bar carries a usage meter: the current 5-hour session as a ring with its percentage and time to reset, and, on hover, a bar per window including the weekly per-model limits. The figures are the ones Claude Code's own `/usage` shows, read with the OAuth token Claude Code stores (every `Claude Code-credentials` item in the macOS keychain, else `~/.claude/.credentials.json`; the token expiring last wins, and an expired one counts as no token). With no valid token the meter asks you to open a Claude Code terminal, which signs in or refreshes the token; when Anthropic cannot be reached it holds the last figures. Readings are not real time: one when the console opens, one every five minutes while it is on screen, one when the window comes back to the front after a minute away, and one whenever you click the blank meter or **Check again** in its popover (`GET /api/claude/usage?refresh=true`, past the server's cache). If Anthropic answers 429 the meter shows **Wait** with the time left and takes nothing, not even a manual check, until that `Retry-After` has passed: asking again is what extends the block.

| Property | Default | Meaning |
|---|---|---|
| `rekall.claude.cli-path` | search `PATH` and the usual install dirs | Absolute path to `claude`, overriding discovery |
| `rekall.claude.usage-url` | `https://api.anthropic.com/api/oauth/usage` | Where the usage meter reads session and weekly limits |
| `rekall.terminal.max-sessions` | `8` | Terminals allowed at once |
| `rekall.terminal.idle-minutes` | `120` | A terminal untouched this long is closed by the sweep |
| `rekall.terminal.sweep-minutes` | `5` | How often the idle sweep runs |
| `rekall.terminal.scrollback-bytes` | `131072` | Bytes of output replayed to a pane that reopens |

## Anchor syntax

An anchor is `entity:value`, where the entity is `company`, `project` or `task` and the value is the record's **label** (its `name` for a company). Label rules are under [Model](#model).

| Form | Meaning |
|------|---------|
| `/rk project:vega task:report-builder` | The project and that task, both in full |
| `/rk project:vega` | The project, plus its tasks as a list of anchors |
| `/rk vega report-builder` | Positional. Works while each term matches one record |
| `/rk task:"report builder"` | Quote a value containing spaces |
| `/rk project:vega task:report-builder wrapup` | Write the task's wrapup instead of loading it |

An anchor brings back the record, everything it references resolved in full with their notes, what references it as anchors, and its own markdown. A note can be attached to several tasks and arrives with each. If a bare term matches more than one record, the candidates come back and nothing loads. `project:` disambiguates a label two projects share.

## MCP tools

| Tool | Access | Effect |
|------|--------|--------|
| `rekall_context` | read | Resolve anchors, return markdown |
| `rekall_wrapup` | write | Replace one task's wrapup |
| `rekall_step` | write | Move one step: `open` to `running` to `claimed` |
| `rekall_record_commit` | write | Log a commit of the project's repo folder against one task or step |

There is no query, get or schema tool. `rekall_step` refuses `done`; that state is set by hand in the console.

## Wrapup

A task has at most one wrapup: what its implementation currently is, not a changelog. It names code by the code's own names (class, file, endpoint, table) and does not transcribe field lists or signatures. It is capped at 20,000 characters, against 100,000 for a note.

A quoted term after `wrapup` is a directive on what to write. It can narrow the subject, dictate the wording, or set the language or length.

```
/rk project:vega task:report-builder wrapup "export module only"
```

The description pane and the steps pane both carry a **Generate the wrapup every session** toggle under the header, so it is set from whichever surface the work is driven from. Turning it on reveals an optional directive field. Set once on the task, the toggle stands in for the quoted term: the context load then tells the session a wrapup is expected every time without being asked, and the words it should follow. Turning the toggle off drops the directive with it.

A wrapup written after a step finishes folds that step's work into the same description. The console counts steps ticked and notes added since the wrapup was last written. You can edit the wrapup in the console; the next `/rk … wrapup` replaces it and the tool reports when it overwrites a hand edit.

## Steps

A step sits on a line: **draft** while you are still wording it, **open** once you promote it and it is ready to work, **running** while a session works it, **claimed** when the session reports it finished, **done** when you accept it. Each step is a title and an optional markdown detail. The pane opens on the first step whose work is not finished.

A new step is created as a draft and sits on a staging shelf below the checklist, off the rail. **Promote** moves it onto the list; an open step not yet started can go back with **To draft**. A draft is not work: it stays out of the checklist a session reads, `rekall_step` will not move it, and the navigator's step count ignores it. A task that holds only drafts is still treated as having no checklist.

A session drives its own checklist over `/rk`:

```
/rk project:vega task:report-builder step:3 start   # step 3 -> running
/rk project:vega task:report-builder step:3 done    # step 3 -> claimed
```

The console holds one `text/event-stream` connection (`GET /api/steps/stream`) carrying four frames: `steps` for a checklist, `task-review` for a stepless task's review line, `wrapup` for a wrapup write or delete, and `commit-reference` for a commit logged against a task or step. A step moved from an in-app terminal, a box ticked in another window, a wrapup written over MCP, or a commit logged by a session all land without a reload.

Claude receives an open or running step with its detail, tagged `(in progress)` or `(claimed, …)`. A finished step arrives as its title alone. A draft step is not sent at all, only counted as `draft="N"` on the `<steps>` tag. A step ticked without a following wrapup is marked `(finished since the wrapup was written)` and handed back with its detail until the next wrapup folds it in.

With steps on a task, the open steps are the work and the description becomes the constraints the steps are built against. Anything the description asks for that no open step covers is not built; `/rk` flags it and asks for the step.

Only you set a step to **done**. `rekall_step` stops at `claimed`. The navigator's progress count is built on `done`.

A claimed step is reviewed from its detail: **Accept** ticks it to done, **Send back** returns it to open for another pass. The **N awaiting review** count in the pane header jumps to the first one. The step node itself only moves a step forward, so a stray click never walks it back; reopening an accepted step is a separate **Reopen** button that arms before it fires.

## Commits

A task, or one of its steps, keeps a ledger of the commits that built it: the hash, the commit's own subject line as the comment, and the diff it introduced, read from the project's **repo folder** (set on the project's page). The console shows the ledger under the description as a collapsible rail, and under each step's detail; a row opens to its diff, and can be deleted by hand.

**Log commit** on the description, or inside a running or claimed step, logs the tip of the repo. The chevron next to it opens a picker over the last 30 commits, newest first, with their subject and age, so an earlier commit can be logged against the task or step instead; rows already logged there are marked. A hash the log does not reach can be pasted in the same panel, abbreviated or full. A hash that names no commit is refused and nothing is written. Logging the same commit against the same task and step twice is a no-op.

A session does the same over MCP right after `git commit`:

```
rekall_record_commit  anchors="project:vega task:report-builder"                 # the tip
rekall_record_commit  anchors="project:vega task:report-builder" step="3"        # against step 3
rekall_record_commit  anchors="project:vega task:report-builder" commit="a0fd5cc" # an earlier commit
```

A logged commit stays out of `rekall_context` until it is chosen for it. Each row carries an **add to context** toggle (shown on hover; **in context** once on) and the rail fills the node of every chosen commit and counts them. A chosen commit travels with the task's context as a `<commit hash="…" step="…">` element inside `<commits>`: its subject, then its diff (capped at 60,000 characters), oldest logged first. Choosing is console-only; `rekall_record_commit` logs but never chooses.

The REST side is `GET /api/commit-references`, `POST /api/tasks/{id}/commit-references/latest`, `POST /api/tasks/{id}/commit-references` (body `commitHash`, optional `stepId`), `GET /api/tasks/{id}/recent-commits`, `GET /api/commit-references/{id}/diff`, `PATCH /api/commit-references/{id}` (body `inContext`) and `DELETE /api/commit-references/{id}`.

### Commit on claim

A project can commit for the session instead of waiting for it to. The **Commit on claim** switch sits under the **Folder** field on the project page and only arms when that folder is a git repository: the strip under it says which branch is checked out and whom git would commit as there (`user.name` / `user.email` as `git config` resolves them in that folder, so the global identity unless the repo overrides it). Outside a repository the switch stays off and says why; on a repository with no `user.email` it arms but warns that the commit will fail until one is set. The setting is saved on the project (`autoCommit` on `PUT /api/projects/{id}`; the server keeps it off whenever the folder is not a repository) and `GET /api/projects/{id}/repository` is what the strip reads.

With it on, a claim commits: `rekall_step … state="claimed"` stages everything in the folder (`git add -A`), commits it, and logs that commit against the step; on a task with no checklist, the Claude-authored `rekall_wrapup` that claims the task does the same against the task. A task with a checklist never commits on its wrapup. The message is generated from what moved: a Conventional Commits subject (`feat:` by default, `docs:` when every file is documentation, `test:` when every file is a test) carrying the step or task title, then a body naming the anchor, the step number and the files, one per line, twenty at most. A clean tree commits nothing and says so; a git refusal (no identity, a hook, a folder that stopped being a repository) is reported in the tool's answer and the claim stands either way. `rekall_context` marks such a project with an `auto-commit` field telling the session not to `git commit` itself.

## Description review

A task with no checklist walks the same line at task scope: **open**, **running** while a terminal is open on it with no step target, **claimed** when a Claude-authored wrapup lands, **accepted** when you accept it in the console. Nothing new is typed for it: running follows the terminal and claimed follows the wrapup write. The description pane shows the running pill and a review bar: on **running** it carries **Accept** and a link to the wrapup, so a session driven by hand still has a console exit; on **claimed** it adds **Send back** (with an optional note the next session sees); accepting offers to also mark the task done. It arrives on the same `GET /api/steps/stream` connection as a `task-review` frame, the wrapup that claims it rides the same connection as a `wrapup` frame so the pane shows the new text with the claim rather than on the next reload, and `PATCH /api/tasks/{id}/review` is the console-only Accept / Send back. Adding a first step retires the task-level line and the checklist takes over.

## Console

One surface, three panes: pick a task on the left, pick its checklist, its wrapup, a note or a session in the middle, write on the right. The field at the top takes the same grammar as `/rk`.

The description, steps, wrapup and terminal are pinned above the notes. Each opens in the writing pane; a task missing one shows an empty card. `c` opens the terminal pane, the same way `s`, `w` and `d` open steps, wrapup and description. Companies, projects and tasks are created, edited and deleted from one editor, opened on the parent record. Title and label sit together with the anchor assembled live as you type. Deleting states what goes with it.

Finished tasks are folded into a "filed" drawer, closed on every load. Writing autosaves; a note has no Save button.

A note is put on a task from either side. From the note, the task chips on its pane open a picker that walks company, project and task. From the task, the **Notes** button in the description and steps headers drops a list of every note under itself, the ones on this task first: one click, or `↑` `↓` and `↵`, adds a note or takes it off, without leaving the pane. A note whose only task is this one stays put, since a note needs at least one.

The note cards in the task's column take a note off the task too. The right end of a card says where else the note lives; under the pointer, or once the card has focus, it becomes a `×` that takes the note off this task without asking, and `⌫` on a focused card does the same. The toast that follows offers **Undo** for a few seconds, which puts the note back and reopens it if it was the one in the editor. A note that is only on this task shows a lock there instead.

`b` switches the left column between tasks and notes. Browsing notes, picking one leaves the task in view alone: the middle column becomes the note's placements, the tasks it is on grouped by project. **Put it on a task** at the top opens the same company → project → task picker the note's "+ task" chip opens on the tasks side; one click on a row takes the note off that task, and the arrow on a row is the only thing that opens that task on the tasks side. Switching back to tasks brings the task up to date with the note.

A note starts from either side too. Browsing tasks, **New note** (or `n`) writes one on the task in view. Browsing notes, the same button, `n`, or **+ New note** at the top of the list turns the middle column into a composer: a name, and the tasks to put it on, with the task in view ticked to begin with and the live tasks in scope offered underneath. Typing finds any task, `↵` ticks it, `⌘↵` or **Create note** makes the note and opens it in the editor. A note lives on at least one task, so nothing is created until one is ticked; `esc` walks away. **Delete** on the editor header removes the note from every task it is on.

## Report

**Report** shows what went to which client and for how long. The frame is a week or a month, stepped with the arrows either side. One column per day is stacked in each company's colour against a dashed line at eight hours. Below it, a section per company, its projects and its tasks, with the hours and the days each ran on.

Every task row opens on the steps it closed inside the period, oldest first, with the day each was ticked and a count of those still open. A step ticked outside the period is counted but not named. The chips narrow the report to the companies you pick. **Copy as markdown** puts the whole report on the clipboard with every task's anchor and its closed steps.

The screen is built from the sessions the timer recorded. A session counts on the day it started; one still running counts up to now.

## Model

```
Company ──< Project ──< Task >──< Document
                         │       via document_task
                         ├──< TaskStep
                         └──1 Wrapup
```

| Entity | Anchored by | Holds |
|--------|-------------|-------|
| `Company` | `name` | description, its projects |
| `Project` | `label`, unique per company | title, status, description, its tasks |
| `Task` | `label`, unique per project | title, status, markdown description, a standing wrapup directive, its notes, steps, wrapup |
| `Document` | none | title, kind, markdown body, the tasks it is on |
| `TaskStep` | through its task | title, optional detail, state, position |
| `Wrapup` | through its task | markdown body, who wrote it last. One per task |

`label` is what an anchor resolves: lowercase letters, digits, `-`, `_`, `.`, no spaces, normalised on write. `title` is free text and changing it never breaks an anchor. Renaming a label moves the anchor, and the editor says so before saving.

A project belongs to one company, a task to one project. A note belongs to at least one task and often several. Deleting a task unlinks its notes and removes only the ones left on nothing. A wrapup and a step belong to exactly one task and are deleted with it. Adding an entity is a JPA class plus a Liquibase changeset, not a UI action.

## Export

```bash
curl -OJ http://localhost:47355/api/export
```

Or the **Export** button in the top bar. The archive is a folder tree, one folder per company, then project, then task, one markdown file per note, plus a `MANIFEST.md` with statuses and anchors. Nothing reads it back. A note on several tasks appears under each; `MANIFEST.md` lists the copies.

## Develop

```bash
make ui-dev   # Vite dev server on :5173, proxying /api and /mcp to :47355
make test     # backend tests, then frontend lint, types and unit tests
```

`rekall-app/src/main/resources/claude/commands/rk.md` is a symlink to `.claude/commands/rk.md`. Maven copies the content, not the link.

### Frontend stack

| Concern | Choice |
|---|---|
| Framework | Vue 3, Composition API, `<script setup lang="ts">` |
| Shell | One surface, three panes. No router |
| Build | Vite 5 |
| Styling | Tailwind CSS 4, semantic tokens in `src/assets/main.css` |
| Fonts | Fira Sans and Fira Code, bundled |
| State | Pinia setup stores; `console.store` holds the working set |
| HTTP | `ofetch`, timeout, retry on 5xx only, correlation id per request |
| Validation | Zod on every response |
| Markdown | `md-editor-v3` wrapped by `AppMarkdownEditor`, highlight.js passed in as a local instance |
| Tests | Vitest + Vue Test Utils |

```
src/
├── model/        domain types and branded ids
├── api/          http calls and Zod schemas, no state
├── stores/       Pinia
├── composables/  reusable logic without UI
└── components/
    ├── ui/       atomic and presentational
    └── console/  the three panes, the anchor bar, the editors
```

### Debug (IntelliJ IDEA)

The backend uses Lombok. Enable Settings > Build, Execution, Deployment > Compiler > Annotation Processors > **Enable annotation processing**, or the IDE reports missing getters on code that compiles with Maven.

1. Run > Edit Configurations > Add > Remote JVM Debug, host `localhost`, port `5005`
2. Start with the debug port open:

```bash
mvn -pl rekall-app -am spring-boot:run \
  -Dspring-boot.run.jvmArguments="-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005"
```

## Environment variables

All optional; the defaults run against `./data/rekall`.

| Variable | Default | Description |
|----------|---------|-------------|
| `REKALL_DB_URL` | `jdbc:h2:file:./data/rekall;AUTO_SERVER=TRUE;DB_CLOSE_DELAY=-1` | JDBC url |
| `REKALL_DB_USER` | `rekall` | |
| `REKALL_DB_PASSWORD` | `rekall` | |
| `SERVER_PORT` | `47355` | HTTP port for the UI, the API and MCP |

Notes are stored in plain text in the database file. Credentials kept in them are only as protected as the disk is.

## Modules

```
rekall-common/     ConflictException, NotFoundException: the error vocabulary every layer shares
rekall-model/      Company, Project, Task, Document, hosted-session entities and their state rules
rekall-repository/ Spring Data repositories and the Liquibase changelog for their schema
rekall-service/    Business logic: context assembly, the step and review lines, wrapups, time entries
rekall-api/        REST API for the UI, and the step event stream
rekall-mcp/        MCP server, on rekall-service and never rekall-api: one tool reads, two write
rekall-claude/     Claude Code sessions hosted in the app: spawn, stream, reap
rekall-app/        Spring Boot entry point, serves everything
rekall-ui/         Vue 3 + Vite frontend
```

Classes still live under the `dev.rekall.domain.*` packages they had before the split; the module
boundary, not the package name, is what keeps `rekall-repository` off the API's classpath and the
MCP server off the write controllers.

## Run tests

```bash
mvn test                                            # everything, against in-memory H2
cd rekall-ui && pnpm lint && pnpm typecheck && pnpm test
```

`RekallEndToEndTest` drives the real HTTP API and MCP endpoint against the same Liquibase changelogs the application uses, so a migration that disagrees with an entity fails there rather than at startup.

## Design

`docs/DESIGN.md` records the decisions and the reasoning, including the ones that were reversed and why.

`docs/MEMORY.md` is the memory soak of the console: what was measured, how, and why the numbers say there is no leak.

## License

Source-available, not open source. Licensed under the
[PolyForm Internal Use License 1.0.0](LICENSE): you may read the source, run the
software for your own and your company's internal business operations, and change it
for those purposes. You may not distribute it, in original or modified form.

Copyright 2026 Valerio Mario Casale.
