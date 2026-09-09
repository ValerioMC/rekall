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

**Open in Claude Code**, on a task or a project, opens a terminal in the project's folder with `/rk` already running. Set the folder in the **Folder** field on the project page. The terminal is iTerm2 when installed, Terminal.app otherwise. A switch in **Settings > Claude Code** adds `--dangerously-skip-permissions`; it is stored on the machine, not in the database. This button works only inside Rekall.app.

## Session in Rekall

**Run here**, next to **Open in Claude Code** on the description and steps panes, or `c` from any task, opens a session inside the app instead of a terminal. It runs `claude` in the project's folder with `--input-format stream-json --output-format stream-json`, sends `/rk <anchors>` first, and stays open on stdin: the pane shows the reply as it arrives and every prompt after the first is another line written to the same process. Works in a plain browser, not only Rekall.app.

A session is tied to one task, and several run at once, one per task. The pane's switcher moves between a task's sessions; **N live elsewhere** jumps to a session on another task. A dock in the bottom-right corner, on every screen while a session is live, lists the running sessions and jumps back to any of them, so leaving the pane or the console does not lose the way back. The transcript is persisted, so it survives a pane swap and a reload; a restart marks every open session ended. The **skip permissions** switch is the same one the terminal button uses, and with it off an in-app session cannot answer a permission prompt, so tool use is denied.

The pane's meta bar shows the model the session is running, read from what `claude` reports on start, and the reasoning-effort level it was started at. **Settings > Claude Code** picks both for a new **Run here** session:

- **Model** (`Account default`, `Sonnet`, `Fable`, `Opus`, `Haiku`): anything but the default adds `--model <alias>` to the command. Each alias is Claude Code's own name for the latest model of that family, so no version is pinned.
- **Reasoning effort** (`Account default`, `Low`, `Medium`, `High`, `Extra-high`, `Max`): anything but the default adds `--effort <level>`. A higher level lets the model think longer on hard problems and spends more; it applies to models that support extended thinking.

Both are stored on the machine, not in the database, and a session keeps what it started with. The terminal path is unaffected.

The top bar carries a usage meter: the current 5-hour session as a ring with its percentage and time to reset, and, on hover, a bar per window including the weekly per-model limits. The figures are the ones Claude Code's own `/usage` shows, read with the OAuth token Claude Code stores (the macOS keychain, else `~/.claude/.credentials.json`). With no token the meter asks you to sign in; when Anthropic cannot be reached it holds the last figures. It refreshes each minute.

| Property | Default | Meaning |
|---|---|---|
| `rekall.claude.cli-path` | search `PATH` and the usual install dirs | Absolute path to `claude`, overriding discovery |
| `rekall.claude.usage-url` | `https://api.anthropic.com/api/oauth/usage` | Where the usage meter reads session and weekly limits |
| `rekall.claude.max-sessions` | `8` | Live sessions allowed at once |
| `rekall.claude.idle-minutes` | `120` | A session untouched this long is closed by the sweep |
| `rekall.claude.sweep-minutes` | `5` | How often the idle sweep runs |

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

There is no query, get or schema tool. `rekall_step` refuses `done`; that state is set by hand in the console.

## Wrapup

A task has at most one wrapup: what its implementation currently is, not a changelog. It names code by the code's own names (class, file, endpoint, table) and does not transcribe field lists or signatures. It is capped at 20,000 characters, against 100,000 for a note.

A quoted term after `wrapup` is a directive on what to write. It can narrow the subject, dictate the wording, or set the language or length.

```
/rk project:vega task:report-builder wrapup "export module only"
```

A wrapup written after a step finishes folds that step's work into the same description. The console counts steps ticked and notes added since the wrapup was last written. You can edit the wrapup in the console; the next `/rk … wrapup` replaces it and the tool reports when it overwrites a hand edit.

## Steps

