# Rekall - Design Document

Status: label, title and description on projects and tasks; notes shared across tasks; one
wrapup per task, written by Claude and corrected by hand; a step checklist a session moves along
and a person ticks done, optionally routed into a flow

Rekall stores the structure and the working context of the projects and tasks you work on, and
hands them to Claude Code over MCP. Claude reads all of it and writes two things: the wrapup of
a task, which is what that task's implementation currently looks like, and the state of a step,
which it can move from open to running to claimed. The last tick, done, is a person's.

---

## 1. Problem

A folder per project, a subfolder per issue, markdown files inside. It works because Claude Code
can read files, but:

- No structure. What a task belongs to exists only as prose repeated in every `CONTEXT.md`.`
- Duplication. Cluster coordinates and credentials are copied across files and drift.
- Loading a task into context means naming the right files by hand, every time.
- Nothing says what the work currently *is*. Every session opens by reading the code back to
  find out what it already does, because the notes record how things were decided and never
  what they became.

Target: type `/rk project:vega task:report-builder-main-workflow` and have the full context
loaded in one call, including every note the task shares with its neighbours.

---

## 2. Decisions

| # | Decision | Rationale |
|---|---|---|
| D1 | Real tables with real foreign keys | A note can never point at a deleted task. On an app whose only job is to be a reliable memory, silent dangling references are the failure mode that matters. |
| D2 | Claude reads everything and writes a few narrow things | `rekall-mcp` depends on `rekall-service` and never on `rekall-api`, so no controller and none of the console's catalog services are on its classpath, and every read runs in a read-only transaction. The exceptions are `rekall_wrapup`, which replaces one task's wrapup; `rekall_step`, which moves a step from `open` to `running` to `claimed`, follows a flow's route on a claim, and cannot reach `done`; `rekall_record_commit`, which logs a commit against a task; `rekall_propose_step`, which can only add a draft a person has to promote; `rekall_propose_flow`, which can only lay drafts and the routes between them onto a task with no steps; `rekall_create_task`, which can only add a `TODO` task whose steps are drafts; `rekall_note`, which can only add a new note to one task; and `rekall_project_section`, which can only add a section to a task's project or rewrite the body of one that already has that title. See §7. |
| D3 | Markdown content lives in the database | One backup target, reachable through MCP, searchable. |
| D4 | One entry point, and it is a slash command | A session begins with `/rk project:vega task:report-builder`, not with a question. Reaching a record through a natural-language query costs several turns and a few thousand tokens before any work starts, and it is the part that fails when the model guesses the wrong entity. An explicit anchor removes both. |
| D5 | The model is fixed at compile time | There is no runtime meta-model and no DDL engine. Company, project, task and document are fixed SeaORM entities; adding a new kind of record is a model and a migration, not a screen. |
| D6 | Modular monolith, single process, embedded database | Single user, localhost. The application has to be reachable with one command or it will not get used. Localhost is enforced, not assumed: `LocalAccessFilter` refuses a peer that is not loopback, a `Host` that is not this machine and an `Origin` from another site, since nothing here authenticates and the API can open a terminal. |
| D7 | What a record is called and what an anchor resolves are two columns | One column serving both jobs would mean a rename breaks anchors written down elsewhere, and makes every name a compromise between readable and typeable. `label` is a slug and is the identity; `title` is prose and is free. See §4. |
| D8 | A task carries one wrapup, and it is a state and not a log | The thing that costs a session its first twenty minutes is reconstructing what the code already does. A note cannot answer that: notes accumulate, and the reader has to synthesise the current state out of them. A wrapup is that synthesis, written once and overwritten thereafter. Its own table, because a document belongs to many tasks by construction and "one answer per task" has to be a constraint rather than a convention. See §4.1. |

Non-goals: multi-user, authentication, remote deployment, vector search.

---

## 3. Module structure

```
rekall/
  rekall-common/     RekallError, Id, Instant: the vocabulary shared by every layer
  rekall-web/        the HTTP vocabulary two routers share: the error body, request extractors, response helpers; knows no service
  rekall-model/      SeaORM entities and their state rules, one folder per domain
  rekall-diagram/    the diagram format: Mermaid sheets, code links, JSON form, validation; depends on nothing of Rekall's
  rekall-repository/ queries, one folder per domain, and the SQLite migrations for their schema
  rekall-service/    context assembly, the step and review lines, flows, wrapups, notes, time entries, token usage
  rekall-api/        Axum REST routes for the UI, one folder per domain: its controller, request shapes and console service
  rekall-mcp/        MCP server: one tool reads, five write (a wrapup, a step's state, a commit log entry, a draft step, a new note); a folder per domain holds its tools
  rekall-claude/     the in-app terminal (portable-pty PTYs over one WebSocket), the Claude Code usage meter, the run queue, the transcript reader, the Claude folder editor
  rekall-platform/   the database registry, backups and restores, the restart handle, the release check, the OS credential store; what the application does around the catalog
  rekall-app/        rekall-server: wires every module, the local-access guard, serves the built frontend, embedded at compile time
  rekall-app/desktop the Tauri desktop app over the same server, in-process
  rekall-ui/         Vue 3 + Vite (built into rekall-ui/dist, embedded by rekall-app)
```

The layers are separate crates in one Cargo workspace so the dependency direction is enforced by
the compiler, not by convention: `rekall-repository` cannot see `rekall-api`, and `rekall-service`
cannot see any route. The crate boundary carries the guarantee.

`rekall-mcp` depends on `rekall-service` and must not depend on `rekall-api`. The two are
independent consumers of the same services, which is what keeps the boundary structural rather
than accidental: `rekall-mcp` can reach `ContextService`, `TaskStepService` and `WrapupService`,
but no route, none of `CompanyService`, `ProjectService`, `TaskService` or `DocumentService` is among its dependencies.

### Layout inside a crate

The crates are the layers; inside one, the code is laid out the way a Java project is.

- **A folder per domain, one public type per file.** A folder is a package, and every layer crate cuts its code the same way (see Domains below): `rekall-service/src/note/`, `rekall-api/src/note/`, `rekall-model/src/note/`, `rekall-repository/src/repository/note/`, `rekall-mcp/src/note/`. A file holds one public type and is named after it: `NoteService` lives in `note/note_service.rs`, `CompanyResponse` in `catalog/company_response.rs`. Inside `rekall-api` a domain folder holds its controllers, request and response shapes, console services and a `routes.rs` that merges its controllers, so `router.rs` only merges the domains.
- **`mod.rs` and `lib.rs` hold no logic.** They declare the files and re-export the public types, so a caller writes `rekall_service::note::NoteService` and never the file's name. Moving a type between files changes no caller.
- **Privacy is per file, so what siblings share is `pub(super)`.** Rust's privacy boundary is the module, which is the file, not the type. A helper two files of one folder use is `pub(super)`, visible to that package and nowhere else: Java's package-private.
- **Private helpers stay beside the one type that uses them.** A small private struct that only one file reads (`ScratchDir` beside `Database`, `Active` beside `Worker`) stays in that file. A private type that carries most of a module's logic gets a file of its own, visible as `pub(super)`: the run queue's `Worker` sits in `run_queue_worker.rs`, beside the `RunQueueRunner` handle that starts it.
- **A module of free functions is one file named for what it does.** `repository/task.rs` is the task repository, `jstr.rs` the string helpers, `usage_ceiling.rs` the ceiling rule: Rust's counterpart of a class of static methods.
- **Tests live under `tests/`, split in two.** `tests/unit/` mirrors `src/`: the tests of `rekall-api/src/filesystem/directory_listing_service.rs` are `rekall-api/tests/unit/filesystem/directory_listing_service_tests.rs`, attached from the implementation with `#[cfg(test)] #[path = "../../tests/unit/filesystem/directory_listing_service_tests.rs"] mod tests;`, so the file is still a child module of the code it tests and reads private items with `use super::*`. `tests/integration/` holds the integration and end-to-end suites, which see only the public API, with their `support/` helpers and fixtures. Cargo discovers only files directly in `tests/`, so it finds neither folder by itself: unit files are compiled through their `#[path]`, and every integration suite is declared as a `[[test]]` in its crate's `Cargo.toml`. A suite missing from that list never runs, and nothing warns about it. `autotests` stays on, since nothing sits in `tests/` itself for it to pick up.

Two things keep their shape on purpose. A SeaORM entity is one file with `Model`, `Relation` and the `Entity` and `ActiveModel` its derive generates: the derive requires them in one module, and the file is the entity, as a JPA entity is one class. The migrations are one file each, starting from `baseline.sql`, the schema as v0.2.x left it; a file once released is frozen history. In the desktop app, the modules holding `#[tauri::command]` functions are `pub`, because `generate_handler!` resolves each command's generated companion by path and a re-export does not carry it.

### Domains

A domain is the unit a change lands in. The same name cuts every layer, so the code of one domain
is found by opening one folder per crate:

| Domain | Holds |
|---|---|
| `catalog` | companies and projects with their logos and icons; in `rekall-api` and the console also the tasks |
| `component`, `section` | a project's components and its titled pages |
| `connection` | the GitLab instances and Kubernetes clusters a company's projects reach, the projects that bind each, and the log of every credential a terminal asked for |
| `step`, `flow` | the checklist, and the routes and visits that turn it into a flow |
| `note` | documents, their attachment to tasks, the scope that decides how they load |
| `wrapup`, `revision`, `review`, `timeentry` | the wrapup, saved states of a task, the review line, logged time |
| `commit`, `diagram`, `token`, `search`, `context` | logged commits, stored diagrams, token usage, search, context assembly and its size |
| `skill`, `tag`, `queue`, `claude` | skill assignments, tags, the run queue, what a session launch needs |

`diagram` also has `rekall-diagram`, the one domain with a crate of its own, because the format
depends on nothing of Rekall's. A domain gets a crate only on that condition.

A domain with a single entity named like itself (`task`, `tag`, `wrapup`, `diagram` in `rekall-model` and `rekall-repository`) stays one flat file, since a folder `task/task.rs` would only repeat the name.

Entity modules keep their flat path at the crate root (`rekall_model::task_step`,
`rekall_repository::repository::task_step`): the folder is where the file lives and the root
re-export is how a caller names it, so a move between folders changes no caller.

**Allowed edges between domains.** `rekall-service/tests/integration/domain_edges.rs` fails on a
domain that uses another one it is not listed for, and on two domains that use each other:

| Domain | May use |
|---|---|
| `claude` | `connection` |
| `context` | `component`, `connection`, `step`, `wrapup` |
| `flow` | `step` |
| `review` | `timeentry` |
| `section` | `note` |
| `step` | `timeentry` |
| `task` | `step` |
| `wrapup` | `review`, `revision` |
| every other | nothing |

`events`, `tx`, `ctx` and `load` are foundations every domain may use. A new edge is a design
decision: change the table and the test together, or change the code.

`rekall-claude` does not depend on `rekall-api`. What the two routers share is in `rekall-web`.

`claude` exists twice on purpose. `rekall-service/src/claude/` reads the entities a launch needs (the
launch target of a task or project, the work left in a task) so `rekall-claude` never touches a
repository; `rekall-claude` owns everything with a process, a file or a socket behind it: the
terminals, the run queue, the usage meter, the transcripts, `~/.claude`, the Claude Code installer.
A rule that needs the database goes in the first, one that needs the machine in the second.

**Events.** A domain announces a change by implementing `StreamEvent` for its own type (a frame
name and a serialisable payload) and publishing it on its transaction. `events` imports no domain, so
adding an event touches the domain only; a subscriber that acts on one kind reads it with
`downcast::<E>()`.

### Stack

| Component | Choice | Note |
|---|---|---|
| Language | Rust (1.85+) | One binary, no runtime to install |
| HTTP | Axum on Tokio | REST, SSE and the terminal's WebSocket on one port |
| Persistence | SeaORM over SQLite | Explicit transactions (`in_write!` / `in_read!`), events published after commit |
| Migrations | SeaORM migrations | One per schema change, applied at startup |
| Database | SQLite, file | Embedded. No server to start |
| MCP server | Hand-rolled | MCP over HTTP is three JSON-RPC methods and one POST endpoint |
| Frontend | Vue 3 + Vite | |
| Document editing | `md-editor-v3` with a CodeMirror live preview | Toolbar over the markdown source, not a WYSIWYG. A WYSIWYG keeps its own tree and regenerates the source on every edit, and that source is what MCP hands to Claude verbatim. The live preview (`rekall-ui/src/common/live-preview/`) only decorates the source: it hides markup off the caret's line and draws tables, code and diagrams as widgets, and a table edit rewrites that cell or that table alone |
| Tests | `cargo test`; end-to-end suites on a real port and a file database | |
| Desktop | Tauri v2 | The same server in-process, a WebView window, three native bridges |

---

## 4. Model

```
Company ──< Project ──< Task >──< Document
                         │      (document_task)
                         ├──< TaskStep ──< StepLink
                         ├──< StepVisit
                         ├──< TokenUsage
                         └──1 Wrapup

Company ──< ExternalConnection ──< ConnectionAccess
                    │
Project ──< ProjectConnection >──┘
```

| Table | Anchored by | Columns of note |
|---|---|---|
| `company` | `name`, unique | `description` |
| `project` | `label`, unique per company | `title`, `status`, `description`, `company_id` |
| `task` | `label`, unique per project | `title`, `status`, `description`, `project_id` |
| `document` | — | `title`, `kind`, `body_markdown` |
| `document_task` | — | `document_id`, `task_id`, `position` |
| `task_step` | through its task | `task_id`, `title`, `body_markdown`, `done`, `done_at`, `position`, `flow_x`/`flow_y` (where it was dragged on the flow canvas, null for the automatic layout), `max_visits` (how many times a flow may enter it, null for no limit) |
| `step_link` | through its task | `task_id`, `from_step_id`, `to_step_id` (null: the route ends the flow), `outcome` unique per `from_step_id`, `position`; deleted with either step |
| `step_visit` | through its task | `task_id`, `step_id`, `ordinal` unique per task, `entered_at`, `left_at`, `outcome`; deleted with its step |
| `wrapup` | through its task | `task_id` unique, `body_markdown`, `written_by` |
| `project_component` | through its project | `project_id`, `name`, `folder`, `description`, `position` |
| `external_connection` | `label`, unique per company | `company_id`, `title`, `kind` (`GITLAB`, `KUBERNETES`), `base_url` (the GitLab instance, or the server read from an imported kubeconfig), `kube_context`, `secret_ref` (the OS keychain account holding the credential, generated, unique), `secret_set_at` (null while no credential is stored). No column holds a credential |
| `project_connection` | through its project | `project_id`, `connection_id` unique together, `target` (GitLab project path or Kubernetes namespace); deleted with either side |
| `connection_access` | through its connection | `connection_id`, `task_id` (set null when the task goes), `terminal_id`, `accessed_at`, `granted`, `reason` (why the request was refused); append-only, never a credential |
| `token_usage` | through its task | `session_id`, `task_id`, `step_id`, the four token counts, `spent_on` |
| `transcript_scan` | `session_id` | `scanned_bytes`, `scanned_at` |
| `task_skill` | through its task | `task_id`, `step_id` (null for the task's own), `skill_name`, `position`; deleted with its task or step |
| `tag_skill` | through its tag | `tag_id`, `skill_name`, `position`; deleted with its tag |
| `skill_placement` | `skill_path` | `group_label`, `placed_at`; no foreign key: the skill is a file under `~/.claude` |

`task.project_id` and `project.company_id` are both `ON DELETE CASCADE`: a project's tasks have
no meaning without it, and neither do a company's projects. The blast radius of deleting a
company is large, which makes it the interface's job to state it before anyone clicks.

### The label and the title are separate columns

`label` and `title` do two different jobs, kept in two columns:

| Column | Job | Constraint |
|---|---|---|
| `label` | What the anchor resolves | Slug: `^[a-z0-9]+([._-][a-z0-9]+)*$`. Unique inside its parent. Normalised on write by `Slug.of`, so `Report Builder` is stored as `report-builder` |
| `title` | What it is called on screen | Free text, `NOT NULL`. Changing it never moves an anchor |
| `description` | What it is about | Markdown, up to 100,000 characters. The brief the work is measured against, handed to Claude in a `<description>` tag on every context that loads the record |

Normalised rather than rejected: what someone types into a label field is a name, and what the
anchor needs is an identifier, and the two differ by punctuation and nothing else. Only a value
with nothing usable left in it is refused, as a 400.

The label is validated with `@Pattern` on the entity as well, so a write that bypasses the
service cannot leave an anchor the grammar cannot carry. Changing a label is allowed and moves
the anchor; nothing stored points at one, so what breaks is outside the application, and saying
so before the save is the interface's job.

Project labels are unique per company rather than globally, for the same reason task labels are
unique per project: two companies routinely have a project called `website`. A bare
`project:website` that matches twice is reported as ambiguous, resolvable with `company:acme`.

A note is attached to **at least one task and possibly many**. Cluster access, a naming
convention and an onboarding step are all notes of that kind.

The lower bound of one is enforced in the service and not in the database, because it is a rule
about what the interface may leave behind rather than about referential integrity: a note on no
task is unreachable, and no screen could show it again. Deleting a task therefore unlinks its
notes and sweeps up only the ones now attached to nothing.

`position` lives on the link, not on the note: the same note can sit first on one task and last
on another without the two orderings fighting.

Task labels are unique per project rather than globally, because two projects routinely have a
task with the same one. A bare `task:setup` that matches in two projects is reported as
ambiguous; `project:beacon task:setup` resolves it.

### 4.1 The wrapup

A wrapup is what a task's implementation looks like **now**. Claude writes one at the end of a
session with `/rk project:vega task:report-builder wrapup`, and reads it back at the start of the
next one, where it arrives with the task like everything else.

It is defined by what it is not. It is not a changelog: no "added", no "before/after", nothing
dated, nothing phrased as a step someone took. A sentence that only makes sense to a reader who
watched the change happen belongs in a note. The reason is the failure it replaces — twenty
minutes at the start of every session spent reading the code back to find out what it already
does, because the notes describe how things were decided and never what they became.

| Decision | Why |
|---|---|
| Its own table, not a `document` with `kind = 'wrapup'` | A document is attached to any number of tasks by construction. That is exactly the shape a wrapup must not have: there is one answer to "what does this task do now", and a row that could hang off three tasks is three answers waiting to disagree |
| `task_id` is `NOT NULL` and unique | "One per task" is then a constraint. A service check reads the table and then writes it, and the two writers this application now has can interleave between those two statements |
| Replaced whole, never appended to, no history | A wrapup that accumulated would become the log it is defined against. The previous version is not kept, which is the cost, and it is why overwriting one that was edited by hand is announced |
| Capped at 20,000 characters, against 100,000 for a note | The cap is where a wrapup that has turned into a log gets caught. If the state of the work does not fit on a screen, what is being written is the process |
| `written_by` is `CLAUDE` or `HAND`, stamped by the transport | The console cannot claim to be Claude and the tool cannot claim to be you: the API stamps `HAND` on anything arriving over HTTP and the tool stamps `CLAUDE` on anything arriving over MCP. A field a client could set is a field that says nothing |
| `ON DELETE CASCADE` on the task, unlike a note | A note may matter to other tasks and is unlinked. A wrapup describes one task and has no meaning anywhere else |

What is **not** enforced is the rule that matters most: that the text describes the state and not
the process. It is not checkable, and a heuristic that rejected the word "added" would be wrong
more often than right. It is stated in the three places it can be read — the tool description, the
slash command, and the empty state in the console — and the cap catches the failure mode it
produces.

A task carries two more columns for the wrapup: `auto_wrapup`, a boolean, and `wrapup_directive`,
an optional short instruction in the console's own words. They are the standing form of the
directive a session would otherwise pass on every `/rk … wrapup`. `Task.configureWrapup` keeps
the directive null whenever the toggle is off, so a stale instruction never rides along after the
intent behind it is gone. `ContextService` renders them as a `wrapup` field on the task only when
the toggle is on, next to `status` and `steps`; a session reading the context sees that a wrapup
is expected without being asked, and the words it should follow. They travel on the same
`TaskRequest` the description does, edited from `WrapupAutomationBar` under both the description
and steps panes so the setting is reachable from whichever surface the work is driven from, and
nothing about them reaches the MCP write path: the toggle is a hint to the reader, not a trigger.

### 4.2 The steps

A task can be broken into steps, and each one is somewhere on a line: `DRAFT` while the
checklist owner is still wording it, `OPEN` once it is promoted and ready to work, `RUNNING`
while a session works on it, `CLAIMED` when that session says it is finished, `DONE` when a
person accepts the work. It exists because of a gap the other two markdown fields on a task leave
between them: the description is the brief and grows as the work is redefined, the wrapup is the
state of the implementation as prose, and neither says which parts are finished. Working that
out meant reading both and comparing them, which is slow by hand and a guess for a model. A row
with a state says it.

| Decision | Why |
|---|---|
| A table rather than checkboxes in the description | The description is one blob that Claude cannot rewrite and that carries no per-item state. A row can be ticked in one click, ordered, and rendered differently depending on where it is |
| `position`, dense from zero, maintained by `TaskStepService` | Every write that could leave a gap renumbers the list. A sparse ordering is correct right up until something reads it as an index. An `@OrderColumn` on the inverse side of the association was the alternative, and it is a column two writers can disagree about |
| `state` is an enum, not a boolean; `running_at`, `claimed_at`, `done_at` follow it in `TaskStep.markState` | The state and its moment are one fact, kept together the way the boolean and `done_at` were. Reopening a step clears all three: a reopened step has no history of being anything else. `completedAt()` is `claimed_at` first, then `done_at`, because a wrapup written right after a claim already accounts for the work and a later console tick does not change what it built |
| Capped at 20,000 characters, like a wrapup | A step whose detail runs past a screen is a task, and the model has a level for that |
| `ON DELETE CASCADE` on the task, like a wrapup | It describes one piece of one task and means nothing beside another |
| A session can write `RUNNING` and `CLAIMED`, never `DONE` | `rekall_step` moves a step as far as claimed, which is what lets a session drive its own checklist. The last tick is a person in the console saying they reviewed the work: a session claiming its own work accepted is what the `CLAIMED`/`DONE` split exists to prevent. The navigator's progress count is built on `DONE` alone, so it still means "accepted" |
| `DRAFT` is the creation default, and only the console leaves it | A new step is a line the checklist owner is still wording, not work: it is kept out of the `<steps>` block a session reads (counted only as `draft="N"` on the tag), `rekall_step` refuses to move it, `markRunning` skips it, and `reviewActive()` still treats a task that holds only drafts as stepless. `TaskStepService.edit` promotes `DRAFT -> OPEN` and sends an untouched `OPEN` step back; a step that has been started, claimed or done cannot return to draft, because the run behind it would be lost. `settleDraftsAtTail` keeps every draft after every non-draft in `position` order, so the numbering a session sees is the numbering the console shows. `stepCount` on `TaskResponse` counts non-draft steps; `draftStepCount` carries the rest |

The rendering is asymmetric, and that is the feature. An open or running step is written into
the context with its detail, because it is the work about to be done or being done now, and its
line is tagged `(in progress)` or `(claimed, waiting for the console to accept it)` so a session
that reloads mid-run does not start it again or redo it. A finished step is written as its title
alone: it needs no doing, and spending the window on the detail of finished work is how a load
costs twice what it is worth. A draft step is left out of the block entirely, present only as
`draft="N"` on the tag and a one-line `drafts` count in the field list. The counts,
including `running` and `awaiting-review` when they are non-zero, go in the field list ahead of
the block.

**The live loop.** `TaskStepService` publishes a `StepStreamEvent` after every write, from the
console or from MCP alike, carrying the affected task's whole checklist. `TaskReviewService`
publishes a `TaskReviewEvent` when a stepless task's review line moves, and `WrapupService` a
`WrapupStreamEvent` when a wrapup is written or deleted (the latter carrying the new
`WrapupView`, or a `deleted` flag). `StepEventStream` in `rekall-api` holds each until the
transaction commits and fans it out to every open console over `GET /api/steps/stream` as
Server-Sent Events under the frame names `steps`, `task-review` and `wrapup`; `useStepStream`
in the UI applies each to the store. A session moves a step to `RUNNING` over MCP and the
console animates the move without a reload: the checklist node breathes, and the branch feeding
it carries a band of light toward it. Opening a terminal on a step ("Run here") is the
other way in: `PtyTerminalManager` moves that step to `RUNNING` on start through
`TaskStepService.markRunning` and back to `OPEN` on close through `releaseRunning`, so the
launch animates whether or not the console is on the checklist pane, and a terminal that dies
without its step being claimed does not strand it. Both are no-ops unless the step is `OPEN`
(start) or still `RUNNING` (end), so an MCP `claimed` or a console tick made meanwhile is never
undone. There is one terminal per task: opening again while one is up is routed to it rather
than spawning another, and if it names a different step the manager releases the old one and
marks the new, so the running marker follows the terminal across a checklist the way it would
in one terminal window. A task with no checklist takes the same path at task scope:
`PtyTerminalManager` calls `TaskReviewService.sessionRunning(taskId, true)` on start and
`(taskId, false)` on close, so the review line runs while the terminal is open on the task and
no step. A wrapup written from a terminal-run session (over MCP) or by an MCP call lands in the
pane with the claim it triggers rather than on the next reload.

### 4.3 The flow

A checklist is an order, and some work is not: build, deploy, test, and a failed test goes to a
fix that goes back to deploy. A **route** (`step_link`) says that when one step ends with an
outcome the work moves to another step of the same task, and a task whose steps carry any route
is a **flow**. A **visit** (`step_visit`) is one entry of the flow into a step, numbered across
the task, closed with the outcome it left by. The visit left open is where the flow stands.

| Decision | Why |
|---|---|
| Routes on the steps, not a second kind of record | A flow is the checklist with its order made explicit. Every rule of §4.2 still holds per step (draft, claim, a person's tick), so a flow needs no new state line and a task can become one by drawing a single route |
| The outcome is a slug, unique per step it leaves | The session names it and the console shows it, so both read `app-error`. One outcome naming two routes would leave the session unable to say which it took. `step_link::normalise_outcome` folds what was typed the way `Slug.of` folds a label |
| A claim follows the route; a person draws it | `FlowService::transition` is the only move that routes, and it runs inside the claim's transaction: refused routing rolls the claim back. Routes are written by `StepLinkService`: the console draws them on any step, and a session can only propose them between drafts (`propose_in`, refusing a released step at either end). `rekall-mcp` reads released routes in the context and follows them, never edits one |
| A flow is planned whole, as drafts | `rekall_propose_flow` hands `FlowProposalService` a `FlowPlan` (drafts in order, routes by position or to End), and `FlowPlan::check` refuses it before anything is written when a step is unreachable from the first, a loop has no way out to End, a step routes to itself, an outcome repeats on a step or a step has more than eight routes. It lands only on a task with no steps, because Start leads to the first step and unrouted steps already there would end the flow. Routes leaving a draft do not make a flow: `StepLinkService::is_flow_in`, the context and `readFlow` count released steps only, and the shelf names each draft's routes (`StepDraftRow`) for review before promotion |
| Arriving reopens finished work | A route back to a claimed or done step means another pass, so `TaskStepService::reopen_for_route_in` returns it to `OPEN`. That is the one way a session's move can undo a tick, and only along a route a person drew. A route into a draft is refused when the flow reaches it |
| One open visit per task | Entering a step leaves the current one without an outcome (`StepVisitService::enter_in`), so "where the work stands" is one row, never a guess between two. A claim on a step never started enters it first, so every pass is counted |
| Visits travel on the step views | `TaskStepView` carries its routes and visits, so the `steps` frame and `/api/steps` already redraw the graph live and the console derives the trail, the counts and the current step itself (`common/flow/step-flow.ts`) |
| Start and End are not records | Start always leads to the first step on the checklist and is drawn by the console alone. End is a route whose `to_step_id` is null (`RouteTarget::End`), so it needs no step and no state: a claim by that route closes the visit and opens none, `FlowRoute::Ended` carries the outcome, and the flow reads as finished because no visit is open. A step with no routes on a flow is drawn joined to End without a stored route |
| Where a step was dragged is stored on the step | `task_step.flow_x` and `flow_y` (`PATCH /api/steps/{id}/flow-place`, `DELETE /api/tasks/{id}/flow-places`) are pure layout: nothing reads them but the canvas, and clearing them hands the steps back to dagre. `FlowPlace` refuses a place beyond 20,000 pixels |
| A flow runs unattended | A claim that follows a route answers with the step to start and says no review is awaited; the wrapup is asked for only when the flow ends. `.claude/commands/rk.md` carries the same rule, so a session never stops between moves to ask for a tick |
| A spent visit budget stops the flow | `StepVisitService::enter_in` is the one place a new visit is made, so it refuses one past `task_step.max_visits` (`admits_visit`). The refusal rolls back the claim that routed there: nothing moves, the session is told to stop and report. Re-entering the step the flow already stands on is not a visit. Raising the budget (`TaskStepService::limit_visits`, `VisitBudget` 1 to 99) or a reset lets it go on |
| Reset forgets the visits, not the states | The states are a person's review; the trail is a count. `FlowService::reset` clears one, never the other |

A claim chooses its route by one rule (`flow::choose_route`): the outcome it names; with no
outcome, the step's only route, or the end of the flow when it has none; several routes and no
outcome is refused with their names, because only the session knows what happened. An outcome on
a plain checklist, or on a move that is not a claim, is refused rather than ignored.

The context marks a flow as `flow="true"`, `current` and `moves` on `<steps>`, `(current, visit N)`
on the step, a `Routes:` line under every step (finished ones too, since a route can return to
them) and a `flow` field naming the step to work. The console draws it in `StepFlowView`: dagre
lays the steps out top to bottom in checklist order (`common/flow/flow-layout.ts`, with unseen
edges keeping the spine), walked routes solid and the rest dotted, the route the last move came
in by animated toward the lit current step, the trail of moves above it. Start and End are drawn
at both ends, a dragged step overrides its dagre place (`placeSteps` redraws only the routes that
touch it between the boxes), and `composables/useFlowDrag.ts` owns the pointer work: moving a
step, and drawing a route from a step's handle onto another step or End.

---

## 5. Loading a context

`ContextService` is the whole of what Rekall does at read time. It resolves each anchor and
walks the associations, inside one read-only transaction, returning a materialised
`ContextRecord` tree. The tree is materialised so the MCP layer can render it without an open
session: handing entities to a renderer instead would make every field access a bet on whether
the session is still open.

The walk distinguishes by direction, not by depth:

| Direction | What is loaded | Why |
|---|---|---|
| Forward (`@ManyToOne`) | The record in full, **with its documents** | What this record depends on to be understood. Bounded by the model: a task reaches its project and stops |
| Inverse (`@OneToMany`) | Labels only, printed as anchors | What points back at this. Unbounded fan-out: a project has forty tasks |
| One-to-one (`wrapup`) | In full, in a tag of its own, ahead of the notes | Exactly one, so there is no fan-out to bound. Ahead of the notes because a session that opens on a task is asking what it does now, and the notes are the background to that answer |
| Owned collection (`task_step`) | Open and running ones in full, finished ones as a title | Bounded by the task and ordered, so it is not fan-out. Rendered between the description and the wrapup, which is the order the three are asked in: what is this, what is left, what did it become |
| Assigned skills (`task_skill`, `tag_skill`) | A `<skills>` block after the description: the task's, its tag's, then those of the steps still to be worked, each with its scope | Short (20 per owner) and invoked by name, so only the name travels |

Skills follow three rules. A tag's skills are read when the context loads and never copied onto its
tasks, so editing a tag changes every task under it. A skill already listed at a wider scope is not
listed again, and a finished step's skills are dropped. The name is checked for its shape only
(`rekall_model::skill_name`): `rekall-api` cannot see `~/.claude`, which `rekall-claude` reads, so the
picker offers only what exists and a skill that later vanishes stays assigned and is flagged in the
console. The writes are `SkillAssignmentService` in `rekall-api`, each replacing one owner's whole
list; `rekall-mcp` can read the skills through the context and never write them.

A project's connections arrive as a `<connections>` block after its components, one line per
binding: the label, the title and how to reach it. A cluster is `kubectl --context <label>` with the
binding's namespace; a GitLab instance is its URL and project path, pushed to over HTTPS through the
configured credential helper and called through `rekall-gitlab --connection <label>`. A connection with no credential
says so, so the session reports it. `ConnectionView` carries no field that could hold a credential,
the keychain account, a cluster's server or its CA, and the block says these work only in a terminal
Rekall opened and forbids handling a credential directly; `/rk` repeats that rule.

Scalars are rendered as bullets and bodies as tags, which is why a description is a `<description>` block rather than a `description` bullet. It is a document, with headings, lists and a scope section, and a document inlined into a list item stops being one: every line after the first falls outside the bullet, and its own headings outrank the record's.

That distinction is why the two-anchor form exists. `/rk project:vega` loads the project and
lists its tasks as anchors; naming the task as a second anchor is how you narrow to one.

A note arriving with every task that references it is the point of the whole design. One row,
many links, is what lets a cluster note reach every task it belongs to without going stale in
any of its copies.

---

## 6. MCP server

Transport: HTTP on the same process as the UI.

```bash
claude mcp add --transport http rekall http://localhost:47355/mcp
```

Six tools. `rekall_context` reads, taking one string:

```json
{ "anchors": "project:vega task:report-builder-main-workflow" }
```

`rekall_wrapup` writes, taking the same anchors and the text:

```json
{ "anchors": "project:vega task:report-builder", "body": "## What it does\n..." }
```

`rekall_step` moves one step of a task along its line, taking the anchors, the step's number or
title, and the target state:

```json
{ "anchors": "project:vega task:report-builder", "step": "3", "state": "running" }
```

`state` is `running`, `claimed` or `open`. It refuses `done`, and refuses a step a person has
already accepted: reopening one is a console decision.

`rekall_record_commit` logs the tip of the project's repo folder, or an earlier hash, against a
task or one of its steps. On a project set to auto-commit, `rekall_step` with `claimed` (and
`rekall_wrapup` on a task with no checklist) commits the folder itself; both take an optional
`commit_message` the session writes, a Conventional Commits subject and a few sentences of body,
and without one `CommitMessageGenerator` draws the message from the step's detail or the wrapup.
No commit message lists files.

`rekall_propose_step` adds one step as a draft, which is what `/rk … plan` calls once per step.
A draft is not work: it stays off the checklist a session reads until a person promotes it, a
title the task already has is refused so a second plan does not double the shelf, and a task
holds at most twenty drafts.

`rekall_propose_flow` is the same gate for a task whose work branches or loops: `/rk … plan` calls it
once, with every step and its routes, when a step's outcome decides what comes next. The plan is
checked whole and written in one transaction, so a refused plan leaves nothing on the shelf (§4.3).

`rekall_create_task` creates a task in the project its anchor names, with an optional description and
up to twenty steps, in one transaction. `TaskAuthoringService` in rekall-service is its only path: the
console's `TaskService` in rekall-api stays the one that edits, reviews and deletes. The task starts
`TODO` and each step is a draft, so the same promotion gate as `rekall_propose_step` applies. A label
the project already uses is refused rather than suffixed, so an anchor never changes under the session.
It publishes `task-created` on the event stream and the console reloads its task list.

`rekall_read_sections` is the read side of `rekall_project_section`: sections are in no context, so
this is the only way a session sees them. `ProjectSectionService::read` lists the titles, or returns one
section by title ignoring case; a title the project lacks is refused with the titles it has.

`rekall_note` writes a new note onto one task, taking the anchors, a `title` and the `body`.
It is what a session uses for output worth keeping that is not the state of the implementation:
what a step asks to keep, or the note a task exists to write. `/rk … note "…"` asks for one
explicitly. It creates, and does nothing else: a title the task's notes already carry is refused
rather than overwritten, and the note is `FULL` with kind `notes` until a person changes it.

`rekall_project_section` is `rekall_note` for a project: the anchors name a task, the section lands on that
task's project, and `replace` rewrites the body of the section with the same title (ignoring case) instead
of refusing. Sections live in their own table, `project_section`, not in `document`, because a note is
reached through tasks and loads into a context and a section does neither. Renaming, reordering and
deleting are the console's, in `ProjectSectionCatalogService` in rekall-api.

A project's components are the folders of it that are parts of it, one row each in `project_component`
(`name`, `folder`, `description`, `position`; the name is unique per project ignoring case, and starts as
the folder's last segment when none is given). Unlike a section, they are context: `ContextService`
reads them with the project and `ContextRenderer.render_components` writes a `<components>` block, so the
project's own anchor and every task of it carry them once, and `ContextSizeService` counts them as their
own part. They are read-only to a session, no MCP tool writes them; the console's
`ProjectComponentCatalogService` in rekall-api lists, adds, edits, orders and deletes them.

A task's description and its step details can name a component as `[[Name]]`. The stored text never
changes: `ComponentReferences` (rekall-service, `component/`) expands each reference while the context
is rendered, from the components `ContextService` puts on the task record as `component_lookup`
(resolution only, the `<components>` block stays on the project). The name matches ignoring case and
surrounding spaces. The first reference to a component in a rendered task becomes its name, folder and
description (cut at `COMPONENT_REFERENCE_MAX`), later ones its name and folder; code spans and fenced
blocks are left as written; a name that matches no component of the task's project becomes
`[[Name]] (component not found in this project)`. `ContextSizeService` measures the expanded text.

The console draws the same syntax in read mode. `markComponentReferences`
(`rekall-ui/src/common/markdown/component-reference.ts`, registered in `common/config/markdown-editor.ts`)
turns `[[Name]]` into a chip that knows only the written name, with the backend's edge cases; markdown-it
never offers it code spans or fences. The project is not known to the global renderer, so
`AppMarkdownEditor` takes the project's `components` and `useComponentReferenceChips` settles each chip
(found with a folder and description title, missing, or plain for text with no project) after every render
and whenever the list changes. `useSelectedProjectComponents` loads the selected task's project into
`useProjectComponentsStore` and answers `null` until the list has arrived, so nothing is called broken
early. Write mode stays plain text, apart from the completion: `componentCompletion`
(`common/markdown/component-completion.ts`) is a CodeMirror completion source handed to `MdEditor` through
its `completions` prop. It matches `[[` plus what was typed, filters the project's components itself
(`filter: false`, name contains the text), skips code and fences through the syntax tree, and applies
`Name]]` through `view.dispatch`, replacing a closing `]]` already after the caret. `common/config/markdown-editor.ts`
adds `Tab` as `acceptCompletion`, which does nothing when no list is open.

A note can travel by reference. It then arrives as its title, its first line and an anchor such
as `note:3f2a9c1e` (the first eight characters of its id), with `loaded="on request"`, and
`rekall_context` loads it in full by that anchor. `ContextRenderer`, which writes every context,
is shared with `ContextSizeService`, so the size the console shows for a task is the length of
what a session receives.

An anchor is `entity:value`. A value containing spaces is quoted. A bare term with no `entity:`
is looked up across both entities and accepted only when exactly one record matches; on more
than one the candidates come back and nothing is loaded. The tool does not score, weigh or
choose.

A note attached to several tasks arrives under each of their anchors. The same markdown showing
up twice in one session is the relation working, not a duplicate.

The two writes are narrower on purpose. `rekall_context` may be handed a company and will
happily return everything under it; a write cannot, because a project names forty tasks and
none of them is the answer. Anything that does not resolve to exactly one task is refused with
the form that would have worked, and an ambiguous bare label is reported rather than picked. The
`rekall_wrapup` description carries the state-not-process rule, because that description is the
only part of this system the model reads before deciding what to write; the `rekall_step`
description says the last tick is not its to make.

Responses are capped, with an explicit truncation notice rather than a silent cut.

### Two protocol eras

MCP has two revisions in play, split at `2026-07-28`. Up to `2025-11-25` a client opens with an
`initialize` handshake and the agreed revision holds for the session. From `2026-07-28` there is
no handshake and no session: every request carries its own revision, in the
`MCP-Protocol-Version` header and again in `params._meta`, along with its method in `Mcp-Method`
and, for a `tools/call`, its tool name in `Mcp-Name`.


| | Handshake era | Stateless era |
|---|---|---|
| Opens with | `initialize`, answered with the revision asked for | nothing |
| Unknown method | `-32601` on a 200 | `-32601` on a 404, so a probe can tell this from a wrong address |
| Unknown revision | — | `-32022` on a 400, listing what is supported |
| `server/discover` | answered | answered, and mandatory |
| Result envelope | a result is a result | every result carries `resultType`, and one without it is discarded |
| `tools/list` | the tools | the tools plus `ttlMs` and `cacheScope`, both required |

The last two rows are the ones that fail silently, and they fail in the same place. A `tools/list`
missing `resultType`, or missing either cache annotation, is not repaired or read leniently: it is
thrown away whole, and a client that cannot read the tool list registers no tools. The server
shows as connected, `/rk` is there, and there is nothing behind it to call. `server/discover` is
the exception that hides this, because its own schema defaults both annotations.


---

## 7. Write safety

`rekall-mcp` has no controller, no `CompanyService`, `ProjectService` or `TaskService`, no `DocumentService` and no
`RevisionRestoreService` on its classpath: they live in `rekall-api` for exactly that reason, and
anything that writes the catalog belongs there. The write services it can reach are narrow by
construction. `WrapupService` takes a task and a body and can do nothing else: no record created,
renamed or deleted, no note touched. `TaskStepService.transition` takes a task, a step and a
target state, and refuses `DONE` and any step a person has already accepted; it cannot add,
reorder or remove a step, and the console's `edit` is still the only path to `DONE`.
`TaskStepService.propose` can only append a draft, which no session reads until a person promotes
it. `FlowProposalService.propose` can only lay drafts onto a task with no steps, and the routes it
asks `StepLinkService.propose_in` for are refused unless both ends are drafts, so no released
step's routes change. `NoteService` (rekall-service) can only create a note and attach it to the one task it was
written for: it never loads a note by id, so it cannot edit, detach or delete one, and
`DocumentService`, which can, stays in `rekall-api`. `CommitReferenceService` and `AutoCommitService` log and make commits in the project's own
folder. Every read runs in a
read-only transaction. `McpTool.writes()` is declared
rather than inferred, and the startup log names the write surface out loud.

Connection credentials never reach `rekall-mcp`. A GitLab token or a Kubernetes kubeconfig lives in
the OS credential store (`KeychainSecretStore` in `rekall-platform`, over `keyring`, service
`Rekall-connections`, one account per connection's `secret_ref`), behind the `SecretStore` trait in
`rekall-service/src/connection/`. `rekall-app` builds the store and hands it to `ApiState` and
`ClaudeState` only: it is not part of `Services`, which is what the MCP tools are built from, and
`rekall-mcp/tests/integration/credential_isolation.rs` fails if `rekall-mcp` or `Services` ever names
it or a type that carries a credential. A credential in memory is a `Secret`: it prints as `***`, has
no `Serialize` and is wiped on drop. A kubeconfig is parsed by `Kubeconfig`, which refuses a user
that runs a command (`exec`, `auth-provider`), a username and password, a server that is not
`https://` or skips TLS verification, and a file path that is relative or unreadable, and whose
parse errors name a line, never the text. The chosen context is kept as `KubeAccess::to_kubeconfig`:
that context alone with every referenced file inlined. On Linux the store is the kernel keyring,
which forgets its entries when the session ends.

The console's write path for connections is `rekall-api/src/connection/`, one service per entity:
`ConnectionService` (a company's connections; deleting one removes its credential from the store
after the row is gone, logging a failure), `ConnectionCredentialService` (the only code that stores,
replaces or clears a credential), `ProjectConnectionService` (bindings; a project binds only a
connection of its own company), `ConnectionAccessQueryService` (the access log, newest first) and
`KubeconfigPreviewService` (a kubeconfig's contexts with why each can or cannot be imported, storing
nothing). Credentials are write-only: `PUT /api/connections/{id}/credential` takes `token` (GitLab)
or `kubeconfig` and `context` (Kubernetes) and answers the connection with `credentialSetAt`, and no
endpoint returns a credential. The store is written before the row, so a failed write leaves the row
saying no credential is set; clearing empties the store before the row, so the row never claims less
than the store holds. A Kubernetes connection takes its server and context from the imported
kubeconfig and refuses a typed URL; a connection's kind is fixed once created. The routes are
`GET`/`POST /api/companies/{id}/connections`, `GET`/`PUT`/`DELETE /api/connections/{id}` (`GET` lists
the projects that bind it, `DELETE` answers `unboundProjects`), `PUT`/`DELETE
/api/connections/{id}/credential`, `POST /api/connections/kubeconfig/contexts`, `GET
/api/connections/{id}/access?offset=&limit=`, `GET /api/projects/{id}/connections` and `PUT`/`DELETE
/api/projects/{id}/connections/{connectionId}` (`PUT` binds, or sets the `target` of a binding the
project has). Every change publishes a `connection` frame naming the company.

`POST /api/connections/{id}/check` asks the connection's server whether its stored credential still
works, before a session finds out mid-deploy. It lives in `rekall-platform/src/connection/` because
it is HTTP to the outside: `ConnectionCheckService` reads the credential through the store and hands
it to a `ConnectionProbe` (`HttpConnectionProbe`, or a fake through `StartOptions::connection_probe`).
GitLab is asked `GET /api/v4/user` with the token; a cluster is asked `/version` and a
`SelfSubjectReview`, trusted through the kubeconfig's CA and with a client certificate as the TLS
identity, falling back to an authenticated `GET /api` on a cluster too old for the review. The
answer (`ok`, a sentence naming the identity, the rejection status or why the server could not be
asked) is shown and never stored, and a connection with no credential is answered without asking
anyone.

A terminal with a credential session also starts `claude` with `--settings` pointing at the
session's `claude-settings.json`, whose `permissions.deny` (`rekall-claude/src/connection/guard_rails.rs`)
closes the session folder to the file tools (`Read`/`Edit` with an absolute `//` rule on the folder's
real path) and denies dumping the environment (`env`, `printenv`, `export -p`, `set`), `git
credential`, `kubectl config view --raw`, running the plugin or helper by hand, `cat`/`head`/`tail`/
`less`/`more` on the folder, `curl`/`wget` to the credentials endpoint, and the `security` keychain
commands. They add to the person's own settings rather than replacing them, and they hold under
`--dangerously-skip-permissions`: checked against Claude Code 2.1.306, where a `--settings` deny on
`printenv` was refused in bypass mode while the same call ran without the rule, and an absolute
`Read(//<folder>/**)` rule was refused too. They are prefix matches against an accidental read, not
a sandbox: a command shaped differently (`kubectl config view --minify --raw`, a script that calls
the endpoint) is not caught, and a session set on extracting a credential can still read
`REKALL_CREDENTIAL_TOKEN` through a command the rules do not name.

The residual risk is real and small: Claude can overwrite one task's wrapup with something
wrong, or with something that replaces a correction made by hand, and it can mark a step running
or claimed when it is not. The first two are repaired from the wrapup's history in the console:
`TaskRevisionService` keeps what every write, delete and restore replaces (a hand edit typed over
hand-written text is kept once per ten minutes, so the history is versions rather than
keystrokes), and the tool's answer still announces a replaced hand edit, because no session reads
that history. The third a person corrects with one tick in the console. It can also add a note
nobody wanted, which a person deletes from the note's pane. What it cannot do is tick a step
done, change or lose a note, move a task or delete anything.

---


### Opening a session from a button

**Open in terminal**, on a task, hands the session to your own terminal app in that
project's folder with `/rk` already running. It answers the last thing the anchor chips could not: an anchor still has
to be pasted somewhere, and that somewhere has to be the right directory, because Claude Code
takes the folder it was launched from and keeps it for the session. So the folder is a column on
the project, `repo_folder`, and it travels down onto every task response beside the project label
those rows already carry.

It is a bridge in the desktop app (`rekall-app/desktop/src/bridges/claude_code_launcher.rs`), next to the
folder chooser, and not an endpoint. A `POST /api/launch` on 47355 would be reachable by any page
open in any browser on the machine, which makes a button that starts a terminal into a way to
start one without a click. Through the WebView, only this application can call it.

The page never names a command either. It sends a folder, an anchor and one flag; the folder has
to exist, the anchor has to be `entity:value` characters and nothing a shell reads as a command,
and the line is assembled on the native side. A note rendered in that window is markdown someone
else may have written, and it is one XSS away from being the caller.

The terminal is launched by opening a short script with it rather than by scripting the terminal
itself. AppleEvents would put a "Rekall wants to control Terminal" prompt in front of a button
whose whole point is that it is one click, and would fail silently for anyone who declines. The
script removes itself, `cd`s and `exec`s, so nothing is left behind and closing the window ends
the session and nothing else.

`--dangerously-skip-permissions` is a switch in Settings, off until it is turned on, and it is
kept in the browser's storage rather than in the database: "run without asking" is a property of
this terminal on this machine, and a database opened somewhere else has no business carrying that
answer along with it.

---

### The terminal pane

The in-app session is a real terminal, `C` in the console. `PtyTerminalManager` in
`rekall-claude` starts the interactive `claude` TUI in a pseudo-terminal (`portable-pty`) in the task's
folder, with `/rk` as the first line (`/rk … plan` for a terminal opened in `TerminalMode.PLAN`,
which the Steps pane's **Plan here** asks for; `/rk … generate` and `/rk … explain` for the Diagrams
screen's **Generate** and **Explain code**, the only modes a terminal on a project alone takes), and pumps its raw bytes to whoever is watching. The reason
it is a PTY and not `claude -p` is token cost: a hand-run `claude` keeps its own cache warm,
compacts its own context and shows its own `/context` and cost read-outs, and a PTY running the
same binary the same way inherits all of that, where a headless stream-json process reassembled
into a transcript could not. It is also the only place an interactive permission prompt actually
renders and can be answered.

The bytes do not go over HTTP. `TerminalController` only opens (`POST
/api/tasks/{id}/terminals`), lists and closes; once a terminal exists the pane connects to
`TerminalSocketHandler` at `/api/terminal/{id}/io`, Rekall's one WebSocket. Binary frames are
stdin and stdout, a `{"resize":[cols,rows]}` text frame sets the window size, and a
`{"type":"ended",...}` frame closes it out. Sends go through one bounded
queue per socket, so a pane that stops reading is dropped rather than left to back up memory. `useTerminalSocket` is the client half,
`TerminalPane.vue` wires it to an `xterm.js` instance.

One terminal per task. Opening again on a task that already has one is routed to it; opening on
a different step releases the step it was on and marks the new (see §4). A plan open is always on
the task, never a step, and on a task that already has a terminal it types the plan line into it
rather than spawning a second. `PtyTerminalManager`
also carries the checklist marker: `TaskStepService.markRunning` / `releaseRunning` for a step,
`TaskReviewService.sessionRunning` for a stepless task, on open and on every way the terminal
ends.

Nothing is persisted. There is no row behind a terminal, so `TerminalLaunchService` (in
`rekall-service`, against the entities) only resolves a task id to its folder and anchors, and a
`PtyTerminalManager` restart starts empty and releases any step it left `RUNNING` on the way
down. A bounded in-memory scrollback ring is replayed to a pane that reopens, so it repaints;
that snapshot can begin mid escape-sequence and flicker once, which is the accepted cost of not
storing a terminal. A cap on how many run at once (`rekall.terminal.max-sessions`, eight), an
idle sweep, and a shutdown hook that kills the rest inside the 5s shutdown budget keep strays
from piling up.

The PTY's environment is the user's login shell's, not the app's: launched from Finder, the app
inherits launchd's bare `PATH`, which lacks everything a shell profile adds (`~/.cargo/bin` and the like). `LoginShellEnvironment`
runs `$SHELL -i -l -c` with stdin on `/dev/null`, has it print `env -0` between two random markers
(so a profile's greeting cannot corrupt it), and caches the result; each read starts a background
refresh. Failure, no markers or the timeout keep the last good copy, falling back to the process
environment. `ClaudeCli` adds `HOME` and the usual install directories to it, strips
`CLAUDECODE`/`CLAUDE_CODE_ENTRYPOINT` so a Rekall started from a Claude session can still open one,
and searches that `PATH` for `claude`. Windows skips the shell and uses the process environment.

---

**Credential sessions.** A terminal opened on a project that binds a connection with a stored
credential gets a credential session from `CredentialBroker` (`rekall-claude/src/connection/`), so
git and kubectl can use the credential without it entering the session. `TerminalLaunchService`
hands over the project's bindings (`TerminalConnection`: label, kind, target, instance URL, keychain
account; never the credential). The broker writes a folder under `rekall.home/terminal-credentials/`
(0700, files 0600, scripts 0700; emptied at every start), holding:

| File | What it does |
|---|---|
| `kubeconfig` | One context per cluster, named by its label: the server, CA and TLS name read from the stored kubeconfig, the binding's namespace, and a user that is the exec plugin (`interactiveMode: Never`) |
| `rekall-kube-credential` | kubectl's exec plugin: prints the `ExecCredential` (a token, or a client certificate and key as PEM) |
| `rekall-git-credential` | git's credential helper: answers `get` with `username=oauth2` and the token |
| `rekall-gitlab` | `rekall-gitlab --connection <label> <METHOD> <path> [curl options]`: a GitLab API call with the `PRIVATE-TOKEN` header, `:project` standing for the binding's project path, URL-encoded |

None holds a credential. `SessionEnvironment` adds the folder first on `PATH`, `KUBECONFIG` (only
Rekall's clusters, so the session uses the credential you provided), a `GIT_CONFIG_KEY_n`/`VALUE_n`
pair per GitLab host (an empty `credential.<url>.helper` that drops helpers configured elsewhere,
then Rekall's, after any entries the shell already set) and `REKALL_CREDENTIAL_TOKEN`, 244 random bits
for this terminal. The helpers send the token as a header read from stdin, so it never appears in a
process listing, to `GET /api/terminal/{id}/credentials/{label}?for=kubectl|git|gitlab`. That route
sits outside the router's UUID check, answers only a loopback peer even with
`rekall.security.remote-access` on, and answers in plain text the helpers print as they are: 403
for a token that is not the terminal's (compared in constant time) or a peer that is not this
machine, 404 for an ended terminal, an unbound label or a missing credential, 400 for a shape the
connection's kind does not have. Every request about a bound connection is a line in
`connection_access` through `ConnectionAccessService` (granted, or refused with why); a credential is
handed out only once its line is written. A request from an ended terminal or for an unbound label
names no connection to log against and is only logged by `tracing`. The session ends with the
terminal on every path (`close`, exit, the idle sweep, shutdown): the token stops working and the
folder goes. A cluster whose stored kubeconfig no longer reads is left out of the session and
logged. The helpers are shell scripts using `curl`, so a Windows terminal gets no session, and the
external **Open in terminal** gets none either.

### The Claude folder

The Claude screen edits the files Claude Code itself loads: `~/.claude/CLAUDE.md`, skills, commands,
agents and each project's memories. It lives in `rekall-claude` (`resources/`). The files have no
table: the disk is the record, a session may rewrite a file at any moment, and a copy in a database
would only be a second answer that drifts. The one table, `skill_placement`, holds how the person
groups their own skills, which is Rekall's own preference and no part of the files. The module is `rekall-claude` and not `rekall-api` because it
belongs with the other pieces that serve Claude Code's own state (the usage meter, the transcripts),
and it sits beside the console's write services, out of `rekall-mcp`'s reach.

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

---

### Token usage

A task and each of its steps carry the tokens Claude Code generated on them, and a project the sum
of its tasks. Nothing a session sends says what it cost, and the in-app terminal is a TUI, not a
stream-json process, so the figures are read from where Claude Code already writes them: the
session transcripts under `~/.claude/projects`, where every model call carries its `usage`.

| Piece | What it decides |
|---|---|
| `TranscriptFolder` (rekall-claude) | Which transcripts belong to a project: the folder Claude Code names after the project's `repo_folder` (every non-alphanumeric character becomes `-`), and the folders of subfolders under it. A session's subagents are in `<session>/subagents/` |
| `SessionLedger` (rekall-claude) | What each call is charged to, and on which UTC day, by the line's own timestamp; a line with none takes the day it is read. `/rk` or a `rekall_context` naming one task moves the session to that task; `rekall_step` `running` to the step, `claimed` or `open` back to the task, after charging the call that moved it. A task named again without a step keeps the running step, so a wrapup written mid-step costs the step. Subagent calls go to what the session was on at their timestamp. A message spread over several lines is charged once, by its id. Calls before any Rekall work are not counted |
| `TranscriptScanner` (rekall-claude) | Reads, every `rekall.claude.transcript-scan-seconds`, each session whose files have grown since `transcript_scan` last saw them; an unchanged transcript is never read twice. A line that does not parse is skipped, and a field in a shape the reader does not know reads as empty rather than losing the line's tokens |
| `TokenUsageService` (rekall-service) | Resolves the anchors and replaces the session's `token_usage` rows whole, so a reread never double counts. A task that no longer resolves is dropped; a step that does not resolve leaves its tokens on the task, and deleting a step sets its rows' `step_id` to null for the same reason. A row is a session's spend on one task or step on one day (`spent_on`, midnight UTC), so a task's spend can be told day by day; migration `041` stamps old rows with the day they were read and clears `transcript_scan` so every transcript is read again with real days. A read that changes a task's tallies or days publishes a `token-usage` frame with all of them |

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

---

### The report

The weekly and monthly report is a regrouping of the sessions the timer writes, and it is built
in the browser rather than behind an endpoint of its own. Everything it needs is already in the
window: the sessions, the tasks and the companies are loaded whole at startup, so a report is a
question about data that has already arrived rather than a new one to ask the server. The
grouping is a pure function over them, `common/report/time-report.ts`, which is what makes the
arithmetic testable without a database.

A session counts on the local day it started on, which is the rule the calendar already uses, and
one still open counts up to now. Both are the readings a person checking a total against their
own memory of the week would make.

The company filter holds ids, and an empty selection means every company rather than none. The
chips are built from the unfiltered period, so narrowing to one client never removes the way back
to the others.

---

### The navigator's filing drawer

A finished task does not leave the navigator, and after a while that is most of the list: a
project with thirty closed tasks buries the three still open. So `DONE` tasks are folded into a
drawer, one per project in the company-wide view and one below the status groups when a single
project is in scope. The drawer is shut on every load and reopens only for the session: the
point is that finished work starts out of the way, not that the window remembers you looked at
it once. This is the opposite of the project collapse next to it, which persists in
`localStorage`, because that one is a layout preference and this one is a default.

The split is `common/catalog/partition-tasks.ts`, a pure function, so the rule that a `DONE`
task is filed lives in one tested place rather than in the template. `console.store` is
untouched: `visibleTasks` stays the one honest list, the tasks/notes counter still counts all
of it, and `J`/`K` still walks it. A filed task that gets selected, by search or by walking,
opens the drawer it sits in, because a selection you cannot see is one you cannot tell you made.

### The navigator row

A row is two lines with air around them rather than one crushed line. The status pip is a
coloured core inside a soft halo of the same hue, so a state carries its own weight instead of
being a hard dot shoved against the title; while a timer runs the pip becomes a sweeping ping,
because "worked right now" should read before the colour does. Progress, a wrapup mark and the
note count sit in a steady right-hand column and do not move: a number that vanishes the moment
you reach for it is a number you stop trusting, so edit rides in as its own chip on hover
instead of displacing them.

A project heading is also two lines: the title and task count on top, the company name and the
project's token total below, so none of the three is fighting the others for the same 12 pixels. The
navigator is given the width for this rather than made to crush it.

### The selected surface

A selected row sits on the raised surface, warmed from its left by its tint (the project's
identity hue in the navigators, the accent elsewhere), held by a tinted hairline, and marked by a
short solid capsule in the left gutter, clear of the top and bottom edges. Its relief is
`--shadow-key`, the same as a key, a tag chip and a project header in the navigator, so
selection reads by colour and outline, never by standing taller than its neighbours. A hovered
row only takes the raised surface's colour. The status pip of a selected row gains a thin
accent ring. `selected-row` and `selected-card` in `main.css` are the same idea at two scales.

A tag badge (`tag-chip`) is the tag picker's quiet key: a neutral face warmed a few percent by the
tag's colour, a tinted hairline, and the colour itself carried by the glowing icon.

### Labels

The kicker above a title, "Description", "Time", "Tracked this week", a form-section label in a
dialog, a company name in the scope popover, was tracked-out capitals everywhere. It is now
ordinary case in the shared `eyebrow` utility, a hair of tracking and nothing more, so it names
its section without shouting over the content beneath it. In the three writing panes it is
preceded by a 3px accent tick, the same mark on all of them, which is how you tell at a glance
which of the four surfaces is on screen. Raw status enums that used to be printed as-is
(`PAUSED`) now go through their label map (`Paused`) wherever they are shown.

The filed tone is a muted verdigris, `--color-filed`. It is deliberately neither the green of a
live `DONE` status nor the grey of disabled chrome: a shelf of finished tasks should read as an
archive, not as a list that has gone dim.

### Tokens, not values

Every size, corner, grey and tempo a component uses is a token in `main.css`, because values
picked one component at a time drift: the console had reached twenty-two font sizes, half of them
half a pixel apart.

| Token | Steps | Rule |
|---|---|---|
| Type, `text-micro` … `text-hero` | 10.5, 11.5, 12.5, 13.5, 15, 19, 26, 34 px | Micro is the floor and never prose. No line heights: text inherits its leading |
| Corners, `rounded-key` / `-control` / `-card` | 3, 5, 8 px | A chip or kbd, a button, field or row, a card, dialog or header capsule. Pills are `full` |
| Relief, `--shadow-key` / `--shadow-raised` | lip + contact; + a short cast | A key gets a faint lit lip and a contact shadow, nothing cast; a raised card adds a short soft one. Depth is a hint, never an extrusion |
| Tempo, `--motion-press` … `--motion-slow` | 90, 140, 220, 320 ms | Press, hover, a pane moving, a seal. Choreographed keyframes keep their own |
| `--color-text-subtle` | `#8c96a6` | 4.5:1 up to `surface-hover`, so small hints stay readable. Never faded further with an opacity modifier |
| `--color-border-field` | `#5e6a82` | The edge of something you type into, 3:1 against the field's well; hairline separators keep `border` |

`tests/unit/design-tokens.spec.ts` fails on a bracketed font size or radius, a faded subtle grey,
a pixel `font-size` in a style block or a numeric `duration-*`. `TerminalPane.vue` is exempt: it
frames a real terminal and is left as it is.

Two components carry what every screen repeats. `AppDialog` is the one modal shell (backdrop,
`role`, `aria-modal`, Escape, focus trap and the modal gate); `common/a11y/dialog-stack.ts` makes
only the dialog on top answer the keyboard, so a confirm over a panel closes alone, and a dialog
that is not dismissible lets Escape through to the control inside it. `AppButton` adds `quiet`,
an `icon` size and `pressed`, which lights a toggle as `key-lit` and sets `aria-pressed`, so
toolbar keys stop being hand-built from the keycap utilities. A control drawn smaller than 24px
takes `hit-target`, which grows the pointer target, not the glyph.

The two fixed panes are tokens too, `--spacing-nav` and `--spacing-notelist`. Below a 1280px
window both narrow to 272px, so at the desktop shell's 960px minimum the note still keeps a
column of about 420px.

The header is one 48px row (`--spacing-header`, shared by the pane headers so their rules line
up), a four-cell grid: the 24px mark with the history arrows, the screen tabs centred in a flexible cell, a centre cell of at most 620px and a flexible right cell. A screen teleports exactly two children into the last two, so the tabs sit in the same place on every screen; the console puts the `/rk` field in the centre and the readings and tools on the right, the others a title and actions. A
screen tab is text with an amber rule under the screen you are on (`nav-tab`, `nav-tab-current`).
On the console the `/rk` field is also the location bar: at rest it shows the open note's or
task's anchors, or the scope's, as chips, and focused it is the search. Everything right of
the tabs is the same piece, a 36px `header-capsule` with the field's corner: usage, the run queue
and the review count share one (`HeaderStatus`, segments drawn with `status-segment`), where
review is the only amber segment because it is the only one asking for action; the save dot,
Tags and Settings (`header-key`) and New note share the other. The save state is
a dot whose word is for screen readers and the tooltip. The bar gives way under 1280px: the
anchors keep only the last one, the review segment drops its words and New note becomes a `+`;
under 1100px the `⌘K` hint goes.

Browsing notes, the middle column (`NotePlacementsPane`) is one card per project the note reaches
(`NotePlacementCard`), lit with the project's hue, each task in it with its status, anchor, step
progress and two labelled keys (`NotePlacedTaskRow`): **Open task**, and **Detach**, which acts
at once and offers Undo in its toast because the note stays in its scope. A note on no task gets
an empty state with **Put it on a task**.

The calendar (`CalendarPage`, `CalendarDayCell`) heads each day with its total and a load bar:
its length is the day against an eight-hour day, its segments the projects largest first
(`common/calendar/day-shares.ts`). An eighth column totals each week; today wears an amber ring.

The console with nothing open is a desk (`ConsoleHome`, its parts under `components/console/home/`):
a headline that counts the live sessions and what waits for review (`situationOf`), the week's
tracked time as seven bars (`trackedLastDays`), a review card and a running-sessions card, the
four notes touched last as tiles, and the keys. It lays out on a container query, so the same
markup is one column in the 420px pane at the window's floor and two to four columns when wide.

---
