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
| D2 | Claude reads everything and writes a few narrow things | `rekall-mcp` depends on `rekall-core` and never on `rekall-http` or `rekall-console`, so no controller and none of the console's catalog services are on its classpath, and every read runs in a read-only transaction. The exceptions are `rekall_wrapup`, which replaces one task's wrapup; `rekall_step`, which moves a step from `open` to `running` to `claimed`, follows a flow's route on a claim, and cannot reach `done`; `rekall_record_commit`, which logs a commit against a task; `rekall_propose_step`, which can only add a draft a person has to promote; `rekall_propose_flow`, which can only lay drafts and the routes between them onto a task with no steps; `rekall_create_task`, which can only add a `TODO` task or epic whose steps are drafts; `rekall_join_epic`, which can only file one task under an epic of its project; `rekall_note`, which can only add a new note to one task; `rekall_project_section`, which can only add a section to a task's project or rewrite the body of one that already has that title; `rekall_inventory`, which can only replace one project's dependency list; and `rekall_assess_vulnerability`, which can only write a verdict on one exposure the project already has. See [design/write-safety.md](design/write-safety.md). |
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
  rekall-kernel/     RekallError, Id, Instant, the string helpers and logged best-effort cleanup: the vocabulary every layer shares
  rekall-store/      one folder per domain: SeaORM entities with their state rules and the queries over them; the SQLite connection and migrations
  rekall-diagram/    the diagram format: Mermaid sheets, code links, JSON form, validation; depends on nothing of Rekall's
  rekall-core/       the services the MCP tools may use and every port, one folder per domain: context assembly, the step and review lines, flows, wrapups, notes, time entries, token usage, vulnerabilities
  rekall-console/    the services only the console calls, one folder per domain: commands in, views out, no HTTP types
  rekall-http/       Axum REST routes for the UI, one folder per domain: controllers, request and response shapes and routes; `shared` holds the error body, request extractors and response helpers
  rekall-mcp/        MCP server: three tools read, twelve write; a folder per domain holds its tools
  rekall-infra/      the implementations of core's ports, a folder per capability: the in-app terminal (`pty`), the run queue's worker, the usage meter, the transcript reader, the Claude folder editor (`resources`), the Claude Code installer, the credential sessions (`connection`), the database registry (`bootstrap`), backups and restores, the restart handle, the release feed and the macOS update installer (`version`), the OS credential store (`secret`), the connection probes, the Jira search, the GitHub Advisory Database client; it serves no route
  rekall-app/        rekall-server: builds the infrastructure and injects it, mounts the HTTP and MCP routers, the local-access guard, serves the built frontend, embedded at compile time
  rekall-app/desktop the Tauri desktop app over the same server, in-process
  rekall-ui/         Vue 3 + Vite (built into rekall-ui/dist, embedded by rekall-app)
