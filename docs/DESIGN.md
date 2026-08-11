# Rekall - Design Document

Status: rewritten after the reduction to a fixed model
Date: 2026-08-11

Rekall stores the structure and the working context of the projects and tasks you work on, and
exposes them read-only to Claude Code over MCP.

It replaces a folder tree of markdown files (`ESA/<project>/issues/<task>/*.md`) with four
tables and one command.

---

## 1. Problem

Current setup: one folder per project, one subfolder per issue, markdown files inside. It works
because Claude Code can read files, but:

- No structure. The relationship between a task, its environment and its cluster exists only as
  prose repeated in every `CONTEXT.md`.
- Duplication. Cluster coordinates and credentials are copied across files and drift.
- Loading a task into context means naming the right files by hand, every time.

Target: type `/rk project:stvv task:code-validator-main-workflow` and have the full context
loaded in one call, including the environment configuration the task points at.

---

## 2. Decisions

| # | Decision | Rationale |
|---|---|---|
| D1 | Real tables with real foreign keys | A task can never reference a deleted environment. On an app whose only job is to be a reliable memory, silent dangling references are the failure mode that matters. |
| D2 | Claude is read-only | `rekall-mcp` depends on the domain and never on `rekall-api`, so no controller and no write service is on its classpath, and every read runs in a read-only transaction. Weaker than the database role this used to be; see §8. |
| D3 | Markdown content lives in the database | One backup target, reachable through MCP, searchable. |
| D4 | One entry point, and it is a slash command | A session begins with `/rk project:stvv task:code-validator`, not with a question. Reaching a record through a natural-language query costs several turns and a few thousand tokens before any work starts, and it is the part that fails when the model guesses the wrong entity. An explicit anchor removes both. |
| D5 | The model is fixed at compile time | Superseded D6 of the previous design, which had a runtime meta-model and a DDL engine. See §7. |
| D6 | Modular monolith, single process, embedded database | Single user, localhost. The application has to be reachable with one command or it will not get used. |

Non-goals: multi-user, authentication, remote deployment, vector search.

---

## 3. Module structure

```
rekall/
  rekall-domain/   entities, repositories, context assembly, Liquibase changelogs
  rekall-api/      REST controllers for the UI
  rekall-mcp/      MCP server, one read-only tool
  rekall-app/      Spring Boot entry point, serves the built frontend
  rekall-ui/       Vue 3 + Vite (built into rekall-app/src/main/resources/static)
```

`rekall-mcp` must not depend on `rekall-api`. The two are independent consumers of the same
domain, which is what keeps the read-only guarantee structural rather than accidental.

### Stack

| Component | Choice | Note |
|---|---|---|
| Runtime | Java 25 | |
| Framework | Spring Boot 4.1 | |
| Persistence | Spring Data JPA | One technology, no second query builder |
| Migrations | Liquibase | Portable types, so the door to PostgreSQL stays open |
| Database | H2, file | Embedded. No server to start |
| Boilerplate | Lombok | `annotationProcessorPaths` is declared explicitly: from JDK 23 javac no longer runs processors it merely finds on the classpath |
| MCP server | Hand-rolled | MCP over HTTP is three JSON-RPC methods and one POST endpoint |
| Frontend | Vue 3 + Vite | |
| Tests | JUnit 5 against in-memory H2 | |

---

## 4. Model

```
Project ──< Task >── Environment
   │          │           │
   └── notes  └── notes   └── notes
```

| Table | Anchored by | Columns of note |
|---|---|---|
| `project` | `name`, unique | `status`, `description` |
| `task` | `name`, unique per project | `status`, `description`, `project_id`, `environment_id` |
| `environment` | `label`, unique | `namespace`, `kubeconfig_path` |
| `document` | — | `title`, `kind`, `body_markdown`, exactly one owner |

`task.project_id` is `ON DELETE CASCADE`: a project's tasks have no meaning without it.
`task.environment_id` is `ON DELETE RESTRICT`: deleting an environment tasks still run on is a
mistake worth refusing, not a reason to delete the tasks.

A document carries three nullable foreign keys and a check constraint that exactly one is set.
The previous design used a soft `(entity_name, record_id)` pair because the owning tables did
not exist when Liquibase ran. They do now, so the database can enforce that a note never
outlives what it documents, and a project reaches its notes by navigating an association rather
than by running a query.

Task names are unique per project rather than globally, because two projects routinely have a
task with the same name. A bare `task:setup` that matches in two projects is reported as
ambiguous; `project:ainabler task:setup` resolves it.

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
| Forward (`@ManyToOne`) | The record in full, **with its documents** | What this record depends on to be understood. Bounded by the model: a task reaches an environment and stops |
| Inverse (`@OneToMany`) | Labels only, printed as anchors | What points back at this. Unbounded fan-out: a project has forty tasks |

