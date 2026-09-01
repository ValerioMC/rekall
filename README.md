# Rekall

Stores the projects and tasks you work on and hands one of them to Claude Code, in full, on a single command.

You keep projects, tasks and their markdown notes in a local app; `/rk project:vega task:report-builder` loads that task, the project it belongs to and every note attached to it. At the end of a session `/rk project:vega task:report-builder wrapup` has Claude record what the implementation now is, so the next session opens on the current state instead of reading the code back.

## Requirements

| Tool  | Version |
|-------|---------|
| Java  | 25      |
| Maven | 3.9+    |
| Node  | 22+     |
| pnpm  | 9+      |

No database server, no Docker, no cluster. The database is an H2 file under `./data`.

## Run

```bash
make run     # compiles the frontend, then starts on http://localhost:47355
```

`run` rebuilds the UI first. The compiled frontend is committed under
`rekall-app/src/main/resources/static`, and a stale bundle fails silently: the application
serves the old screens instead of reporting an error.

```bash
make build   # the above, plus the packaged jar
make ui      # the frontend alone
```

| Service | Address                      |
|---------|------------------------------|
| UI      | `http://localhost:47355`     |
| MCP     | `http://localhost:47355/mcp` |

The port is 47355 and not 8080 on purpose: 8080 is the first port everything else on a
developer machine takes, and the MCP endpoint is a fixed URL registered with Claude Code, so a
clash breaks the registration rather than moving the application somewhere else. `SERVER_PORT`
overrides it for the server, the `.app` launcher and the tests alike.

```bash
make reset   # delete the database file, no undo
make console # H2 shell on the database
```

## The macOS application

```bash
make dmg-native   # the GraalVM binary in the bundle. Needs GraalVM as JAVA_HOME
make dmg-jvm      # the jar plus a bundled Java runtime. Any JDK 25
```

Both write `dist/Rekall-<version>-<flavour>-<arch>.dmg` and then install the bundle they built
into `/Applications` on this machine: the disk image is what travels to another one, and the
machine that ran the build should be running what it built. An existing `/Applications/Rekall.app`
is replaced without asking, and a copy that is running is quit first and started again after.
`REKALL_INSTALL=0 make dmg-jvm` stops at the disk image.

That bundle is the whole of Rekall: no terminal, no `make`, nothing else installed on the
machine. Both targets are additive, and macOS only. Every target above them is the plain build
and keeps working unchanged on Windows and Linux, which is what the rest of the project uses.

| | `dmg-native` | `dmg-jvm` |
|---|---|---|
| Payload | `Contents/Resources/rekall-app` | `Contents/runtime` plus `rekall-app.jar` |
| Disk image | 91 MB | 99 MB |
| First screen | under a second | about three seconds |
| Build | GraalVM, 4 to 8 minutes | any JDK 25, under a minute |

Both also need the Xcode Command Line Tools, for `swiftc`: the bundle's executable is not the
server but `packaging/macos/Launcher.swift`, a window that starts the server underneath itself.
Double clicking opens that window immediately, on a splash screen that is the server booting,
and swaps it for the console the moment port 47355 answers. Quitting sends SIGTERM, so the H2
file is closed properly instead of left behind a lock file.

One thing the window can do that a browser tab cannot: the folder icon in the database field
opens the system folder chooser, and what it returns is the absolute path the server validates.
A page is never handed a real filesystem path, so in a browser that field stays typed, and the
help text under it says which of the two you are looking at.

It is the same port, the same `~/.rekall/config.json` and the same MCP endpoint as `make run`,
not a second installation with a database of its own. When something is already listening on
47355 the app attaches to it rather than starting a second server, which is what makes opening
the app while a terminal instance runs harmless.

The bundle is signed ad hoc, not with a Developer ID. That is enough on the machine that built
it. A disk image that reaches another machine through a browser arrives quarantined and needs
one command before it will open:

```bash
xattr -dr com.apple.quarantine /Applications/Rekall.app
```

The server's own output goes to `~/Library/Logs/Rekall/server.log`, which View > Open Server Log
opens. It is where a window that never gets past the splash screen says why.

## Connect Claude Code