```

**A crate is a layer and a folder is a domain.** The layers are separate crates in one Cargo
workspace, so the dependency direction is enforced by the compiler, not by convention. Inside a
layer, the same domain names cut every crate (see Domains below), so the code of one domain is one
folder per crate. The chain:

1. `rekall-kernel` <- `rekall-store` <- `rekall-core` <- `rekall-console` <- `rekall-http`: each layer
   sees the ones before it, so `rekall-store` cannot see a route and `rekall-core` cannot see a
   console service.
2. `rekall-mcp` sits beside the console and sees `rekall-core` only.
3. `rekall-infra` sits beside them too: it sees `rekall-core` to implement its ports and nothing that
   serves HTTP, and `rekall-http` never sees it.
4. `rekall-app` sees everything: it builds the `rekall-infra` implementations, hands them to
   `rekall-http` in `ApiPorts`, and mounts both routers.

`rekall-mcp` lists `rekall-kernel` and `rekall-core` and no other Rekall crate: no entity,
repository, console service or route is in its reach, and the few `rekall-store` enums its tools
read (`TaskKind`, `TaskStepState`, `WrapupAuthor`, `ExposureVerdict`, `AdvisorySeverity`) come
re-exported from the `rekall-core` domain that owns them. It and `rekall-http` are
independent consumers of the same services, which is what keeps the boundary structural rather
than accidental: `rekall-mcp` can reach `ContextService`, `TaskStepService` and `WrapupService`,
but no route, none of `CompanyService`, `ProjectService`, `TaskService` or `DocumentService` is among its dependencies.

### Layout inside a crate

The crates are the layers; inside one, the code is laid out the way a Java project is.

- **A folder per domain, one public type per file.** A folder is a package, and every layer crate cuts its code the same way (see Domains below): `rekall-core/src/note/`, `rekall-http/src/note/`, `rekall-store/src/note/`, `rekall-console/src/note/`, `rekall-mcp/src/note/`. A file holds one public type and is named after it: `NoteService` lives in `note/service/note_service.rs`, `CompanyResponse` in `catalog/response/company_response.rs`. Inside `rekall-http` a domain folder holds its controllers, request and response shapes and a `routes.rs` that merges its controllers, so `router.rs` only merges the domains. A service in `rekall-console` takes a command and returns a view, both in the same domain; the controller converts the request into the command and the view into the response (`From` impls in the request and response files).
- **Inside a domain, a folder per kind.** A domain folder that holds more than one kind of type puts each kind in a subfolder named after it, and keeps only `mod.rs` (and, in `rekall-http`, `routes.rs`) at its root. The kind comes from the file's suffix: `rekall-console` has `service/`, `command/`, `view/`, `rule/` and `support/` (the private helpers its services lean on, such as `Snapshot`); `rekall-http` has `controller/`, `request/` (queries and path parameters too), `response/` and `support/`; `rekall-core` has `service/`, `view/`, `event/` (`*_stream_event`), `port/` (a trait with its in-memory doubles), `rule/` and `model/` (the values its services and ports carry); `rekall-store` has `entity/` and `repository/`. A domain of one kind stays flat, and so do `rekall-mcp`, whose folders hold tools only, and `rekall-infra`, whose folders are capabilities. A kind's `mod.rs` declares its files `pub(super)`, and the domain's `mod.rs` re-exports the public types from them.
- **`mod.rs` and `lib.rs` hold no logic.** They declare the files and re-export the public types, so a caller writes `rekall_core::note::NoteService` and never the file's name. Moving a type between files changes no caller.
- **Privacy is per file, so what a domain shares is `pub(in crate::<domain>)`.** Rust's privacy boundary is the module, which is the file, not the type. A helper two files of one domain use, such as `connection/rule/connection_rules.rs`, is `pub(in crate::connection)`, visible to that domain's kinds and nowhere else: Java's package-private. In a flat folder it is `pub(super)`.
- **Private helpers stay beside the one type that uses them.** A small private struct that only one file reads (`ScratchDir` beside `Database`, `Active` beside `Worker`) stays in that file. A private type that carries most of a module's logic gets a file of its own, visible as `pub(super)`: the run queue's `Worker` sits in `run_queue_worker.rs`, beside the `RunQueueRunner` handle that starts it.
- **A module of free functions is one file named for what it does.** `task/task_repository.rs` is the task repository, `jstr.rs` the string helpers, `usage_ceiling.rs` the ceiling rule: Rust's counterpart of a class of static methods.
- **Tests live under `tests/`, split in two.** `tests/unit/` mirrors `src/`: the tests of `rekall-console/src/filesystem/service/directory_listing_service.rs` are `rekall-console/tests/unit/filesystem/service/directory_listing_service_tests.rs`, attached from the implementation with `#[cfg(test)] #[path = "../../../tests/unit/filesystem/service/directory_listing_service_tests.rs"] mod tests;`, so the file is still a child module of the code it tests and reads private items with `use super::*`. `tests/integration/` holds the integration and end-to-end suites, which see only the public API, with their `support/` helpers and fixtures. Cargo discovers only files directly in `tests/`, so it finds neither folder by itself: unit files are compiled through their `#[path]`, and every integration suite is declared as a `[[test]]` in its crate's `Cargo.toml`. A suite missing from that list never runs, and nothing warns about it. `autotests` stays on, since nothing sits in `tests/` itself for it to pick up.

