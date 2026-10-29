# Rekall

Rekall stores your projects, tasks and markdown notes, and loads one task in full into a Claude Code session on a single command.

Projects, tasks and notes live in a local app. `/rk project:vega task:report-builder` loads that task, its project, its checklist and every attached note. `/rk project:vega task:report-builder wrapup` records what the implementation is now, so the next session starts from the current state instead of reading the code back.

Source-available, not open source. See [License](#license).

## Requirements

| Tool  | Version |
|-------|---------|
| Rust  | 1.85+   |
| Node  | 22+     |
| pnpm  | 9+      |

The backend is one Rust binary, `rekall-server`, serving the console, the REST API and the MCP endpoint on one port. The database is a SQLite file (`rekall.db`) in a folder you choose on first run. No database server, Docker or cluster.

## Run

```bash
make run     # build the frontend, then start on http://localhost:47355
make build   # the above, as target/release/rekall-server, without starting it
make start   # start the binary `make build` left, no rebuild
make ui      # the frontend alone
```

`make run` and `make build` build the UI first. The frontend compiles to `rekall-ui/dist` (git-ignored) and `rekall-app` embeds it into the binary at compile time, so `rekall-server` is the only file that has to travel. `rekall.ui.dist=<folder>` serves the console from a folder instead.

| Service | Address                      |
|---------|------------------------------|
| UI      | `http://localhost:47355`     |
| MCP     | `http://localhost:47355/mcp` |

Port 47355 is fixed because the MCP endpoint is registered with Claude Code by URL, so a clash breaks the registration rather than moving the app. `SERVER_PORT` overrides it for the server, the desktop app and the tests.

### This machine only

Nothing in Rekall authenticates, and its API can open a `claude` terminal in a project folder, so the local-access guard (`rekall-app/src/security.rs`) answers only this machine. It refuses with a 403 a peer that is not loopback (another machine on the same network), a `Host` header that is not `localhost`, `127.0.0.1` or `[::1]` (a page using DNS rebinding), and an `Origin` header from another site (a page open in the same browser). The terminal's WebSocket accepts only loopback origins too. Claude Code and `curl` send no `Origin` and pass.

| Property | Default | Meaning |
|---|---|---|
| `rekall.security.remote-access` | `false` | `true` accepts other machines and their own origin. It gives up the DNS-rebinding check; only for a network you trust |

## macOS application

A disk image for Apple Silicon is published with every `v*` tag pushed on a commit of `main`.

| | |
|---|---|
| Download | [Rekall-macos-arm64.dmg](https://github.com/ValerioMC/rekall/releases/latest/download/Rekall-macos-arm64.dmg) |
| Needs | macOS 13 or later, Apple Silicon |
| Install | Drag Rekall onto Applications, then run the `xattr` command below once |

```bash
xattr -dr com.apple.quarantine /Applications/Rekall.app
```

The app is a Tauri v2 window over the same server `rekall-server` runs, on the same port, config and MCP endpoint. It checks for a newer release when it starts and installs it in place. `make dmg`, `make app` and `make install` build it locally. The release process, update flow, splash, quit guard and icons are in [docs/guide/desktop.md](docs/guide/desktop.md).

## Connect Claude Code

**Settings > Claude Code** in the app registers the MCP server for every folder and installs the `/rk` command in one click. It also repairs a registration that points at the wrong port, carries an older command, or is shadowed in one folder by a registration made without `--scope user`. While a registration is out of date, a warning triangle sits on the Settings key in the top bar until Repair is run.

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

**Open in terminal**, on a task, hands the session to your own terminal app in the project's folder with `/rk` already running. Set the folder in the **Folder** field of the project page's **Settings** tab. The terminal is iTerm2 when installed, Terminal.app otherwise. A switch in **Settings > Claude Code** adds `--dangerously-skip-permissions`; it is stored on the machine, not in the database. This button works only inside the desktop app. **Run here** runs a real terminal inside the app instead, see [Terminals](docs/guide/sessions.md).

## Anchor syntax

An anchor is `entity:value`, where the entity is `company`, `project` or `task` and the value is the record's **label** (its `name` for a company). Label rules are in [Data](docs/guide/data.md#model).

| Form | Meaning |
|------|---------|
| `/rk project:vega task:report-builder` | The project and that task, both in full |
| `/rk project:vega` | The project, plus its tasks as a list of anchors |
| `/rk vega report-builder` | Positional. Works while each term matches one record |
| `/rk task:"report builder"` | Quote a value containing spaces |
| `/rk … wrapup` | Write the task's wrapup instead of loading it |
| `/rk … note "what to keep"` | Write a note on the task, then stop |
| `/rk … section "Deployment" "how we ship"` | Write a titled section on the task's project, or rewrite the one with that title |
| `/rk … plan` | Propose the task's checklist as drafts, then stop; `plan flow` proposes a flow |
| `/rk … step:3 start` / `step:3 done` | Move a step to running / claimed; on a flow `done outcome:<name>` follows that route |
| `/rk project:vega generate kinds:sequence,state "how checkout charges"` | Store a diagram answering the request, one sheet per type |
| `/rk project:vega explain "how an order is placed"` | Store a walkthrough the console plays step by step |
| `/rk project:vega inventory` | Record every dependency at its exact version |
| `/rk project:vega assess advisory:GHSA-xxxx-xxxx-xxxx` | Record whether that advisory's vulnerable code is really reached; without `advisory:`, every one not assessed yet |
| `/rk note:3f2a9c1e` | A note sent by reference, loaded in full |

An anchor brings back the record, everything it references resolved in full with their notes, what references it as anchors, and its own markdown. A note can be attached to several tasks and arrives with each. If a bare term matches more than one record, the candidates come back and nothing loads. `project:` disambiguates a label two projects share.

## MCP tools

| Tool | Access | Effect |
|------|--------|--------|
| `rekall_context` | read | Resolve anchors, return markdown |
| `rekall_wrapup` | write | Replace one task's wrapup |
| `rekall_step` | write | Move one step: `open` to `running` to `claimed`; on a flow, a claim's `outcome` follows that route and its `report` reaches the step it leads to |
| `rekall_record_commit` | write | Log a commit of the project's repo folder against one task or step |
| `rekall_propose_step` | write | Add one step as a draft, for a person to promote; `new_session` makes it open a fresh Claude session |
| `rekall_propose_flow` | write | Add a whole flow to a task with no steps: its steps as drafts and the routes between them |
| `rekall_create_task` | write | Create a task or an epic in a project with a description and up to 20 draft steps (none on an epic), optionally filed under an epic |
| `rekall_join_epic` | write | File an existing task under an epic of its project, or move it from another epic |
| `rekall_note` | write | Write a note on one task, or rewrite one it carries by title with `replace` |
| `rekall_project_section` | write | Write a titled markdown section on a task's project, or rewrite one by title with `replace` |
| `rekall_read_sections` | read | List a project's sections by title, or read one whole with `title` |
| `rekall_diagram` | write | Store Mermaid sheets generated for a project or one of its tasks, or replace a diagram by id |
| `rekall_inventory` | write | Replace a project's dependency inventory whole |
| `rekall_vulnerabilities` | read | List the advisories that reach a project, or read one whole with `advisory` |
| `rekall_assess_vulnerability` | write | Record whether an advisory really reaches a project: `affected` or `not_affected`, with reasoning and a conclusion |

There is no query, get or schema tool. Nothing a session creates is work until you promote it: proposed steps and flows land as drafts, and a created task starts `TODO`. `rekall_step` refuses `done`; that state is set by hand in the console. `rekall_note` and `rekall_project_section` refuse a title already used unless `replace: true`, and neither can rename, reorder or delete. `rekall_step` (on `claimed`) and `rekall_wrapup` take an optional `commit_message`, used only on a project that commits on claim. Every parameter is in [docs/guide/mcp-tools.md](docs/guide/mcp-tools.md), the reasons in [docs/design/mcp.md](docs/design/mcp.md) and [docs/design/write-safety.md](docs/design/write-safety.md).

## What Rekall does

The guides describe each area as it behaves today, with its REST routes and properties.

| Area | What it covers | Guide |
|---|---|---|
| Wrapup | One state per task, rewritten whole each session; notes a session writes | [workflow](docs/guide/workflow.md#wrapup) |
| Steps and flows | Draft, open, running, claimed, done; routes, visit budgets, sessions that open a fresh Claude session; planning; send back | [workflow](docs/guide/workflow.md#steps) |
| Deployment graphs | A project's route from commit to running app, laid on a task | [workflow](docs/guide/workflow.md#deployment-graphs) |
| Epics | A task that gathers other tasks and passes its brief to them | [workflow](docs/guide/workflow.md#epics) |
| Commits | A ledger of commits per task or step; commit on claim | [workflow](docs/guide/workflow.md#commits) |
| Review | Accept or send back a claim; the review queue and its notifications | [workflow](docs/guide/workflow.md#description-review) |
| Terminals | A real `claude` in a pseudo-terminal inside the app; the run queue; the usage meter | [sessions](docs/guide/sessions.md) |
| Token usage | Tokens Claude Code generated per task and step, read from its transcripts | [sessions](docs/guide/sessions.md#token-usage) |
| Projects | Page, sections, components, icons; company connections to GitLab, Kubernetes and Jira | [projects](docs/guide/projects.md) |
| Console | Search, notes and their scopes, tags, skills, the markdown editor, timers | [console](docs/guide/console.md) |
| Report | Hours per client over a week or a month | [console](docs/guide/console.md#report) |
| Claude resources | Edit `~/.claude`: instructions, skills, commands, agents, memories, memory review | [claude-resources](docs/guide/claude-resources.md) |
| Diagrams | Mermaid diagrams linked to code, and walkthroughs the console plays | [diagrams](docs/guide/diagrams.md) |
| Security | Dependency inventory, critical advisories, assessments, fix tasks | [security](docs/guide/security.md) |
| Data | The model, context size, notes by reference, backups, older databases | [data](docs/guide/data.md) |

## Configuration

Properties are passed as `--name=value` arguments or as environment variables in relaxed form (`rekall.terminal.max-sessions` is `REKALL_TERMINAL_MAXSESSIONS`). All optional. Each guide lists the properties of its area (terminals, run queue, usage, updates, backups, transcripts).

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_PORT` | `47355` | HTTP port for the UI, the API and MCP |
| `REKALL_HOME` | `~/.rekall` | Where `config.json` records the database folder |
| `REKALL_DATABASE_URL` (`rekall.database.url`, or `REKALL_DB_URL`) | the folder in `config.json` | `sqlite:<path>` to a database file, or `sqlite::memory:`, overriding the setup wizard's choice. `spring.datasource.url` is still read, with a deprecation warning |

Notes are stored in plain text in the database file. Credentials kept in them are only as protected as the disk is. A connection's credential is never in the database: it goes to the operating system's credential store.

## Develop

```bash
make ui-dev   # Vite dev server on :5173, proxying /api and /mcp to :47355
make test     # every crate's tests, then frontend lint, types and unit tests
make lint     # clippy, as CI runs it
```

`RUST_LOG=rekall=debug make run` turns on debug logging. The `/rk` command the app installs is `.claude/commands/rk.md`, compiled into the binary. The frontend stack and its folders are in [docs/guide/development.md](docs/guide/development.md).

## Modules

A Cargo workspace, one crate per layer: `rekall-kernel <- rekall-store <- rekall-core <- rekall-console <- rekall-http`, with `rekall-mcp` and `rekall-infra` beside them on `rekall-core` and `rekall-app` wiring everything. [docs/DESIGN.md](docs/DESIGN.md#3-module-structure) says where a feature goes and how to add a field or a capability.

```
rekall-kernel/     RekallError, Id, Instant, the string helpers and logged best-effort cleanup every layer shares
rekall-store/      SeaORM entities and their state rules, the queries over them and the SQLite migrations, one folder per domain
rekall-diagram/    the diagram format: Mermaid sheets and their code links, JSON form and validation. No database, no UI
rekall-core/       business logic and every port: context assembly, the step and review lines, flows (routes and visits), wrapups, notes, time entries, token usage, vulnerabilities
rekall-console/    the services only the console calls, commands in and views out with no HTTP types: the catalog, notes, tags, connections, graphs, the folder listing, and the thin services over core's ports (terminals, run queue, backups, database folders, update check)
rekall-http/       Axum REST API for the UI, one folder per domain: controllers, request and response shapes, and the step event stream; `shared` holds the error body, request extractors and response helpers
rekall-mcp/        MCP server, on rekall-core and never rekall-http: three tools read, twelve write
rekall-infra/      everything with a process, a file, a socket or outbound HTTP behind it, each implementing a port of rekall-core and serving no route: the in-app terminal (portable-pty), the Claude Code usage meter, the run queue's worker, the transcript reader, the editor for Claude Code's own files (`~/.claude`), the database registry, backups and restores, the restart handle, the release feed, the OS credential store and the HTTP clients
rekall-app/        rekall-server: the local-access guard, the console bundle, the supervisor; builds the infrastructure, injects it and serves everything
rekall-app/desktop the Tauri desktop app over the same server
rekall-ui/         Vue 3 + Vite frontend
```

Inside a crate the code is laid out by package: a folder per domain (`note/`, `step/`, `catalog/`), the same names in every layer crate, one public type per file named after it (`NoteService` in `note_service.rs`), and a `mod.rs` that only declares the files and re-exports their types. The rules and their exceptions are in [docs/DESIGN.md](docs/DESIGN.md#layout-inside-a-crate).

The crate boundary is what keeps the MCP server off the write paths: `CompanyService`, `ProjectService`, `TaskService` and `DocumentService` live in `rekall-console` and not in `rekall-core` because they write the catalog, and `rekall-mcp` cannot reach them. Read-only services (search, context size) and the narrow writes a session is allowed are in `rekall-core`.

## Run tests

```bash
cargo test                                            # every crate but the desktop app
cd rekall-ui && pnpm lint && pnpm typecheck && pnpm test
```

Each crate keeps its tests under `tests/`: `tests/unit/` mirrors `src/` (`rekall-core/src/search/service/search_service.rs` is tested by `rekall-core/tests/unit/search/service/search_service_tests.rs`, attached with `#[cfg(test)] #[path = "…"] mod tests;` so it can test private functions) and `tests/integration/` holds the integration suites. A new integration suite has to be declared as a `[[test]]` in the crate's `Cargo.toml`, or it never runs. The end-to-end suites in `rekall-app/tests/integration` start the whole application on a real port with a file database and drive the HTTP API, the MCP endpoint and the event stream; a stub stands in for the `claude` TUI in the terminal and run-queue tests.

`.github/workflows/check.yml` runs all of it on every pull request and every push to a branch other than `main`: clippy and `cargo test`, eslint, vue-tsc, vitest, a production build of the UI, and a build of the desktop app. `release.yml` runs only on a `v*` tag on `main`, and builds and publishes that release.

## Design

- [docs/DESIGN.md](docs/DESIGN.md) records the decisions and the reasoning, including the ones that were reversed and why, and points to the design notes by topic in `docs/design/`.
- [CLAUDE.md](CLAUDE.md) holds the code rules every change follows.
- [docs/SPECIFICATION.md](docs/SPECIFICATION.md) describes what the application is and does, entities and rules only.
- [docs/DIAGRAMS.md](docs/DIAGRAMS.md) is the diagram format.
- [docs/MEMORY.md](docs/MEMORY.md) is the memory soak of the console: what was measured, how, and why the numbers say there is no leak.

## License

Source-available, not open source. Licensed under the
[PolyForm Internal Use License 1.0.0](LICENSE): you may read the source, run the
software for your own and your company's internal business operations, and change it
for those purposes. You may not distribute it, in original or modified form.

Copyright 2026 Valerio Mario Casale.