That distinction is why the two-anchor form exists. `/rk project:stvv` loads the project and
lists its tasks as anchors; naming the task as a second anchor is how you narrow to one.

The environment's own notes arriving with the task is the point of the whole design. In the
folder tree those lived in a separate `cluster.md` that had to be opened by hand, and usually
was not.

---

## 6. MCP server

Transport: HTTP on the same process as the UI.

```bash
claude mcp add --transport http rekall http://localhost:8080/mcp
```

One tool, `rekall_context`, taking one string:

```json
{ "anchors": "project:stvv task:code-validator-main-workflow" }
```

An anchor is `entity:value`. A value containing spaces is quoted. A bare term with no `entity:`
is looked up across all three entities and accepted only when exactly one record matches; on
more than one the candidates come back and nothing is loaded. The tool does not score, weigh or
choose.

`rekall_query`, `rekall_get`, `rekall_search` and `rekall_schema` were all deleted. The first
three were alternative ways in, and a second way in is a second thing to get wrong.
`rekall_schema` existed to tell Claude which entities a user had defined at runtime; with three
entities fixed in code, its answer is a constant and belongs in the tool description.

Responses are capped, with an explicit truncation notice rather than a silent cut.

---

## 7. What was removed, and why

The previous design had a runtime meta-model (`meta_table`, `meta_field`, `meta_relation`), a
jOOQ DDL engine that diffed it against `information_schema`, an alter-rule classifier
(`SAFE` / `NEEDS_INPUT` / `BLOCKED`), transactional apply, a `ddl_log` that replaced Liquibase
for the generated schema, and a UI to design all of it. Roughly 4,600 lines of Java and 1,200
of frontend.

It was removed because **D4 had already made it pointless**. The meta-model existed so that
Claude could map a natural-language question onto entities unknown at compile time, using
user-written descriptions and aliases as its map. Once the entry point became an explicit,
typed anchor, the entity name is something the user writes by hand. A name you type by hand can
be a constant.

What that bought, beyond the deletion:

- "Load everything connected" stopped being a feature to design. Many-to-many was never read by
  the dynamic repository at all; with associations it is Hibernate's problem, not ours.
- PostgreSQL stopped being a requirement. Transactional DDL was the only reason it was
  mandatory, and there is no runtime DDL any more. That in turn removed minikube, Docker,
  port-forwards and Testcontainers.
- Immutable identifiers stopped being a rule to enforce. There is no user-supplied identifier.

What it cost: adding an entity is now a JPA class plus a changeset plus a restart, instead of
three clicks. For a single user who is also the author, that is twenty minutes against five.

---

## 8. Known weakening

The read-only guarantee used to be a PostgreSQL role: the MCP connection authenticated as
`rekall_reader`, which held `SELECT` and nothing else, so the database refused a write
regardless of what the code did.

On an embedded H2 file, a separate identity would mean a second `DataSource`, a second
`EntityManagerFactory` and a duplicated set of repository interfaces. That is more machinery
than the refactor removed elsewhere, so it was not done. The guarantee is now:

1. `rekall-mcp` has no controller and no write service on its classpath
2. every read runs under `@Transactional(readOnly = true)`, so Hibernate will not flush

This is weaker, and it is a deliberate trade rather than an oversight. If the application ever
moves back to a database with real roles, restoring the old guarantee is a changeset and a
second pool.

---

## 9. Reversed decisions

Worth keeping, because each was right under its premise and wrong once the premise changed.

| Decision | Reversed because |
|---|---|
| PostgreSQL is a hard requirement, H2 is unsafe | True while a generated DDL plan had to apply atomically; H2 commits implicitly on every DDL statement. With the DDL engine gone there is no plan to apply |
| Rename of a table or column is unsupported | Was a consequence of diffing a meta-model against the physical schema. There is no diff now; a rename is an ordinary migration |
| `rekall_context` is hardcoded around project and task, with the names as configuration | Replaced by a generic list of anchors, then by three known entities. The configuration properties in between existed for about a day |
| The meta-model is the point of the application | See §7 |

---

## 10. Roadmap

| Phase | Deliverable | Done when |
|---|---|---|
| 1 | Fixed model, H2, one MCP tool, typed UI | `/rk project:stvv task:code-validator-main-workflow` answers correctly. **Done** |
| 2 | Use it on STVV for two weeks, entering data by hand | Either it replaced the folders or it did not |
| 3 | Importer for the existing `ESA/` tree | Only if phase 2 says the tool is worth filling |
| 4 | Export to a folder tree | A backup and an escape hatch, not a sync |

Phase 2 is deliberately not a coding task. The risk to this project was never the design, it
was re-architecting instead of finishing; the only way to find out what is missing is to use it.