Two things keep their shape on purpose. A SeaORM entity is one file with `Model`, `Relation` and the `Entity` and `ActiveModel` its derive generates: the derive requires them in one module, and the file is the entity, as a JPA entity is one class. The migrations are one file each, starting from `baseline.sql`, the schema as v0.2.x left it; a file once released is frozen history. In the desktop app, the modules holding `#[tauri::command]` functions are `pub`, because `generate_handler!` resolves each command's generated companion by path and a re-export does not carry it.

### Domains

A domain is the unit a change lands in. The same name cuts every layer, so the code of one domain
is found by opening one folder per crate:

| Domain | Holds |
|---|---|
| `catalog` | companies and projects with their logos and icons; in `rekall-console`, `rekall-http` and the UI also the tasks |
| `component`, `section` | a project's components and its titled pages |
| `connection` | the GitLab instances, Kubernetes clusters and Jira sites a company's projects reach, the projects that bind each, the log of every credential a terminal asked for, and the Jira issues a project imports as tasks |
| `task` | the task row, and in `rekall-core` and `rekall-mcp` a session creating one or filing it under an epic; the console's task service is in `catalog` |
| `step`, `flow` | the checklist, and the routes and visits that turn it into a flow |
| `graph` | a project's deployment graphs and laying one on a task; console-only, so it has no folder in `rekall-core` |
| `note` | documents, their attachment to tasks, the scope that decides how they load |
| `wrapup`, `review`, `timeentry` | the wrapup, the review line, logged time |
| `commit`, `diagram`, `token`, `search`, `context` | logged commits, stored diagrams, token usage, search, context assembly and its size |
| `skill`, `tag`, `queue`, `claude` | skill assignments, tags, the run queue, what a session launch needs |
| `backup`, `settings`, `version` | the backups of the open database and restoring one, the registered database folders and the restart onto another, the update check; no table behind them, so no folder in `rekall-store` |
| `inventory` | what each project is built from: its dependencies at the version each resolves to |
| `terminal`, `filesystem`, `stream` | the in-app terminals, the folder listing behind the path picker, the event stream the console listens to; console and HTTP only |
| `vulnerability` | the advisories, the projects they reach, the scan that finds them and what it looks for, a session's verdict and the fix task |

`diagram` also has `rekall-diagram`, the one domain with a crate of its own, because the format
depends on nothing of Rekall's. A domain gets a crate only on that condition.

In `rekall-store` a domain folder holds each entity under `entity/` (`step/entity/task_step.rs`)
and the queries over its table under `repository/` (`step/repository/task_step_repository.rs`). A
domain with a single entity named like itself (`task`, `tag`, `wrapup`, `diagram`) keeps the entity
as one flat file at the crate root, since a folder holding only that entity would repeat the name;
that file declares its queries, which sit in `task/task_repository.rs`.

Entity modules keep their flat path at the crate root (`rekall_store::task_step`), and the
`repository` module re-exports each query module under its table's name
(`rekall_store::repository::task_step`, which callers alias as `repo`): the folder is where the file
lives and the re-export is how a caller names it, so a move between folders changes no caller.

**Allowed edges.** Two suites in `rekall-app/tests/integration/` read every crate's manifest and
sources. `crate_edges.rs` fails on a `Cargo.toml` that lists a Rekall crate it is not allowed to, in
any dependency section (dev and target-specific ones included), and on a workspace member missing
from its table:

| Crate | May depend on |
|---|---|
| `rekall-kernel` | nothing |
| `rekall-diagram` | nothing |
| `rekall-store` | `rekall-kernel` |
| `rekall-core` | `rekall-diagram`, `rekall-kernel`, `rekall-store` |
| `rekall-console` | `rekall-core`, `rekall-kernel`, `rekall-store` |
| `rekall-http` | `rekall-console`, `rekall-core`, `rekall-diagram`, `rekall-kernel`, `rekall-store` |
| `rekall-mcp` | `rekall-core`, `rekall-kernel` |
| `rekall-infra` | `rekall-core`, `rekall-diagram`, `rekall-kernel`, `rekall-store` |
| `rekall-app` | `rekall-core`, `rekall-http`, `rekall-infra`, `rekall-kernel`, `rekall-mcp`, `rekall-store` |
| `rekall-app/desktop` | `rekall-app`, `rekall-infra` |

