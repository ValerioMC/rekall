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
make run     # compiles the frontend, then starts on http://localhost:8080
```

`run` rebuilds the UI first. The compiled frontend is committed under
`rekall-app/src/main/resources/static`, and a stale bundle fails silently: the application
serves the old screens instead of reporting an error.

```bash
make build   # the above, plus the packaged jar
make ui      # the frontend alone
```

| Service | Address                     |
|---------|-----------------------------|
| UI      | `http://localhost:8080`     |
| MCP     | `http://localhost:8080/mcp` |

```bash
make reset   # delete the database file, no undo
make console # H2 shell on the database
```

## Connect Claude Code

```bash
make mcp-add          # claude mcp add --transport http rekall http://localhost:8080/mcp
make mcp-check        # verify the endpoint answers, independently of the client
cp .claude/commands/rk.md ~/.claude/commands/rk.md
```

A session then starts with one line:

```
/rk project:vega task:report-builder-main-workflow
```

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

## The console

One surface, three panes: pick a task on the left, pick its wrapup or one of its notes in the
middle, write on the right. The field at the top is always there and takes the same grammar as
`/rk`.

The wrapup is pinned above the notes rather than filed among them, because it is the answer to
the question you arrive with and the notes are the background to it. A task that has none shows
an empty card: the absence is the reason to write one.

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
| `W` | Its wrapup: what it currently is |
| `B` | Switch between browsing tasks and browsing notes |
| `J` / `K` | Walk the list |
| `1`–`4` | Set the status of the selected task |
| `⌘↵` | Save the record editor |

Writing autosaves. There is no Save button on a note.

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
| `Task` | `label`, unique per project | title, status, description, its project, its notes, its wrapup |
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
curl -OJ http://localhost:8080/api/export
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
make ui-dev           # Vite dev server on :5173, proxying /api and /mcp to :8080
make test             # backend tests, then frontend lint, types and unit tests
```

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
| `SERVER_PORT`        | `8080`  | |

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