**Settings > Claude Code** does it in one click: it registers the MCP server for every folder and
installs the `/rk` command. The same button repairs a registration that points at the wrong port,
carries an older copy of the command, or is shadowed in one folder by a registration made there
without `--scope user`. The badge above it says which of those it found.

From a terminal instead:

```bash
make mcp-add          # claude mcp add --scope user --transport http rekall http://localhost:47355/mcp
make mcp-check        # verify the endpoint answers, independently of the client
cp .claude/commands/rk.md ~/.claude/commands/rk.md
```

`--scope user` is the part that matters. Without it `claude mcp add` registers the server for the
one directory it was run from, so `/rk` works there and nowhere else.

A session picks up the registration when it starts, so one already open has to be restarted. It
then starts with one line:

```
/rk project:vega task:report-builder-main-workflow
```

### Open a session from the app

**Open in Claude Code**, on a task or on a project, opens a terminal in that project's folder with
`/rk` already running, so the anchor is never copied or typed. Set the folder on the project page,
in the **Folder** field under the description; without one the button says what is missing and
does nothing.

| | |
|---|---|
| Terminal | iTerm2 when it is installed, Terminal.app otherwise |
| Permissions | **Settings > Claude Code** has a switch that adds `--dangerously-skip-permissions`. Off until turned on, and it stays on this machine rather than in the database |

Only inside Rekall.app. A browser tab cannot open a terminal, and an endpoint that let it would be
one any other page open in that browser could call.

## The anchor syntax

An anchor is `entity:value`, where the entity is `company`, `project` or `task` and the value is the record's **label**.

| Form | Meaning |
|------|---------|
| `/rk project:vega task:report-builder` | The project and that task, both in full |
| `/rk project:vega` | The project, plus its tasks as a list of anchors |
| `/rk vega report-builder` | Positional. Works while each term matches exactly one record |
| `/rk task:"report builder"` | Quote a value containing spaces |
| `/rk project:vega task:report-builder wrapup` | Write the task's wrapup instead of loading it |

Every anchor brings back the record, what it references resolved in full **with their notes**, what references it as anchors, and its own markdown. A note can be attached to several tasks, so cluster access is written once and arrives with each task that needs it.

If a bare term matches more than one record the candidates come back and nothing is loaded. Task labels are unique per project, so `project:` is what disambiguates a label two projects share.

Two tools are exposed. `rekall_context` reads; there is no query tool, no get tool and no schema tool, because reaching a record by asking a question in prose costs several turns before any work starts and fails silently when the guess is wrong, so the entry point is an explicit anchor and nothing else.

`rekall_wrapup` is the only thing Claude can write, and all it can do is replace the wrapup of one task.

## The wrapup

A task has at most one wrapup: **what its implementation currently is**. Not a changelog — no "added", no "before and after", nothing dated. It describes the system as it stands, for a reader who was not in the session.

```
/rk project:vega task:report-builder wrapup
```

Claude reads the current one, rewrites it whole and replaces it. It comes back with the task on the next `/rk`, ahead of the notes. You can correct it in the console at any time, and the pane says who wrote what is on screen and how long ago — the next `/rk … wrapup` replaces it either way, and the tool says so when the words it replaced were yours.

It is capped at 20,000 characters against 100,000 for a note. A wrapup that no longer fits on a screen has stopped describing the state and started recording the process.

A wrapup is written from a terminal, into a window that was already open. The window reads everything again when it comes back to the front, so switching from the session to Rekall is what shows it. It skips that while something on screen is waiting to be saved, so a refresh never lands on top of what is being typed.

## The console

One surface, three panes: pick a task on the left, pick its wrapup or one of its notes in the
middle, write on the right. The field at the top is always there and takes the same grammar as
`/rk`.

The description and the wrapup are pinned above the notes rather than filed among them: what
the task is, then where it got to, then the background to both. Each opens in the writing pane
with a markdown editor of its own, and a task missing either shows an empty card, because the
absence is the reason to write one.

A description is the brief the work is measured against: what has to be built, what it has to
satisfy, what is out of scope. It travels into every context that loads the task. It is
markdown at whatever length the work needs, so it is written on the pane rather than in the
field of the create dialog, which only ever holds the first sentence of it.