`domain_edges.rs` fails on a domain that names another one it is not listed for, wherever the path
points: into its own crate or into another Rekall crate's domain (`rekall_store::task` in a `note`
file is `note -> task`). It also fails on a domain missing from its crate's table, and on two
domains of one crate whose own code uses each other; `rekall-store` is exempt from that last rule,
because SeaORM declares a relation from both ends. A dash means the crate has no such domain:

| Domain | `rekall-store` | `rekall-core` | `rekall-console` | `rekall-http` | `rekall-mcp` |
|---|---|---|---|---|---|
| `backup` | - | nothing | nothing | nothing | - |
| `catalog` | `task` | - | `commit`, `note`, `review`, `step`, `tag`, `task`, `timeentry`, `wrapup` | nothing | - |
| `claude` | - | `catalog`, `connection`, `task` | nothing | `skill` | - |
| `commit` | `step`, `task` | `catalog`, `step`, `task` | - | nothing | nothing |
| `component` | `catalog` | `connection`, `task` | nothing | nothing | - |
| `connection` | `catalog`, `task` | nothing | `catalog`, `commit`, `component`, `task` | `catalog` | - |
| `context` | - | `catalog`, `commit`, `component`, `connection`, `note`, `step`, `tag`, `task`, `wrapup` | - | nothing | nothing |
| `diagram` | `catalog`, `task` | `catalog`, `task` | nothing | nothing | nothing |
| `filesystem` | - | - | nothing | nothing | - |
| `flow` | `step`, `task` | `step` | - | `step` | `step` |
| `graph` | `catalog`, `flow`, `step` | - | `catalog`, `flow`, `step`, `task` | `step` | - |
| `inventory` | `catalog` | `catalog` | - | nothing | `vulnerability` |
| `note` | `task` | `task` | `catalog` | `catalog` | nothing |
| `queue` | `task` | `catalog`, `claude`, `task` | nothing | nothing | - |
| `review` | - | `task`, `timeentry` | - | - | - |
| `search` | - | `task` | - | nothing | - |
| `section` | `catalog` | `catalog`, `note` | nothing | nothing | `note` |
| `settings` | - | nothing | nothing | nothing | - |
| `skill` | `step`, `tag`, `task` | `claude` | nothing | nothing | - |
| `step` | `task` | `flow`, `task`, `timeentry` | - | nothing | `commit`, `flow` |
| `stream` | - | - | - | nothing | - |
| `tag` | `task` | - | nothing | nothing | - |
| `task` | `catalog`, `note`, `step`, `tag`, `wrapup` | `step` | - | - | nothing |
| `terminal` | - | - | `claude` | `claude`, `vulnerability` | - |
| `timeentry` | `task` | `catalog`, `task` | - | nothing | - |
| `token` | `step`, `task` | nothing | - | nothing | - |
| `version` | - | nothing | nothing | nothing | - |
| `vulnerability` | `catalog`, `task` | `catalog`, `commit`, `connection`, `inventory` | `catalog`, `step`, `task` | `catalog` | nothing |
| `wrapup` | `task` | `catalog`, `review`, `task` | - | nothing | `commit` |

Each crate's foundations are modules every domain may use and are no domain: `ctx`, `events`,
`load`, `services`, `text_rule` and `tx` in `rekall-core`; `constraints`, `enums`, `prelude`,
`slug` and `stamped` in `rekall-store`, with `database`, `migration`, `query_support`,
`stamped_update` and `repository`, whose re-exports the test reads back to their domains; `api_ports`, `api_state`, `router` and `shared` in `rekall-http`; `controller`, `protocol`, `texts`,
`tool` and `tool_registry` in `rekall-mcp`. A new edge is a design decision: change the table and
the test together, or change the code.

