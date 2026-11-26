# Rekall - Design Document

Status: label, title and description on projects and tasks; notes that belong to one task; one
wrapup per task, written by Claude and corrected by hand; a step checklist a session moves along
and a person ticks done, optionally routed into a flow

Rekall stores the structure and the working context of the projects and tasks you work on, and
hands them to the chosen session provider over MCP. A session can write the wrapup of
a task, which is what that task's implementation currently looks like, and the state of a step,
which it can move from open to running to claimed. The last tick, done, is a person's.
The console uses the saved provider by default and lets a person override it for a single task
launch when both CLIs are installed. The launch uses the chosen provider's saved options.

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

Target: type `/rk task:VG-12` and have the full context
loaded in one call, including its notes.

---

## 2. Decisions

| # | Decision | Rationale |
|---|---|---|
| D1 | Real tables with real foreign keys, each leading an index | A note can never point at a deleted task. SQLite indexes no foreign-key column on its own, so every one leads an index (the `schema_indexes` test of `rekall-store` fails otherwise) and deleting a parent does not scan its children. A visit, route, snapshot or check result that names a step and its task references the pair `(task_id, id)` of the step, so the two cannot disagree (migration `079-step-owned-task-keys`; `token_usage` keeps a single-column key because `SET NULL` on a pair would clear its task). On an app whose only job is to be a reliable memory, silent dangling references are the failure mode that matters. |
| D2 | Claude reads everything and writes a few narrow things | `rekall-mcp` depends on `rekall-core` and never on `rekall-http` or `rekall-console`, so no controller and none of the console's catalog services are on its classpath, and every read runs in a read-only transaction. The exceptions are `rekall_wrapup`, which replaces one task's wrapup; `rekall_step`, which moves a step from `open` to `running` to `claimed`, follows a flow's route on a claim, and cannot reach `done`; `rekall_record_commit`, which logs a commit against a task; `rekall_propose_step`, which can only add one step, a draft when asked; `rekall_propose_flow`, which can only lay steps and the routes between them onto a task with no steps, as drafts when asked; `rekall_replan` and `rekall_delete_step`, which can only delete draft and open steps of a task, the first inside the transaction that lays their replacement; `rekall_edit_step`, which can only rewrite the title or detail of one draft or open step; `rekall_create_task`, which can only add a `TODO` task or epic whose steps, a checklist or a flow, start open; `rekall_join_epic`, which can only file one task under an epic of its project; `rekall_description`, which can only replace the description of one task, whole, opening no revision and keeping no copy of the old text; `rekall_note`, which can only add a new note to one task; `rekall_feedback`, which can only answer open entries (waiting or in a pass) of one task's thread by their seq; `rekall_project_section`, which can only add a section to a task's project or rewrite the body of one that already has that title; `rekall_inventory`, which can only replace one project's dependency list; and `rekall_assess_vulnerability`, which can only write a verdict on one exposure the project already has. See [design/write-safety.md](design/write-safety.md). |
| D3 | Markdown content lives in the database | One backup target, reachable through MCP, searchable. |
| D4 | One entry point, and it is a slash command | A session begins with `/rk task:VG-12`, not with a question. Reaching a record through a natural-language query costs several turns and a few thousand tokens before any work starts, and it is the part that fails when the model guesses the wrong entity. An explicit anchor removes both. |
| D5 | The model is fixed at compile time | There is no runtime meta-model and no DDL engine. Company, project, task and document are fixed SeaORM entities; adding a new kind of record is a model and a migration, not a screen. The status, state and kind columns of `project`, `task`, `task_step`, `run_queue`, `run_queue_item`, `external_connection` and `task_feedback` carry a `CHECK` of the stored names of their enum (migration `078-enum-and-scope-checks`): a new variant of one of those enums needs a migration that rebuilds its table. |
| D6 | Modular monolith, single process, embedded database | Single user, localhost. The application has to be reachable with one command or it will not get used. Localhost is enforced, not assumed: `LocalAccessFilter` refuses a peer that is not loopback, a `Host` that is not this machine and an `Origin` from another site, since nothing here authenticates and the API can open a terminal. |
| D7 | What a record is called and what an anchor resolves are two columns | One column serving both jobs would mean a rename breaks anchors written down elsewhere, and makes every name a compromise between readable and typeable. `label` is a slug and is the identity; `title` is prose and is free. See §4. |
| D8 | A task carries one wrapup, and it is a state and not a log | The thing that costs a session its first twenty minutes is reconstructing what the code already does. A note cannot answer that: notes accumulate, and the reader has to synthesise the current state out of them. A wrapup is that synthesis, written once and overwritten thereafter. Its own table, because notes accumulate on a task by construction and "one answer per task" has to be a constraint rather than a convention. See §4.1. |
| D9 | An export's diagrams are drawn by the console and sent with the request, and its file is stored and downloaded by link | The server has no browser, and the console already draws Mermaid: it draws each fence again on a light palette (`diagram-palette.ts`), takes a PNG and posts `{ source, png }` pairs beside the choice, so `rekall-export` only places pictures it is handed and a fence with none becomes an error paragraph, never a failed export. A finished file is stored in `project_export_file` and the person downloads it from `GET /api/exports/{id}/file`, a plain attachment link, as the desktop shell already saves a backup, so the console handles no blob and an export can be fetched again until it is deleted. |
| D10 | Every text a session saves for a person opens on a scenario | A session writes with the code loaded, so a step, note or deviation can be accurate and still unreadable to the person who reviews it in the console without the code. `.claude/rk/reader.md` sets one order (scenario, what changes, the parts involved in plain words then code names, terms, the work) and the modes that write such text point to it; the MCP tools that write a task, a step, a note, a deviation or a decision end their descriptions with its short form (`reader!` in `rekall-mcp/src/texts.rs`), so a client that never reads the modes gets it too. A diagram carries the same thing as data: a `Scenario` on `DiagramDocument`, required on every diagram a session writes (`DocumentValidator::validate_generated`) and optional on an imported or older one, and the console opens a walkthrough on it. |
| D11 | Pictures are files beside the database, addressed by URL in the markdown | The database stays text. A picture is copied to `images/<company>/<project>/...` beside the database file and the markdown holds its address, `/api/images/...`. The address is the picture's identity: renaming or moving a record moves no file, so no link breaks, and the eight characters of the content's hash in the name keep two pictures from overwriting each other. Only a `PicturePath` names a file there, the format is read from the content and never from the extension, and a picture is at most 10 MB. A database that is not a file keeps no pictures. Every backup zip holds them under `images/`, so it restores on its own; the price is that each kept backup repeats every picture, a download reads the zip into memory and an uploaded restore stops at 1 GB. A restore only adds and overwrites pictures, and takes no entry that is not a `PicturePath`. |