It is also the instruction. A `/rk` that loads a task with a description summarises what it
found, says in one line what it is about to do, and does it. It asks only where the context does
not settle the question: no description, one that contradicts itself, or work that would
overwrite something nothing keeps a copy of. Anything already written down is not asked about
twice.

Companies, projects and tasks are created, edited and deleted from one editor, opened from the
row of the record itself: the scope picker for companies and projects, the task row or `E` for
tasks. It opens on the parent, because that is the half of the anchor already settled: a task
says which project it lands in and a project which company, and changing it there moves the
record. Title and label sit together underneath, with the anchor assembled live as you type, so
what a record will answer to is visible before it is saved. Deleting always states what goes
with it first.

| Key | Does |
|-----|------|
| `⌘K` | Focus the anchor field |
| `T` | New task, in the project you are scoped to |
| `E` | Edit the task in view |
| `N` | New note on it |
| `D` | Its description: what the work is |
| `W` | Its wrapup: what it currently is |
| `B` | Switch between browsing tasks and browsing notes |
| `J` / `K` | Walk the list |
| `1`–`4` | Set the status of the selected task |
| `⌘↵` | Save the record editor |

Writing autosaves. There is no Save button on a note.

## Report

**Report** answers the question a week ends on: what went to which client, and for how long.

The frame is a week or a month, stepped with the arrows either side of it. Under the total, one
column per day, stacked in each company's colour and measured against a dashed line at eight
hours: which day carried the week, and who it went to, before a single row is read. Then a
section per company, its projects, its tasks, and what each task came to, with the days it ran on
beside it.

The chips narrow it to the companies you pick. No pick means all of them.

**Copy as markdown** puts the whole report on the clipboard with every task's anchor intact, so a
line in an invoice or a status mail is still one `/rk` away from the work behind it.

Nothing on this screen is typed in. It is the sessions the timer already recorded, regrouped,
which is the only reason a report like this is ever true. A session counts on the day it started,
and one still running counts up to now.

## Model

```
Company ──< Project ──< Task >──< Document
                         │       via document_task
                         └──1 Wrapup
```

| Entity | Anchored by | Holds |
|--------|-------------|-------|
| `Company` | `name` | description, its projects |
| `Project` | `label`, unique per company | title, status, description, its company, its tasks |
| `Task` | `label`, unique per project | title, status, description (markdown), its project, its notes, its wrapup |
| `Document` | — | title, kind, markdown body, the tasks it is on |
| `Wrapup` | through its task | markdown body, who wrote it last. One per task, enforced by the database |

A project and a task carry two names, and they are not interchangeable:

| Field | What it is | Rules |
|-------|-----------|-------|
| `label` | What the anchor resolves. `project:vega` is a lookup on this column | Lowercase letters, digits, `-`, `_`, `.`. No spaces. Unique inside its parent. Normalised on write, so `Report Builder` is stored as `report-builder` |
| `title` | What the record is called on screen and in a sentence | Free text. Changing it never breaks an anchor |
| `description` | What it is about, in prose | Free text. Travels into every context that loads the record |

Renaming a label moves the anchor, and the editor says so before it is saved. Nothing stored points at a label, so there is no reference to repair; what breaks is what was written down outside the application.

A project belongs to exactly one company, and a task to exactly one project. A note belongs to **at least one task and often several**: cluster access or a naming convention is written once and arrives with every task that references it. Deleting a task unlinks its notes and removes only the ones left on nothing.

A wrapup is the opposite: exactly one task, always, and it goes when that task goes. That is why it is not a note with a special kind — a note can be attached to three tasks by construction, and "what does this task currently do" has one answer.

Adding an entity is a JPA class plus a Liquibase changeset, not a UI action: the schema is fixed at compile time on purpose.

## Export

```bash
curl -OJ http://localhost:47355/api/export
```

Or the **Export** button in the top bar. The archive is a folder tree, one folder per company, then per project, then per task, one markdown file per note, plus a `MANIFEST.md` with statuses and anchors.

```
Acme/
  vega/
    report-builder/
      WRAPUP.md       <- what the task currently is, if it has said
      CONTEXT.md
      kmaster14.md
    retry-policy/
      kmaster14.md    <- the same note, written under each task it is on
MANIFEST.md
```

