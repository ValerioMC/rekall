# Rekall — project rules

Rules every change to this repository follows. The crate layout and the file-per-type
convention are in `docs/DESIGN.md` §3 ("Layout inside a crate") and are not repeated here.

## Architecture

- **Layers are crates, and a crate holds one layer.** The target is `rekall-kernel <- rekall-store
  <- rekall-core <- rekall-console <- rekall-http`, with `rekall-mcp` (depends on `rekall-core`
  only) and `rekall-infra` (implements the traits of `rekall-core`) beside them, and
  `rekall-app` wiring everything. `rekall-http` never depends on `rekall-infra`. A dependency that
  points the other way is a design error, not a missing `use`. Until `task:layered-crates-refactor`
  is finished the workspace still has `rekall-common`, `rekall-model`, `rekall-repository`,
  `rekall-service`, `rekall-api`, `rekall-web`, `rekall-claude` and `rekall-platform`: new code goes
  where the target puts it, and a change in an old crate does not widen its role.
- **Code is cut by domain.** `catalog`, `step`, `flow`, `note`, `wrapup`, `diagram`... name a folder
  in every layer crate and a feature in the console. A new use of one domain by another is added
  to the table in `docs/DESIGN.md` and to `domain_edges.rs`, or not written.
- **A feature is a folder first.** A new feature is a domain folder in the layer crates it needs.
  It is a crate of its own only when it knows nothing of Rekall (`rekall-diagram`) or touches the
  machine (a process, file, socket or HTTP) behind a trait.
- **Anything that writes the catalog lives in `rekall-console`.** `rekall-mcp` lists `rekall-core`
  as its only Rekall dependency, so it cannot name a repository, an entity write or a console
  service. A new MCP write is a narrow method on a `rekall-core` service, never a reuse of a
  console service.
- **A console service has no HTTP types.** It takes a command and returns a view, both defined
  beside it. The controller maps the request to the command and the view to the response.
- **One service per entity.** `CompanyService`, `ProjectService`, `TaskService`, `TagService`,
  `TaskStepService`, `WrapupService`… A service that answers for two entities is split. A
  cross-entity flow (auto-commit, importing Jira issues) is its own small service that depends on
  the entity services and holds no queries of its own.
- **Controllers stay thin.** A handler parses the path and body, validates the request shape
  with `Validator` / `present`, calls one service method and maps the result. No queries, no
  business rules, no transactions in a controller.
- **Repositories are free functions over `&impl ConnectionTrait`**, so they run inside whatever
  transaction the caller holds. A lookup that must exist is `repo::<entity>::require`, never a
  hand-written `find_by_id(..).ok_or_else(..)`.
- **Transactions are explicit.** A public service method opens one with `in_read!` / `in_write!`.
  Its `*_in(tx, ..)` twin joins one a caller already holds. Events go through `tx.publish` and
  reach the bus only after commit.
- **Dependency inversion where there is a seam.** Something with an external effect (git, the
  clock, the filesystem, a process, HTTP) is behind a trait or an injected value (`Clock`), so a
  test can replace it. Do not add a trait that has one implementation and no test double.
- **A capability that touches the machine is a port.** The trait lives in `rekall-core`, a thin
  service in `rekall-console` uses it, `rekall-infra` implements it and `rekall-app` injects it.
  `rekall-infra` holds no decision that needs only data: that goes in `rekall-core`, which calls
  repositories.

## Code

- **Small functions with one clear path.** Early returns for refusals, then the happy path.
  If describing a function needs "and", split it.
- **Name the intent, not the mechanics.** A well-named private function replaces a comment.
  Extract a private function when a block does one identifiable thing (a rule, a lookup, a
  computation), even if it has only one caller. Do not extract one-liners: a function whose body
  is a single call adds a hop and no meaning.
- **No duplication.** Before writing a pattern a third time, look for the helper. Existing
  shared pieces: `repo::update_if_changed` (stamp, validate, write only when a value changed),
  `repo::<entity>::require`, `load::resolve_task`, `jstr` for text rules, `text_rule` for a
  writer's capped free text, `task.anchor(..)` / `project.anchor()`, `rekall_common::cleanup` for
  logged best-effort deletes and kills, `Snapshot` for response assembly; in the console,
  `stores/console/rows` (`upsertById`, `replaceById`, `trackSave`).
- **No useless abstraction.** No trait, generic or builder without a second implementation or
  a test that needs it. Composition over inheritance: services hold the services they use.
- **Errors are values.** Return `RekallError` through `Result`. Never `unwrap`/`expect` on input
  or on data from the database; `expect` only on invariants the code itself just established,
  with a message that states the invariant. Never discard a `Result` silently: `let _ =` is only
  for sending on a channel whose receiver may legitimately be gone. Otherwise log it with
  `tracing` at the right level.
- **Client mistakes are 4xx.** Input the caller got wrong is `RekallError::illegal` (400),
  `not_found` (404) or `conflict` (409). `internal` (500) is only for states the code cannot
  explain.
- **Structured logging only.** `tracing::{info, warn, error}` with fields
  (`error!(%failure, task_id = %id, "...")`). No `println!` / `eprintln!` outside the CLI entry
  point.
- **Strong types.** `Id`, `Instant`, enums from `rekall-model` instead of `String`/`bool` flags.
  No `serde_json::Value` where a DTO can carry the shape.

## Comments

- A comment says **why** or states a **rule** the code cannot express by itself. It never
  restates the code.
- At most 30 words on a function and 50 on a type or module.
- Describe Rekall as it is. Do not reference the retired Java/Spring implementation
  (`@Transactional`, `orElseThrow`, Hibernate, Jackson, H2).
- No `TODO` in committed code: fix it or open a task.

## Tests

- Unit tests live under `tests/unit/` mirroring `src/`, attached with `#[path]`; integration
  suites under `tests/integration/`, each declared as a `[[test]]` in its crate's `Cargo.toml`
  (an undeclared suite never runs). See `docs/DESIGN.md` §3.
- Every non-trivial function has a test. External I/O is faked (temporary directories, an
  in-memory database, a fixed `Clock`).
- `cargo test --workspace` and, for the console, `pnpm --dir rekall-ui test`, `typecheck` and
  `lint` must pass before a change is done.

## Frontend (`rekall-ui`)

- Vue 3 `<script setup lang="ts">`, typed props and emits, no `any`.
- Data access goes through the API modules and Pinia stores, never `fetch` from a component.
- One component per concern. Code lives in `src/features/<feature>/{api,model,store,composables,lib,components}`;
  what no feature owns stays in `components/ui`, `components/shell`, `common/` and `stores/console/`.
  The console store is one slice per concern, each in its feature's `store/` and wired in
  `stores/console.store.ts`; a new concern is a new slice, not more lines there.
- Split a component when it does several jobs or serves several entities, not because of its
  length. A long component that is one cohesive thing stays whole; a very long one is a prompt to
  ask what its separate jobs are.

## Commits and docs

- Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`). Never add an AI
  co-author trailer.
- `README.md` and `docs/DESIGN.md` describe the code as it is. Change them in the same commit as
  the behaviour they describe.