**Events.** A domain announces a change by implementing `StreamEvent` for its own type (a frame
name and a serialisable payload) and publishing it on its transaction. `events` imports no domain, so
adding an event touches the domain only; a subscriber that acts on one kind reads it with
`downcast::<E>()`.

### Where a feature goes

A new feature is a folder first. What it is decides which kind:

| It is | Where it goes | Examples |
|---|---|---|
| Rekall's data and the rules on it | a domain folder in each layer crate it needs: the entities and queries in `rekall-store`, the rules a session may use in `rekall-core`, the console's writes in `rekall-console`, the routes in `rekall-http`, the tools in `rekall-mcp` | `note`, `step`, `wrapup` |
| Something that knows nothing of Rekall | a crate of its own, depending on no Rekall crate | `rekall-diagram` |
| A capability that touches the machine: a process, a file, a socket, outbound HTTP | a port: the trait in a `rekall-core` domain, a thin service over it in `rekall-console`, the implementation in a `rekall-infra` folder, injected by `rekall-app` through `ApiPorts` | the terminals, the backups, the database registry, the release feed, the OS credential store |

- **A rule that needs only data never goes in `rekall-infra`.** One that reads the database is a
  `rekall-core` service (`RunQueueService`, `ConnectionCheckService`); one only the console needs is a
  `rekall-console` service over the port (`DatabaseFolderService` decides which database is active,
  `UpdateCheckService` when the release feed is asked again).
- **A port with one implementation still has a second: the fake in its console service's unit test.**
  That is the seam. Where an end-to-end test needs to swap it too, `StartOptions` carries it.
- **A capability that no route reaches and only `rekall-infra` uses gets no port**: the CLI, Claude
  Code's own token (`credentials`), the transcript reader, the update installer the desktop bridge
  calls.

`claude` exists twice on purpose. `rekall-core/src/claude/` reads the entities a launch needs (the
launch target of a task or project, the work left in a task), the models and efforts a session may
start with, and `ResourcePath`, so `rekall-infra` never touches a repository; `rekall-infra` owns
everything with a process, a file or a socket behind it: the terminals, the run queue's worker, the
usage meter, the transcripts, `~/.claude`, the Claude Code installer. A rule that needs the database
goes in the first, one that needs the machine in the second. The run queue's rows and moves are
`RunQueueService` in `rekall-core/src/queue/`, and how skills are grouped is `SkillGroupService` in
`rekall-core/src/skill/`.

### Adding a field or a capability

A field on an entity:

1. A migration in `rekall-store/src/migration/` with the next number, declared in `mod.rs` and listed
   in `migrator.rs`. A released migration is never edited.
2. The column on the entity in `rekall-store/src/<domain>/`, its length in `constraints` when it has
   one, and its write through `repo::update_if_changed`.
3. The service that writes it: its command and view in `rekall-console` for a console write, or the
   `rekall-core` service when a session reads or writes it.
4. The request and response in `rekall-http/src/<domain>/`, mapped with `From` impls in their files.
5. The tool's input and output in `rekall-mcp/src/<domain>/`, when a session reads or writes it.
6. The type in `rekall-ui/src/features/<feature>/model` and the call in its `api`.
7. The entity's row in §4, in the same commit.

A capability that touches the machine:

1. The trait and the types it carries in the `rekall-core` domain that uses it, answering
   `RekallError` through `Result`.
2. A thin service over it in the same domain of `rekall-console`, with a unit test over a fake.
3. The implementation in a `rekall-infra` folder, holding no rule that needs only data.
4. A field on `ApiPorts`, the service built from it in `ApiState::new`, and the implementation
   built in `rekall-app` (from `StartOptions` when a test swaps it).
5. The controller in `rekall-http` calls the console service.
6. A new domain folder goes in its crate's table in `domain_edges.rs` and in the tables above.

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
                         ├──> Task (its epic, optional)
                         └──1 Wrapup