A step sits on a line: **open**, **running** while a session works it, **claimed** when the session reports it finished, **done** when you accept it. Each step is a title and an optional markdown detail. The pane opens on the first step whose work is not finished.

A session drives its own checklist over `/rk`:

```
/rk project:vega task:report-builder step:3 start   # step 3 -> running
/rk project:vega task:report-builder step:3 done    # step 3 -> claimed
```

The console holds one `text/event-stream` connection (`GET /api/steps/stream`), so a step moved from a terminal or a box ticked in another window animates without a reload.

Claude receives an open or running step with its detail, tagged `(in progress)` or `(claimed, …)`. A finished step arrives as its title alone. A step ticked without a following wrapup is marked `(finished since the wrapup was written)` and handed back with its detail until the next wrapup folds it in.

With steps on a task, the open steps are the work and the description becomes the constraints the steps are built against. Anything the description asks for that no open step covers is not built; `/rk` flags it and asks for the step.

Only you set a step to **done**. `rekall_step` stops at `claimed`. The navigator's progress count is built on `done`.

A claimed step is reviewed from its detail: **Accept** ticks it to done, **Send back** returns it to open for another pass. The **N awaiting review** count in the pane header jumps to the first one. The step node itself only moves a step forward, so a stray click never walks it back; reopening an accepted step is a separate **Reopen** button that arms before it fires.

## Description review

A task with no checklist walks the same line at task scope: **open**, **running** while a hosted session is attached to its anchor with no step target, **claimed** when a Claude-authored wrapup lands, **accepted** when you accept it in the console. Nothing new is typed for it: running follows the session and claimed follows the wrapup write. The description pane shows the running pill and a review bar: on **running** it carries **Accept** and a link to the wrapup, so a session driven by hand still has a console exit; on **claimed** it adds **Send back** (with an optional note the next session sees); accepting offers to also mark the task done. It arrives on the same `GET /api/steps/stream` connection as a `task-review` frame, and `PATCH /api/tasks/{id}/review` is the console-only Accept / Send back. Adding a first step retires the task-level line and the checklist takes over.

## Console

One surface, three panes: pick a task on the left, pick its checklist, its wrapup, a note or a session in the middle, write on the right. The field at the top takes the same grammar as `/rk`.

The description, steps, wrapup and session are pinned above the notes. Each opens in the writing pane; a task missing one shows an empty card. `c` opens the session pane, the same way `s`, `w` and `d` open steps, wrapup and description. Companies, projects and tasks are created, edited and deleted from one editor, opened on the parent record. Title and label sit together with the anchor assembled live as you type. Deleting states what goes with it.

Finished tasks are folded into a "filed" drawer, closed on every load. Writing autosaves; a note has no Save button.

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
| `Task` | `label`, unique per project | title, status, markdown description, its notes, steps, wrapup |
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
rekall-domain/   Project, Task, Document, and the context assembly
rekall-api/      REST API for the UI, and the step event stream
rekall-mcp/      MCP server: one tool reads, two write
rekall-claude/   Claude Code sessions hosted in the app: spawn, stream, reap
rekall-app/      Spring Boot entry point, serves everything
rekall-ui/       Vue 3 + Vite frontend
```

## Run tests

```bash
mvn test                                            # everything, against in-memory H2
cd rekall-ui && pnpm lint && pnpm typecheck && pnpm test
```

`RekallEndToEndTest` drives the real HTTP API and MCP endpoint against the same Liquibase changelogs the application uses, so a migration that disagrees with an entity fails there rather than at startup.

## Design

`docs/DESIGN.md` records the decisions and the reasoning, including the ones that were reversed and why.

## License

Source-available, not open source. Licensed under the
[PolyForm Internal Use License 1.0.0](LICENSE): you may read the source, run the
software for your own and your company's internal business operations, and change it
for those purposes. You may not distribute it, in original or modified form.

Copyright 2026 Valerio Mario Casale.