A backup and an escape hatch, not a sync: nothing reads it back. A note attached to several
tasks appears under each of them, because a tree cannot say "this file is also over there";
`MANIFEST.md` lists those copies so a reader knows not to edit them apart.

## Develop

```bash
make ui-dev           # Vite dev server on :5173, proxying /api and /mcp to :47355
make test             # backend tests, then frontend lint, types and unit tests
```

`rekall-app/src/main/resources/claude/commands/rk.md` is a symlink to `.claude/commands/rk.md`.
The command this repository uses is the one the application installs, so editing it in one place
is editing it everywhere; Maven copies the content, not the link.

### Frontend stack

| Concern | Choice |
|---|---|
| Framework | Vue 3, Composition API with `<script setup lang="ts">` only |
| Shell | One surface, three panes. No router: what would have been a route is a selection |
| Build | Vite 5 |
| Styling | Tailwind CSS 4, palette as semantic tokens in `src/assets/main.css` |
| State | Pinia setup stores; `console.store` holds the whole working set |
| HTTP | `ofetch`, with timeout, retry on 5xx only and a correlation id per request |
| Validation | Zod on every response; brands are applied by the schema, so an id's kind is decided where it was checked |
| Markdown | `md-editor-v3`, wrapped by `AppMarkdownEditor`; highlight.js is passed in as a local instance so nothing is fetched at runtime |
| Tests | Vitest + Vue Test Utils |

```
src/
├── model/        domain types and branded ids
├── api/          http calls and Zod schemas, no state
├── stores/       Pinia
├── composables/  reusable logic without UI
└── components/
    ├── ui/       atomic and presentational, no store, no API
    └── console/  the three panes, the anchor bar, the wrapup and the record editor
```

There is no `router/` and no `views/`: the console is one surface, and what would have been a
route is a selection in `console.store`.

### Debug (IntelliJ IDEA)

The backend uses Lombok: Settings > Build, Execution, Deployment > Compiler > Annotation Processors > **Enable annotation processing**. Without it the IDE reports missing getters on code that compiles fine with Maven.

1. Run > Edit Configurations > Add > Remote JVM Debug, host `localhost`, port `5005`
2. Start with the debug port open:

```bash
mvn -pl rekall-app -am spring-boot:run \
  -Dspring-boot.run.jvmArguments="-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005"
```

### Debug (VS Code)

Add to `.vscode/launch.json`:

```json
{
  "type": "java",
  "request": "attach",
  "name": "Attach to Rekall",
  "hostName": "localhost",
  "port": 5005
}
```

## Environment variables

All optional: the defaults run the application against `./data/rekall`.

| Variable             | Default | Description |
|----------------------|---------|-------------|
| `REKALL_DB_URL`      | `jdbc:h2:file:./data/rekall;AUTO_SERVER=TRUE;DB_CLOSE_DELAY=-1` | JDBC url |
| `REKALL_DB_USER`     | `rekall` | |
| `REKALL_DB_PASSWORD` | `rekall` | |
| `SERVER_PORT`        | `47355` | HTTP port for the UI, the API and MCP |

Notes are stored in plain text in the database file. If you keep credentials in them, they are as protected as your disk is.

## Modules

```
rekall-domain/    Project, Task, Document, and the context assembly
rekall-api/       REST API for the UI
rekall-mcp/       Read-only MCP server: one tool
rekall-app/       Spring Boot entry point, serves everything
rekall-ui/        Vue 3 + Vite frontend
```

Dependencies run strictly downward. `rekall-mcp` does not depend on `rekall-api`, so no controller and no write service is on its classpath.

## Run tests

```bash
mvn test                          # everything, against an in-memory H2
cd rekall-ui && pnpm lint && pnpm typecheck && pnpm test
```

`RekallEndToEndTest` drives the real HTTP API and the real MCP endpoint: it creates a project and its tasks, attaches one note to several of them, and asserts that one anchored call brings back all of it. It runs against the same Liquibase changelogs the application uses, so a migration that disagrees with an entity fails there rather than at startup.

## Design

`docs/DESIGN.md` records the decisions and the reasoning behind them, including the ones that were reversed and why.