Company ──< ExternalConnection ──< ConnectionAccess
                    │
Project ──< ProjectConnection >──┘

Project ──< DeploymentGraph ──< GraphStep ──< GraphRoute
                                    └──> ExternalConnection (optional)

Project ──< ProjectDependency
Project ──< VulnerabilityExposure >── SecurityAdvisory
                    └──> Task (the fix, optional)
```

| Table | Anchored by | Columns of note |
|---|---|---|
| `company` | `name`, unique | `description` |
| `project` | `label`, unique per company | `title`, `status`, `description`, `company_id` |
| `task` | `label`, unique per project | `title`, `status`, `description`, `project_id`, `graph_laid_at` (when a deployment graph was laid on it), `release_skipped` (the project's release graph is not laid when the console runs it; migration `054-release-on-run`), `kind` (`TASK` or `EPIC`), `epic_id` (the epic of the same project it belongs to, set null when the epic is deleted; migration `055-epic`) |
| `document` | — | `title`, `kind`, `body_markdown` |
| `document_task` | — | `document_id`, `task_id`, `position` |
| `task_step` | through its task | `task_id`, `title`, `body_markdown`, `done`, `done_at`, `position`, `flow_x`/`flow_y` (where it was dragged on the flow canvas, null for the automatic layout), `max_visits` (how many times a flow may enter it, null for no limit), `starts_session` (whether it opens a fresh Claude session instead of joining the one the step before it ran in), `connection_id` (the company connection it reaches, set when a deployment graph laid it; null when the connection is deleted) |
| `step_link` | through its task | `task_id`, `from_step_id`, `to_step_id` (null: the route ends the flow), `outcome` unique per `from_step_id`, `position`; deleted with either step |
| `step_visit` | through its task | `task_id`, `step_id`, `ordinal` unique per task, `entered_at`, `left_at`, `outcome`, `report` (what the claim that closed it reported; migration `053-visit-report`); deleted with its step |
| `wrapup` | through its task | `task_id` unique, `body_markdown`, `written_by` |
| `project_component` | through its project | `project_id`, `name`, `folder`, `description`, `position` |
| `deployment_graph` | through its project | `project_id`, `title` (unique per project, ignoring case), `description`, `release_default` (laid on every task the console runs; at most one per project); deleted with its project |
| `graph_step` | through its graph | `graph_id`, `position` (0 is where the flow begins), `title`, `body_markdown`, `connection_id` (null when it reaches nothing or the connection is deleted), `max_visits`, `flow_x`/`flow_y` (where it was dragged on the graph's canvas, null for the automatic layout; migration `050-graph-step-place`); rewritten whole with its graph |
| `graph_route` | through its graph | `graph_id`, `from_step_id`, `to_step_id` (null: the route ends the flow), `outcome` unique per `from_step_id`, `position`; deleted with either step |
| `external_connection` | `label`, unique per company | `company_id`, `title`, `kind` (`GITLAB`, `KUBERNETES`, `JIRA`), `base_url` (the GitLab instance or Jira site, or the server read from an imported kubeconfig), `kube_context`, `proxy_url` (Jira only: the proxy its calls go through), `secret_ref` (the OS keychain account holding the credential, generated, unique), `secret_set_at` (null while no credential is stored). No column holds a credential |
| `project_connection` | `label`, unique per project | `project_id`, `connection_id`, `target` (GitLab project path, Kubernetes namespace or Jira project key). A project may bind one connection several times, one target each; the label starts as the connection's and is what a session names the binding by (migration `051-binding-label`). Deleted with either side |
| `connection_access` | through its connection | `connection_id`, `task_id` (set null when the task goes), `terminal_id`, `accessed_at`, `granted`, `reason` (why the request was refused); append-only, never a credential |
| `project_dependency` | through its project | `project_id`, `ecosystem`, `name`, `version`, `manifest` (the declaring file, relative to the project's folder), `created_at`; unique per project, ecosystem, name and version; replaced whole |
| `security_advisory` | `ghsa_id`, unique | `cve_id`, `summary`, `description`, `url`, `published_at`, `severity` (GitHub's, upper-cased; `CRITICAL` on advisories recorded before it), `malware`, `affected_json` (the packages it affects, one entry per release line: ecosystem, name, vulnerable range, first patched version, vulnerable functions) |
| `vulnerability_exposure` | through its project | `project_id`, `advisory_id`, `ecosystem`, `package_name` and `package_version` unique together (one exposure per package version an advisory reaches), `verdict` (`AFFECTED`, `NOT_AFFECTED`, null until judged), `reasoning`, `conclusion` (why the project is or is not affected, in plain words; null on verdicts given before it), `assessed_at`, `assessed_commit` (the project's `HEAD` at the verdict), `fix_task_id` (set null when the task goes); deleted with its project or advisory |
| `scan_scope` | one row | `include_high`, `include_malware`; absent until first changed, which reads as both off |
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


### 4.2 to 4.5 Steps, flows, deployment graphs and epics

The step line, the flow with its routes and visits, the deployment graphs and the epics are
designed in [design/workflow.md](design/workflow.md), under the same numbers.

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
(`rekall_store::skill_name`): `rekall-console` cannot see `~/.claude`, which `rekall-infra` reads, so the
picker offers only what exists and a skill that later vanishes stays assigned and is flagged in the
console. The writes are `SkillAssignmentService` in `rekall-console`, each replacing one owner's whole
list; `rekall-mcp` can read the skills through the context and never write them.

A project's connections arrive as a `<connections>` block after its components, one line per
binding: the binding's own label, the connection's title and how to reach it. A cluster is `kubectl --context <label>` with the
binding's namespace; a GitLab instance is its URL and project path, pushed to over HTTPS through the
configured credential helper and called through `rekall-gitlab --connection <label>`; a Jira site is its
URL and project key, called through `rekall-jira --connection <label>` under `/rest/api/2`, which Jira Cloud
and Data Center both serve, its search named for each. A connection with no credential
says so, so the session reports it. `ConnectionView` carries no field that could hold a credential,
the keychain account, a cluster's server or its CA, and the block says these work only in a terminal
Rekall opened and forbids handling a credential directly; `/rk` repeats that rule. A step laid from a
deployment graph names the connection it works through on a `Connection:` line by the labels of the
project's bindings of it (`a` or `b` when it binds it twice; see 4.4 in [design/workflow.md](design/workflow.md)), and says to report one the
project no longer binds.

Scalars are rendered as bullets and bodies as tags, which is why a description is a `<description>` block rather than a `description` bullet. It is a document, with headings, lists and a scope section, and a document inlined into a list item stops being one: every line after the first falls outside the bullet, and its own headings outrank the record's.

That distinction is why the two-anchor form exists. `/rk project:vega` loads the project and
lists its tasks as anchors; naming the task as a second anchor is how you narrow to one.

A note arriving with every task that references it is the point of the whole design. One row,
many links, is what lets a cluster note reach every task it belongs to without going stale in
any of its copies.

---

## 6. Design notes by topic

Each file keeps the decisions of one area and the reasons for them, as the code stands.

| Topic | Holds |
|---|---|
| [design/workflow.md](design/workflow.md) | 4.2 to 4.5: the step line and its live loop, flows, deployment graphs, epics |
| [design/mcp.md](design/mcp.md) | The MCP tools one by one, `[[...]]` references in a description, the two protocol eras |
| [design/write-safety.md](design/write-safety.md) | What `rekall-mcp` can write and cannot, connections, credentials, repository discovery, Jira import |
| [design/vulnerabilities.md](design/vulnerabilities.md) | The inventory, the advisory feed, the scan, verdicts and fix tasks |
| [design/terminals.md](design/terminals.md) | Open in terminal, the in-app PTY terminal, credential sessions |
| [design/claude-and-tokens.md](design/claude-and-tokens.md) | The Claude folder editor and the memory review; reading token usage from transcripts |
| [design/console-ui.md](design/console-ui.md) | The report, the navigator, design tokens, the header |
