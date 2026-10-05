# Rekall — project rules

Rules every change to this repository follows. The crate layout and the file-per-type
convention are in `docs/DESIGN.md` §3 ("Layout inside a crate") and are not repeated here.

## Architecture

- **Layers are crates.** `rekall-common <- rekall-model <- rekall-repository <- rekall-service <-
  rekall-api / rekall-mcp / rekall-claude <- rekall-app`. A dependency that points the other way
  is a design error, not a missing `use`.
- **Anything that writes the catalog lives in `rekall-api`.** `rekall-mcp` must never be able to
  reach it. A new MCP write is a narrow method on a `rekall-service` service, never a reuse of a
  console service.
- **One service per entity.** `CompanyService`, `ProjectService`, `TaskService`, `TagService`,
  `TaskStepService`, `WrapupService`… A service that answers for two entities is split. A
  cross-entity flow (restoring a revision, auto-commit) is its own small service that depends on
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
- One component per concern. The console store is one slice per concern under
  `stores/console/`, wired in `console.store.ts`; a new concern is a new slice, not more lines there.
- Past ~300 lines a component is doing several jobs: split the one you are changing
  (`RunQueuePanel`, `NavigatorPane`, `StepSeal`, `NoteAssignmentDialog` are well past it) instead
  of growing it further.

## Commits and docs

- Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`). Never add an AI
  co-author trailer.
- `README.md` and `docs/DESIGN.md` describe the code as it is. Change them in the same commit as
  the behaviour they describe.