Non-goals: multi-user, authentication, remote deployment, vector search.

---

## 3. Module structure

```
rekall/
  rekall-kernel/     RekallError, Id, Instant, the string helpers and logged best-effort cleanup: the vocabulary every layer shares
  rekall-store/      one folder per domain: SeaORM entities with their state rules and the queries over them; the SQLite connection and migrations
  rekall-diagram/    the diagram format: its scenario, Mermaid sheets, code links, JSON form, validation; depends on nothing of Rekall's
  rekall-export/     a project's pages as a PDF or Word file: the document model, the markdown it is read from and the two writers (`DocxWriter`, `PdfWriter`); depends on nothing of Rekall's
  rekall-core/       the services the MCP tools may use and every port, one folder per domain: context assembly, the step and review lines, flows, wrapups, notes, time entries, token usage, vulnerabilities
  rekall-console/    the services only the console calls, one folder per domain: commands in, views out, no HTTP types
  rekall-http/       Axum REST routes for the UI, one folder per domain: controllers, request and response shapes and routes; `shared` holds the error body, request extractors and response helpers
  rekall-mcp/        MCP server: six tools read, twenty-one write; a folder per domain holds its tools
  rekall-infra/      the implementations of core's ports, a folder per capability: the in-app terminal (`pty`), the chat session (`chat`) and the `session` host that routes between the two, the run queue's worker, the usage meter, the transcript reader, the Claude folder editor (`resources`), the Claude Code installer, the shell that runs a step's checks (`check`), the git worktrees of a task (`worktree`), the credential sessions (`connection`), the database registry (`bootstrap`), backups and restores, the restart handle, the release feed and the macOS update installer (`version`), the credential file and the OS credential store it replaced (`secret`), the connection probes, the pictures beside the database (`picture`), the Jira search, the task manager's REST calls and the runner that creates a new task's issue in it (`tracker`), the GitHub Advisory Database client; it serves no route
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
5. `rekall-diagram` and `rekall-export` depend on no Rekall crate. `rekall-export` takes markdown parts and
   the pictures of their diagrams and returns the bytes of a PDF or Word file; `rekall-console` calls it and
   nothing below `rekall-console` does.

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
- **`mod.rs` and `lib.rs` hold no logic.** They declare the files and re-export the public types, so a caller writes `rekall_core::note::NoteService` and never the file's name. Moving a type between files changes no caller. The in-memory and scripted test doubles of `rekall-core`'s ports (`ScriptedCheckRunner`, `ScriptedWorktreeHost`, `ScriptedTaskTracker`, `InMemoryChatPipe`, `RecordingChatListener`, `InMemorySecretStore`, `InMemoryTerminalHost`) are compiled only for its own tests or with its `testing` feature, which the dev-dependencies of the crates whose tests use them turn on (`rekall-core` lists itself there, the one self-edge the crate-edge test allows, for its own integration tests). `ContextRenderer` is composed in `rekall-core/src/context/model/context_renderer.rs` and each section of the context is one file of `rekall-core/src/context/render/` that adds its `render_*` methods to it. A large service is split the same way, one file per job: `TaskStepService` keeps a step's lifecycle while `StepBindingService` (the session a step opens, the connection it reaches) and `StepPlacementService` (its canvas place and visit cap) write only those columns; the run queue's worker (`rekall-infra/src/queue/run_queue_worker/`) and `RunQueueService` (`run_queue_service/`) have a file per phase; `Instance::build` calls the functions of `rekall-app/src/server/instance_parts.rs`. A domain named like its entity (`task`, `tag`, `wrapup`, `diagram` in `rekall-store`) re-exports that entity with a glob (`pub use entity::task::*`): a `pub use` of an item called `task` would clash with the module of that name.
- **Privacy is per file, so what a domain shares is `pub(in crate::<domain>)`.** Rust's privacy boundary is the module, which is the file, not the type. A helper two files of one domain use, such as `connection/rule/connection_rules.rs`, is `pub(in crate::connection)`, visible to that domain's kinds and nowhere else: Java's package-private. In a flat folder it is `pub(super)`.
- **Private helpers stay beside the one type that uses them.** A small private struct that only one file reads (`ScratchDir` beside `Database`, `Active` beside `Worker`) stays in that file. A private type that carries most of a module's logic gets a file of its own, visible as `pub(super)`: the run queue's `Worker` sits in `run_queue_worker.rs`, beside the `RunQueueRunner` handle that starts it.
- **A module of free functions is one file named for what it does.** `task/repository/task_repository.rs` is the task repository, `jstr.rs` the string helpers, `usage_ceiling.rs` the ceiling rule: Rust's counterpart of a class of static methods.
- **Tests live under `tests/`, split in two.** `tests/unit/` mirrors `src/`: the tests of `rekall-console/src/filesystem/service/directory_listing_service.rs` are `rekall-console/tests/unit/filesystem/service/directory_listing_service_tests.rs`, attached from the implementation with `#[cfg(test)] #[path = "../../../tests/unit/filesystem/service/directory_listing_service_tests.rs"] mod tests;`, so the file is still a child module of the code it tests and reads private items with `use super::*`. `tests/integration/` holds the integration and end-to-end suites, which see only the public API, with their `support/` helpers and fixtures. Cargo discovers only files directly in `tests/`, so it finds neither folder by itself: unit files are compiled through their `#[path]`, and every integration suite is declared as a `[[test]]` in its crate's `Cargo.toml`. A suite missing from that list never runs, and nothing warns about it. `autotests` stays on, since nothing sits in `tests/` itself for it to pick up. In `rekall-ui`, `tests/unit/support/offline.setup.ts` runs before every test file and leaves `fetch` pending, so a request a test does not fake never reaches the network or fails in the middle of another assertion.

Two things keep their shape on purpose. A SeaORM entity is one file with `Model`, `Relation` and the `Entity` and `ActiveModel` its derive generates: the derive requires them in one module, and the file is the entity, as a JPA entity is one class. The migrations are one file each, starting from `baseline.sql`, the schema as v0.2.x left it; a file once released is frozen history. In the desktop app, the modules holding `#[tauri::command]` functions are `pub`, because `generate_handler!` resolves each command's generated companion by path and a re-export does not carry it.

### Domains

A domain is the unit a change lands in. The same name cuts every layer, so the code of one domain
is found by opening one folder per crate:

| Domain | Holds |
|---|---|
| `catalog` | companies and projects with their logos and icons; in `rekall-console`, `rekall-http` and the UI also the tasks |
| `component`, `section` | a project's components and its titled pages, kept in titled contexts (`section_context`; every page is in one, the migration put the pages that existed in a context called General, and a page written with no context goes in the project's first). The console arranges them in one call (`SectionLayoutService`, a whole arrangement that names every context and page once, applied by `placement_rules` in `rekall-core`) and adds, renames and deletes contexts (`SectionContextService`, a context is deleted only when empty); a session regroups them with `rekall_reorganize_sections` (`ProjectSectionService::reorganize`: every page named once under the contexts it wants, pages move and emptied contexts go, nothing is renamed or rewritten; terminal mode `REORGANIZE`, `/rk project:<label> reorganize`); a page names another as `[[page:Title]]`, by title, so a move never breaks it; a component has a short `summary`, which every context carries, and a structured `description`, a markdown page a session reads with `rekall_read_components`; `ProjectComponentService` (`rekall-core`) takes a session's write and a scan is a session (`/rk project:<label> components`, terminal mode `SCAN_COMPONENTS`) that reads the folder and calls `rekall_component` for each part |
| `adherence` | how closely the work followed the plan: steps accepted at the first pass, steps sent back, deviations and flow visits against their budget, per task and per project; read-only, console and HTTP only |
| `check` | the commands a person sets to pass before a claim is accepted, for a whole project or one step, and what each run found; the console writes them, no session can |
| `decision` | what a person and a session settle about a task before it runs, one record per question: a key, its options, the answer (a changed answer keeps the one it replaced), details added later, the conversation lines behind it, optionally one step; `DecisionService` in `rekall-core` is what every writer calls (a session through `rekall_decision` in `rekall-mcp`, a person editing, adding or removing one by hand through `TaskDecisionService` in `rekall-console`, whose records the context marks as edited by you), the context carries the records in a `<decisions>` block and the task's decisions come before each step's |
| `deviation` | where a task's work departed from its plan, one record per task and step and apart from its notes: a session writes it (`rekall_deviation`, in `rekall-core` and `rekall-mcp`), the console only reads it, and the context carries it in a `<deviations>` block |
| `export` | a project's exports as one PDF or Word file: the row that records each (`project_export`, done or failed with the kind of error) and the file's bytes in a table of its own (`project_export_file`), kept until deleted; `ProjectExportService` (`rekall-console`) writes one with `DocxWriter` or `PdfWriter` from `rekall-export`, records a failed write as a `FAILED` row with its kind (`RENDER` or `STORAGE`) and refuses an invalid request without recording it, lists, serves the file, deletes one and clears a company's, a project's or the failed ones |
| `feedback` | a task's revisions of its description: the text before and after a saved change, aimed at the work (`IMPLEMENTATION`) or at the plan (`PLAN`, the default on a task with steps), each `PENDING` while it waits, `RUNNING` once Run revision took it into a numbered pass, `APPLIED` when a session replies; a session answers open revisions with `FeedbackService::reply` (`rekall-core`), which also gives unanswered running ones back when its session ends, and the context carries the thread in a `<thread>` block; the console's `FeedbackService` (`rekall-console`) records every saved change of an evaluated task's description as a revision, points a waiting one at the work or the plan with `PATCH /api/feedback/{id}`, starts a pass with `POST /api/tasks/{id}/feedback/passes` (refused while a pass runs) and discards a waiting one; a revision is never typed apart from the description |
| `dependency` | a task that waits for another task of its project to be accepted: the pair, the cycle rule, and what a task still waits for as the run queue and the context read it; the console writes the pairs, no session can |
| `connection` | the GitLab instances, GitHub servers, Kubernetes clusters, Jira sites, Confluence sites and Grafana instances projects reach, the companies each is shared with, the projects that bind each, the log of every credential a terminal asked for, the Jira issues a project imports as tasks, the Confluence space a project's description, blueprint and sections are filled from (`ConfluenceSpaces`, the port, `ConfluenceSpaceService` in `rekall-console`, which groups the pages by concept in `confluence_grouping_rules`), the Jira project, Confluence space or GitLab group or project a binding's target is checked against and suggested from (`ConnectionTargetService`, and `GitlabTargetService` over the `GitlabNamespaces` port, implemented by `HttpGitlabNamespaces` in `rekall-infra/src/tracker/`), the task manager a project writes its tasks to (`TaskTracker`, the port, and `TaskTrackerService` in `rekall-core`), and the pull request or merge request of a task's branch on that task manager (`PullRequestHost`, the port, with `GithubPullRequestHost`, `GitlabPullRequestHost` and `ConnectionPullRequestHost`, which picks one by the connection's kind, in `rekall-infra/src/tracker/`) |
| `task` | the task row, and in `rekall-core` and `rekall-mcp` a session creating one, filing it under an epic or replacing its description; the console's task service is in `catalog` |
| `step`, `flow` | the checklist, and the routes and visits that turn it into a flow |
| `graph` | a project's deployment graphs and laying one on a task; console-only, so it has no folder in `rekall-core` |
| `note` | documents, each owned by one task and deleted with it, and how they load (in full or by reference) |
| `wrapup`, `review`, `timeentry` | the wrapup, the review line, logged time |
| `picture` | the pictures a markdown text shows, kept as files in `images/` beside the database (D11): `PicturePath` and the `PictureStore` port in `rekall-core`, `PictureService` in `rekall-console`, which names the folder after the owner (a task, a project's blueprint, a page or a component) and judges the format by the content, and `FolderPictureStore` in `rekall-infra`. Called `picture` so no module shadows the `image` crate; no table behind it, so no folder in `rekall-store` |
| `commit`, `diagram`, `token`, `search`, `context` | logged commits and what they and the working tree changed, stored diagrams, token usage, search, context assembly and its size |
| `context_fill` | the last share of its context window a chat held at the end of a turn, per task and per step (`ContextFillService`, the only writer of `context_fill`; the console reads it) |
| `quick` | the quick task: a task started from one line of text with its session opened at once, accepted without a review when Claude's wrapup lands; console and HTTP only, the acceptance itself is in `review` and `wrapup` |
| `skill`, `tag`, `queue`, `claude` | skill assignments, tags, the run queue, what a session launch needs |
| `codex` | the Codex CLI port for the model catalog, account limits and Rekall setup. The console calls it through its service and HTTP routes. Infrastructure runs the CLI and its App Server. The header draws both provider readings with the same account usage meter. |
| `chat` | the chat view of a session: its events (`ChatEvent`) and the ports a client reaches it through (`ChatPipe`, `ChatListener`), in `rekall-core`, next to `TerminalHost`, the lifecycle port, widened so that a session is no longer taken to be a PTY; in `rekall-infra` the stream parser, the headless `claude` process, `ChatSessionManager` and `SessionHosts`, which routes each call to the terminal or chat host; in `rekall-console` and `rekall-http` the service and the routes that follow and talk to one (opening, listing and closing stay with `terminal`) |
| `backup`, `settings`, `version` | the backups of the open database and restoring one, the registered database folders and the restart onto another, the update check; no table behind them, so no folder in `rekall-store` |
| `guard` | the rules a Claude Code hook applies before a tool runs and before a session stops: which commands and reads are refused, and when a session may not end; decisions over data, no I/O |
| `worktree` | a project's second working folder on a branch of its own, one per task, started from and merged into the project's main branch: the port `WorktreeHost`, the names and the safety rules of `WorktreeLayout`, the porcelain listing and the reclaiming of worktrees a crash left; `rekall-infra` runs git; for a project with `pull_request_flow` on, `PullRequestFlowService` guards the switch and `TaskPullRequestService` pushes the branch (`WorktreeHost::push_branch`), opens the request through `PullRequestHost` and merges it on acceptance, then closes the issue and deletes the remote branch, the only writer of `task_pull_request`; `TaskWorktreeService::requires_branch` decides which tasks live on a branch (a project with a folder and a task manager, never a quick task) and every session that works one, whether the run queue, `TerminalService` or `TaskBranchService::start` (for the desktop's external terminal) makes it first; a merged task is closed with `close_merged`; `rekall-core` only |
| `inventory` | what each project is built from: its dependencies at the version each resolves to |
| `terminal`, `filesystem`, `stream` | the in-app terminals, the folder listing behind the path picker, the event stream the console listens to; console and HTTP only |
| `vulnerability` | the advisories, the projects they reach, the scan that finds them and what it looks for, a session's verdict and the fix task |
| `tracker` | the issue a task was written to in its project's task manager, and why the last write failed: the console creates it for a task imported with its issue, `rekall-core`'s `TaskTrackerService` keeps it on every write, with a fingerprint of the content written, and removes the issue when `TaskDeletedEvent` says its task was deleted (`rekall-store` only) |
| `workingset` | the tasks a person keeps open, in the order they were opened, and the place last seen in each project; saved on the server and read by the console's two-level bar. Only a person opening or closing a task changes it: no session, queue or review does |
| `telemetry` | where Claude Code's OpenTelemetry traces are sent (a self-hosted Langfuse), and the variables a terminal's `claude` starts with when they are on |

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
| `rekall-export` | nothing |
| `rekall-store` | `rekall-kernel` |
| `rekall-core` | `rekall-diagram`, `rekall-kernel`, `rekall-store` |
| `rekall-console` | `rekall-core`, `rekall-export`, `rekall-kernel`, `rekall-store` |
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
| `adherence` | - | - | `deviation`, `flow`, `step`, `task` | nothing | - |
| `backup` | - | nothing | nothing | nothing | - |
| `catalog` | `task` | - | `commit`, `feedback`, `note`, `review`, `step`, `tag`, `task`, `timeentry`, `tracker`, `wrapup` | nothing | - |
| `check` | `catalog`, `step`, `task` | nothing | `catalog`, `step`, `task` | nothing | - |
| `chat` | - | `claude` | `claude` | nothing | - |
| `claude` | - | `catalog`, `connection`, `flow`, `task` | nothing | `skill` | - |
| `codex` | - | nothing | nothing | nothing | - |
| `commit` | `component`, `step`, `task` | `catalog`, `component`, `step`, `task` | - | nothing | nothing |
| `component` | `catalog` | `catalog`, `connection`, `task` | nothing | nothing | nothing |
| `connection` | `catalog`, `task` | `task`, `tracker` | `catalog`, `commit`, `component`, `section`, `task` | `catalog`, `section` | - |
| `context` | - | `catalog`, `commit`, `component`, `connection`, `decision`, `dependency`, `deviation`, `feedback`, `note`, `section`, `step`, `tag`, `task`, `wrapup` | - | nothing | nothing |
| `context_fill` | `step`, `task` | `chat` | - | - | - |
| `decision` | `task` | `deviation`, `step` | `context`, `deviation`, `step` | nothing | nothing |
| `dependency` | `task` | `claude` | `queue`, `task` | nothing | - |
| `deviation` | `task` | `step`, `task` | `task` | nothing | nothing |
| `diagram` | `catalog`, `task` | `catalog`, `task` | nothing | nothing | nothing |
| `export` | `catalog` | - | `catalog`, `component`, `section` | nothing | - |
| `feedback` | `task` | `task` | `step`, `task`, `wrapup` | nothing | nothing |
| `filesystem` | - | - | nothing | nothing | - |
| `flow` | `step`, `task` | `step` | - | `step` | `step` |
| `graph` | `catalog`, `flow`, `step` | - | `catalog`, `flow`, `step`, `task` | `step` | - |
| `guard` | - | nothing | `catalog`, `step`, `task` | nothing | - |
| `inventory` | `catalog` | `catalog` | - | nothing | `vulnerability` |
| `note` | `task` | `task` | nothing | nothing | nothing |
| `picture` | - | nothing | `catalog`, `component`, `section`, `task` | nothing | - |
| `queue` | `task` | `catalog`, `claude`, `dependency`, `task` | nothing | nothing | - |
| `quick` | - | - | `catalog`, `claude`, `terminal` | `catalog`, `claude` | - |
| `review` | - | `step`, `task`, `timeentry` | - | - | - |
| `search` | - | `task` | - | nothing | nothing |
| `section` | `catalog` | `catalog`, `note` | nothing | nothing | `note` |
| `settings` | - | nothing | nothing | nothing | - |
| `skill` | `step`, `tag`, `task` | `claude` | nothing | nothing | - |
| `step` | `task` | `flow`, `task`, `timeentry` | - | nothing | `commit`, `flow` |
| `stream` | - | - | - | nothing | - |
| `tag` | `task` | - | nothing | nothing | - |
| `task` | `catalog`, `note`, `step`, `tag`, `wrapup` | `flow`, `step` | - | - | `flow` |
| `telemetry` | - | `claude`, `connection` | - | nothing | - |
| `terminal` | - | - | `claude`, `worktree` | `claude`, `vulnerability` | - |
| `timeentry` | `task` | `catalog`, `task` | - | nothing | - |
| `token` | `step`, `task` | nothing | - | nothing | - |
| `tracker` | `task` | - | - | - | - |
| `version` | - | nothing | nothing | nothing | - |
| `vulnerability` | `catalog`, `task` | `catalog`, `commit`, `connection`, `inventory` | `catalog`, `step`, `task` | `catalog` | nothing |
| `workingset` | `catalog`, `task` | - | nothing | nothing | - |
| `worktree` | - | `claude`, `commit`, `connection`, `task`, `tracker` | nothing | nothing | - |
| `wrapup` | `task` | `catalog`, `commit`, `review`, `task` | - | nothing | `commit` |

Each crate's foundations are modules every domain may use and are no domain: `ctx`, `events`,
`load`, `project_ref`, `services`, `text_rule` and `tx` in `rekall-core`; `constraints`, `enums`, `prelude`,
`slug`, `stamped` and `task_key` in `rekall-store`, with `database`, `migration`, `query_support`,
`stamped_update` and `repository`, whose re-exports the test reads back to their domains; `shared` in `rekall-console`; `api_ports`, `api_state`, `router` and `shared` in `rekall-http`; `controller`, `protocol`, `texts`,
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
| Something that knows nothing of Rekall | a crate of its own, depending on no Rekall crate | `rekall-diagram`, `rekall-export` |
| A capability that touches the machine: a process, a file, a socket, outbound HTTP | a port: the trait in a `rekall-core` domain, a thin service over it in `rekall-console`, the implementation in a `rekall-infra` folder, injected by `rekall-app` through `ApiPorts` | the terminals and chat sessions, the backups, the database registry, the release feed, the OS credential store |

- **A rule that needs only data never goes in `rekall-infra`.** One that reads the database is a
  `rekall-core` service (`RunQueueService`, `ConnectionCheckService`); one only the console needs is a
  `rekall-console` service over the port (`DatabaseFolderService` decides which database is active,
  `UpdateCheckService` when the release feed is asked again).
- **A port with one implementation still has a second: the fake in its console service's unit test.**
  That is the seam. Where an end-to-end test needs to swap it too, `StartOptions` carries it.
- **A port that only `rekall-infra` drives is wired in `rekall-app`, not in `ApiPorts`.** `PullRequestHost` is
  the trait in `rekall-core`'s `connection` domain; `TaskPullRequestService` in `rekall-core`'s `worktree`
  domain holds every rule on it (when to push and open, what a refusal or a conflict means) and
  `WorktreeCleaner` in `rekall-infra` calls the service on each step and review move. `rekall-app` builds
  `ConnectionPullRequestHost` and hands it to the service; its fake is `ScriptedPullRequestHost`.
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
| Document editing | `md-editor-v3` with a CodeMirror live preview | Toolbar over the markdown source, not a WYSIWYG. A WYSIWYG keeps its own tree and regenerates the source on every edit, and that source is what MCP hands to Claude verbatim. The live preview (`rekall-ui/src/common/live-preview/`) only decorates the source: it hides markup off the caret's line and draws tables, code and diagrams as widgets (a mermaid fence is drawn like a Diagram sheet: the console's own components for sequence, flowchart, state, class and ER, Mermaid in the console theme for the rest; the read view draws the same way, from a frame a markdown-it rule leaves in place of the fence), and a table edit rewrites that cell or that table alone |
| Tests | `cargo test`; end-to-end suites on a real port and a file database | |
| Desktop | Tauri v2 | The same server in-process, a WebView window, three native bridges |

---

## 4. Model

```
Company ──< Project ──< Task ──< Document
                         │
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
| `project` | `label`, unique per company | `task_key` (two to four capital letters, unique across every company: the first half of its tasks' codes; migration `086-task-code`), `title`, `status`, `description`, `company_id`, `main_branch` (the branch a task's branch starts from and is merged into, null for the folder's current one; migration `075-task-manager`), `pull_request_flow` (whether accepted work reaches the main branch through a pull request, off by default; migration `081-pull-request-flow`) |
| `task` | `code`, unique across every project | `number` (its place in its project, counted from 1: one past the highest the project holds), `label` (an internal slug, unique per project: the name of its branch and worktree, and what an anchor written before codes existed still names; migration `086-task-code`), `title`, `status`, `description`, `project_id`, `graph_laid_at` (when a deployment graph was laid on it), `release_skipped` (the project's release graph is not laid when the console runs it; migration `054-release-on-run`), `kind` (`TASK` or `EPIC`), `epic_id` (the epic of the same project it belongs to, set null when the epic is deleted; migration `055-epic`), `quick` (started from one line of text; at most one still open or running per project), `auto_accepted` (accepted by Rekall when the wrapup landed, not by a person; cleared when the review line moves again; migration `063-quick-task`) |
| `task_pull_request` | through its task | `task_id` (the key: one row per task of a project with the pull request flow on, once its work was first claimed), `number` and `url` (the pull request, or GitLab merge request, open for its branch), `failure` (why it was not opened or not merged, cleared by the next move that works), `merged_at` (when the host merged it); deleted with its task (migrations `082-task-pull-request` and `083-pull-request-merged`) |
| `task_tracker_link` | through its task | `task_id` (the key: one row per task that was ever pushed to its project's task manager, none for one that was not), `issue_key` and `issue_url` (the issue the task manager holds for it), `sync_error` (why the last write there failed, cleared by the next that works), `synced_fingerprint` (a fingerprint of the title and description last written, which says whether the task changed since); deleted with its task (migrations `075-task-manager`, `080-task-tracker-link` and `085-task-tracker-synced-content`) |
| `document` | through its task | `task_id`, `position` (its place on the task, dense from zero), `title`, `kind`, `body_markdown`, `context_mode`; deleted with its task (migration `084-note-owned-by-task`) |
| `task_step` | through its task | `task_id`, `title`, `body_markdown`, `done`, `done_at`, `position`, `flow_x`/`flow_y` (where it was dragged on the flow canvas, null for the automatic layout), `max_visits` (how many times a flow may enter it, null for no limit), `starts_session` (whether it opens a fresh Claude session instead of joining the one the step before it ran in), `connection_id` (the company connection it reaches, set when a deployment graph laid it; null when the connection is deleted) |
| `step_link` | through its task | `task_id`, `from_step_id`, `to_step_id` (null: the route ends the flow), `outcome` unique per `from_step_id`, `position`; deleted with either step |
| `step_visit` | through its task | `task_id`, `step_id`, `ordinal` unique per task, `entered_at`, `left_at`, `outcome`, `report` (what the claim that closed it reported; migration `053-visit-report`); deleted with its step |
| `step_snapshot` | through its task | `task_id`, `step_id` (null for a task with no steps), `taken_at`; at most one per step and one per task, replaced each time the work goes from open to running on an auto-commit project; deleted with its task or step (migration `060-step-snapshot`) |
| `step_snapshot_file` | through its snapshot | `snapshot_id`, `path` (relative to the repository root), `content_hash` (git's hash of the file when the snapshot was taken, null for a file already deleted); a snapshot with no file rows is a clean tree, not a missing snapshot |
| `commit_reference` | through its task | `task_id`, `step_id` (set null when the step is deleted, so the commit stays on its task; migration `076-commit-reference-step-kept`), `commit_hash`, `comment`, `diff`, `in_context`; indexed by task and by step; deleted with its task |
| `wrapup` | through its task | `task_id` unique, `body_markdown`, `written_by` |
| `project_component` | through its project | `project_id`, `name`, `folder`, `description`, `position` |
| `project_export` | through its project | `project_id`, `format` (`PDF`, `DOCX`), `status` (`DONE`, `FAILED`), `summary` (what was exported, in a line), `file_name`, `size_bytes`, `diagrams_missing` (the diagrams the file holds an error paragraph for), `failure_kind` (`RENDER`, `STORAGE`) and `failure_message` (set exactly when it failed), `created_at`; migration `m095_project_export`; deleted with its project |
| `project_export_file` | through its export | `export_id` (the key), `bytes`; a table of its own so a listing never reads a file; deleted with its export |
| `done_check` | through its project | `project_id`, `step_id` (null: the check runs for every step of the project), `label`, `command`, `folder` (relative to the project folder, null for the folder itself), `timeout_seconds` (1 to 3600), `position` within its scope (the project's checks, or one step's); deleted with its project or step (migration `061-done-check`; not `check`, a SQL keyword). Written by the console only |
| `done_check_result` | through its check | `check_id`, `task_id`, `step_id` (null for a task with no steps), `passed`, `exit_code` (null when it timed out), `output_tail`, `ran_at`, `duration_ms`; deleted with its check, task or step. Written by `CheckExecutionService` in `rekall-core`, one row per run, the failing run included |
| `deployment_graph` | through its project | `project_id`, `title` (unique per project, ignoring case), `description`, `release_default` (laid on every task the console runs; at most one per project); deleted with its project |
| `graph_step` | through its graph | `graph_id`, `position` (0 is where the flow begins), `title`, `body_markdown`, `connection_id` (null when it reaches nothing or the connection is deleted), `max_visits`, `flow_x`/`flow_y` (where it was dragged on the graph's canvas, null for the automatic layout; migration `050-graph-step-place`); rewritten whole with its graph |
| `graph_route` | through its graph | `graph_id`, `from_step_id`, `to_step_id` (null: the route ends the flow), `outcome` unique per `from_step_id`, `position`; deleted with either step |
| `external_connection` | `label`, unique across the workspace | `title`, `kind` (`GITLAB`, `GITHUB`, `KUBERNETES`, `JIRA`, `CONFLUENCE`, `GRAFANA`), `base_url` (the GitLab instance, GitHub server, Jira site, Confluence site or Grafana instance, or the server read from an imported kubeconfig), `kube_context`, `proxy_url` (Jira and Grafana only: the proxy its calls go through), `accept_self_signed` (Grafana only: its certificate is not verified; migration `072-connection-self-signed`), `root_path` (GitLab and GitHub only: the group, namespace or owner its credential covers, without a slash at either end; migration `059-shared-connection`), `secret_ref` (the OS keychain account holding the credential, generated, unique), `secret_set_at` (null while no credential is stored). It belongs to no company: `connection_company` says who it is shared with. No column holds a credential |
| `connection_company` | the pair `connection_id`, `company_id` | The companies a connection is shared with; a project can bind a connection only when its company is one of them. Deleted with either side, so deleting a company unshares its connections and keeps them |
| `project_connection` | `label`, unique per project | `project_id`, `connection_id`, `target` (GitLab project path, Kubernetes namespace, Jira project key, Confluence space key or Grafana folder UID; a Jira or Confluence target is checked against the site before it is kept). `task_manager` (the project's one task manager, at most one binding per project; migration `075-task-manager`). A project may bind one connection several times, one target each; the label starts as the connection's and is what a session names the binding by (migration `051-binding-label`). Deleted with either side |
| `connection_access` | through its connection | `connection_id`, `task_id` (set null when the task goes), `terminal_id`, `accessed_at`, `granted`, `reason` (why the request was refused); append-only, never a credential |
| `project_dependency` | through its project | `project_id`, `ecosystem`, `name`, `version`, `manifest` (the declaring file, relative to the project's folder), `created_at`; unique per project, ecosystem, name and version; replaced whole |
| `security_advisory` | `ghsa_id`, unique | `cve_id`, `summary`, `description`, `url`, `published_at`, `severity` (GitHub's, upper-cased; `CRITICAL` on advisories recorded before it), `malware`, `affected_json` (the packages it affects, one entry per release line: ecosystem, name, vulnerable range, first patched version, vulnerable functions) |
| `vulnerability_exposure` | through its project | `project_id`, `advisory_id`, `ecosystem`, `package_name` and `package_version` unique together (one exposure per package version an advisory reaches), `verdict` (`AFFECTED`, `NOT_AFFECTED`, null until judged), `reasoning`, `conclusion` (why the project is or is not affected, in plain words; null on verdicts given before it), `assessed_at`, `assessed_commit` (the project's `HEAD` at the verdict), `fix_task_id` (set null when the task goes); deleted with its project or advisory |
| `scan_scope` | one row | `include_high`, `include_malware`; absent until first changed, which reads as both off |
| `working_set_task` | `task_id` | `position` (from zero, kept without gaps; the tasks a person keeps open, in the order they were opened). Deleted with its task |
| `working_set_place` | `project_id` | `task_id`, the task last seen in the project, or empty for its task list; set to empty when that task is deleted. Deleted with its project |
| `telemetry_setting` | one row | `enabled`, `host` (the Langfuse address), `public_key`, `log_content` (whether prompt and command text travel with the traces); absent until first saved, which reads as off. The secret key is in the credential store under `langfuse-secret-key`, never in the row |
| `token_usage` | through its task | `session_id`, `task_id`, `step_id`, the four token counts, `spent_on` |
| `transcript_scan` | `session_id` | `scanned_bytes`, `scanned_at` |
| `task_skill` | through its task | `task_id`, `step_id` (null for the task's own), `skill_name`, `arguments` (optional), `position`; deleted with its task or step |
| `tag_skill` | through its tag | `tag_id`, `skill_name`, `arguments` (optional), `position`; deleted with its tag |
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
`project:website` that matches twice is reported as ambiguous, with each candidate written as
`company:<name> project:<label>`. Every tool that takes a project alone (`rekall_create_task`,
`rekall_search`, `rekall_inventory`, `rekall_vulnerabilities`, `rekall_assess_vulnerability`,
`rekall_diagram`, `rekall_read_sections`) accepts that pair as its anchors, and `ProjectRef` in
`rekall-core` carries it down to `load::resolve_project`. A task is named by its globally unique code,
so a task anchor never needs the company.

A note belongs to **exactly one task** and is deleted with it. Cluster access, a naming
convention and an onboarding step are all notes of that kind, written on the task that needs them.

`task_id` is a foreign key with `ON DELETE CASCADE`, so a note cannot outlive its task or point at
a deleted one. `position` is the note's place on its task, kept dense when a note is deleted.

Migration `084-note-owned-by-task` moved notes that were shared: one on several tasks became one
copy per task, the first task it was attached to keeping the original id, and one on no task was
deleted, since nothing could show it any more.

A task is named by its **code**: its project's key, a dash and its number, `RK-12`. The key is
unique across every company, so a code names one task anywhere and `task:RK-12` needs no project
beside it. The number is one past the highest the project holds, so the number of the newest task can come back once that task is deleted or moved. A task that moves to another project takes
the next number there. Changing a project's key writes every code of the project again in the same
transaction. Codes are compared ignoring case.

The `label` column stays for what needs a stable slug: a new task's label is its code in lowercase, and
it is the name of its git branch and worktree folder. A `project:<label> task:<label>` anchor written
before codes existed still resolves, and so does a `[[task:<label>]]` reference; nothing writes either
any more. An imported Jira issue keeps its issue key as its label, which is how a second import
recognises it.

Migration `086-task-code` gave every project a key suggested from its label (the initials of its
words, then its first letters, then a letter after them, so no two share one) and numbered each
project's tasks from 1 in the order they were created.

### 4.1 The wrapup

A wrapup is what a task's implementation looks like **now**. Claude writes one at the end of a
session with `/rk task:VG-12 wrapup`, and reads it back at the start of the
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
| Replaced whole, never appended to; each text kept as a version beside it | A wrapup that accumulated would become the log it is defined against, so the row is one text. `wrapup_version` keeps the last 20 texts per task (`number`, author, the project's `HEAD` when written), written in the transaction that replaces the row, so a rewrite that goes wrong or overwrites a hand edit can be read and restored from the task page. A session never reads them: `rekall_context` carries only the current text, since an old one would be taken for a fact |
| Capped at 20,000 characters, against 100,000 for a note | The cap is where a wrapup that has turned into a log gets caught. If the state of the work does not fit on a screen, what is being written is the process |
| `written_by` is `CLAUDE` or `HAND`, stamped by the transport | The console cannot claim to be Claude and the tool cannot claim to be you: the API stamps `HAND` on anything arriving over HTTP and the tool stamps `CLAUDE` on anything arriving over MCP. A field a client could set is a field that says nothing |
| `ON DELETE CASCADE` on the task | A wrapup describes one task and has no meaning anywhere else |

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

### 4.6 Exports

A project's blueprint, ticked pages and ticked components become one PDF or Word file (`rekall-export`): the
blueprint first, then pages by context and position, then components by position, each opening with its
title as a level-1 heading and the headings inside it moved one level down. `ProjectExportService` writes
the file in `spawn_blocking`, stores it with its row, and on a failed write or insert records a `FAILED` row with
its kind and returns an error whose message names the kind; a request refused as invalid (nothing ticked, a blank
blueprint alone, an item of another project) is not recorded. Diagrams are decision D9.

| Route | Does |
|---|---|
| `POST /api/projects/{id}/exports` | Makes an export from `format` (`pdf`, `docx`), `blueprint`, `sectionIds`, `componentIds` and `diagrams` (`source` and base64 `png`); answers `201` with the entry. A 64 MB body limit on this route only |
| `GET /api/exports` | Every export, newest first, with its project and company |
| `GET /api/exports/{id}/file` | The bytes as an attachment with the format's content type; a failed or unknown export is `404` |
| `DELETE /api/exports/{id}` | Deletes the export and its file, `204` |
| `DELETE /api/exports?companyId&projectId&status` | Deletes every export the filters name, answers `{ "deleted": n }` |

The console lists them on `/exports`, a Library screen, grouped by company then project; filters and search run
over the loaded list and have no route.

### 4.7 Pictures

A picture is a file, not a row (D11). `PictureService` copies the chosen file into its owner's folder,
`<company>/<project>/` followed by the task's code, `blueprint`, `pages/<title>` or `components/<name>`, and
names it `<file name>-<hash>.<format>`. A name that leaves no slug falls back to the start of the record's id,
a file's to `image`. A file already there is written again, so its modification time says when it was last
asked for.

| Route | Does |
|---|---|
| `POST /api/images` | Copies the file at `sourcePath` for `owner` (`kind` `TASK`, `BLUEPRINT`, `PAGE` or `COMPONENT`, and `id`; a blueprint is named by its project's id) and answers `201` with its `url`. Not a picture, past 10 MB, not a file or a relative path is `400`, an unknown owner `404`, a database that is not a file `409` |
| `GET /api/images/{path}` | The bytes with their content type, cacheable for good, `nosniff` and same-origin only; an SVG carries a sandboxing `Content-Security-Policy`, so one opened directly runs no script. Anything that is not a picture's path is `404` |
| `GET /api/image-thumbnail?path=` | A preview of the file at the absolute `path`, for the picker, under the content check and 10 MB limit of an import, else `400`; nothing is copied. PNG, JPEG, WebP and a GIF's first frame come back as a PNG fitting 320 pixels, never enlarged and turned as their EXIF orientation says; an SVG as it is. The headers of `GET /api/images/{path}`, so no other page can embed it and probe your files, but `Cache-Control: private, no-cache` |

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
| Assigned skills (`task_skill`, `tag_skill`) | A `<skills>` block after the description: the task's, its tag's, then those of the steps still to be worked, each with its scope | Short (20 per owner) and invoked by name, so only the name and its arguments travel |

Skills follow three rules. A tag's skills are read when the context loads and never copied onto its
tasks, so editing a tag changes every task under it. An invocation, a skill's name and its arguments
together, already listed at a wider scope is not listed again, so a step that assigns the same skill
with other arguments is listed as its own, and a finished step's skills are dropped. The arguments are
free text, trimmed and capped at 300 characters (`rekall_store::skill_arguments`), one string per skill
per owner: Rekall does not model a skill's grammar, and the context prints them as `args: "..."` for
the session to pass to the Skill tool. The name is checked for its shape only
(`rekall_store::skill_name`): `rekall-console` cannot see `~/.claude`, which `rekall-infra` reads, so the
picker offers only what exists and a skill that later vanishes stays assigned and is flagged in the
console. The writes are `SkillAssignmentService` in `rekall-console`, each replacing one owner's whole
list; `rekall-mcp` can read the skills through the context and never write them.

A project's connections arrive as a `<connections>` block after its components, one line per
binding: the binding's own label, the connection's title and how to reach it. A cluster is `kubectl --context <label>` with the
binding's namespace; a GitLab instance is its URL and project path, pushed to over HTTPS through the
configured credential helper and called through `rekall-gitlab --connection <label>`; a GitHub server
is its URL and `owner/repo`, pushed to the same way and called through `rekall-github --connection
<label>`, which is how a session files an issue or opens a pull request; a Jira site is its
URL and project key, called through `rekall-jira --connection <label>` under `/rest/api/2`, which Jira Cloud
and Data Center both serve, its search named for each; a Grafana instance is its URL and, when the binding names one, its folder UID, called through `rekall-grafana --connection <label>` under `/api`. A connection with no credential
says so, so the session reports it. `ConnectionView` carries no field that could hold a credential,
the keychain account, a cluster's server or its CA. The helpers work in local Claude Code and Codex
terminals opened by Rekall. The context and the provider's Rekall command both direct the session to
use the helpers. A step laid from a deployment graph names the connection it works through on a
`Connection:` line by the labels of the
project's bindings of it (`a` or `b` when it binds it twice; see 4.4 in [design/workflow.md](design/workflow.md)), and says to report one the
project no longer binds.

A project's sections arrive as a `<sections>` block after its blueprint: their titles only, in the
console's order, with a line telling the session to read the one the work touches through
`rekall_read_sections` and follow it. The bodies stay out because a project can hold many long pages
and only one matters to a given piece of work; the titles stay in because a page nobody knows exists
is never read, and a `Contributing` page is how a project's procedure for an issue, a branch and a
pull request reaches every task.

Scalars are rendered as bullets and bodies as tags, which is why a description is a `<description>` block rather than a `description` bullet. It is a document, with headings, lists and a scope section, and a document inlined into a list item stops being one: every line after the first falls outside the bullet, and its own headings outrank the record's.

That distinction is why the two-anchor form exists. `/rk project:vega` loads the project and
lists its tasks as anchors; naming the task as a second anchor is how you narrow to one.

A note arrives with the task it belongs to, in full or, when the person marked it so, as a title,
a line and an anchor the session loads on request. Knowledge several tasks need belongs in the
project's sections or components, which are shared by construction, not in a note.

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
| [design/console-ui.md](design/console-ui.md) | The console structure (project tasks, task page, Review), the report, design tokens, the header |
