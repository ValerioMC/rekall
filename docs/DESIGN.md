# Rekall - Design Document

Status: label, title and description on projects and tasks; notes shared across tasks; one
wrapup per task, written by Claude and corrected by hand
Date: 2026-08-15

Rekall stores the structure and the working context of the projects and tasks you work on, and
hands them to Claude Code over MCP. Claude reads all of it and writes one thing: the wrapup of
a task, which is what that task's implementation currently looks like.

It replaces a folder tree of markdown files (`ESA/<project>/issues/<task>/*.md`) with five
tables and one command.

---

## 1. Problem

Current setup: one folder per project, one subfolder per issue, markdown files inside. It works
because Claude Code can read files, but:

- No structure. What a task belongs to exists only as prose repeated in every `CONTEXT.md`.
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
| D2 | Claude reads everything and writes one thing | `rekall-mcp` depends on the domain and never on `rekall-api`, so no controller is on its classpath, and every read runs in a read-only transaction. The single exception is `rekall_wrapup`, which can replace one column of one row keyed by a task. Weaker than the absolute this used to be, which was itself weaker than the database role before that; see §8. |
| D3 | Markdown content lives in the database | One backup target, reachable through MCP, searchable. |
| D4 | One entry point, and it is a slash command | A session begins with `/rk project:vega task:report-builder`, not with a question. Reaching a record through a natural-language query costs several turns and a few thousand tokens before any work starts, and it is the part that fails when the model guesses the wrong entity. An explicit anchor removes both. |
| D5 | The model is fixed at compile time | Superseded D6 of the previous design, which had a runtime meta-model and a DDL engine. See §7. |
| D6 | Modular monolith, single process, embedded database | Single user, localhost. The application has to be reachable with one command or it will not get used. |
| D7 | What a record is called and what an anchor resolves are two columns | One column serving both meant a rename broke anchors written down elsewhere, and made every name a compromise between readable and typeable. `label` is a slug and is the identity; `title` is prose and is free. See §4. |
| D8 | A task carries one wrapup, and it is a state and not a log | The thing that costs a session its first twenty minutes is reconstructing what the code already does. A note cannot answer that: notes accumulate, and the reader has to synthesise the current state out of them. A wrapup is that synthesis, written once and overwritten thereafter. Its own table, because a document belongs to many tasks by construction and "one answer per task" has to be a constraint rather than a convention. See §4.1. |

Non-goals: multi-user, authentication, remote deployment, vector search.

---

## 3. Module structure

```
rekall/
  rekall-domain/   entities, repositories, context assembly, Liquibase changelogs
  rekall-api/      REST controllers for the UI
  rekall-mcp/      MCP server, one tool that reads and one that writes a wrapup
  rekall-app/      Spring Boot entry point, serves the built frontend
  rekall-ui/       Vue 3 + Vite (built into rekall-app/src/main/resources/static)
```

`rekall-mcp` must not depend on `rekall-api`. The two are independent consumers of the same
domain, which is what keeps the boundary structural rather than accidental: the only write
`rekall-mcp` can reach is `WrapupService`, and no controller, no `CatalogService` and no
`DocumentService` is on its classpath.

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
| Document editing | `md-editor-v3` | Toolbar over the markdown source, not a WYSIWYG. A WYSIWYG keeps its own tree and regenerates the source on every edit, and that source is what MCP hands to Claude verbatim |
| Tests | JUnit 5 against in-memory H2 | |

---

## 4. Model

```
Company ──< Project ──< Task >──< Document
                         │      (document_task)
                         └──1 Wrapup
```

| Table | Anchored by | Columns of note |
|---|---|---|
| `company` | `name`, unique | `description` |
| `project` | `label`, unique per company | `title`, `status`, `description`, `company_id` |
| `task` | `label`, unique per project | `title`, `status`, `description`, `project_id` |
| `document` | — | `title`, `kind`, `body_markdown` |
| `document_task` | — | `document_id`, `task_id`, `position` |
| `wrapup` | through its task | `task_id` unique, `body_markdown`, `written_by` |

`task.project_id` and `project.company_id` are both `ON DELETE CASCADE`: a project's tasks have
no meaning without it, and neither do a company's projects. The blast radius of deleting a
company is large, which makes it the interface's job to state it before anyone clicks.

### The label and the title are separate columns

One column used to do both jobs. `name` was what a person called the project and what
`project:vega` resolved, which made every rename a silent break of anchors written down
elsewhere, and made a readable name and a typeable identifier the same compromise.

| Column | Job | Constraint |
|---|---|---|
| `label` | What the anchor resolves | Slug: `^[a-z0-9]+([._-][a-z0-9]+)*$`. Unique inside its parent. Normalised on write by `Slug.of`, so `Report Builder` is stored as `report-builder` |
| `title` | What it is called on screen | Free text, `NOT NULL`. Changing it never moves an anchor |
| `description` | What it is about | Free text, travels into every context |

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

A note is attached to **at least one task and possibly many**. It used to belong to exactly one
owner, enforced by three nullable foreign keys and a check constraint, which meant a note that
mattered to three tasks had to be written three times and the three copies drifted. Cluster
access, a naming convention and an onboarding step are all notes of that kind.

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

That distinction is why the two-anchor form exists. `/rk project:vega` loads the project and
lists its tasks as anchors; naming the task as a second anchor is how you narrow to one.

A note arriving with every task that references it is the point of the whole design. In the
folder tree the cluster notes lived in a separate `cluster.md` that had to be opened by hand and
usually was not; copied into each task's folder they went stale instead. One row, many links,
solves both.

---

## 6. MCP server

Transport: HTTP on the same process as the UI.

```bash
claude mcp add --transport http rekall http://localhost:8080/mcp
```

Two tools. `rekall_context` reads, taking one string:

```json
{ "anchors": "project:vega task:report-builder-main-workflow" }
```

`rekall_wrapup` writes, taking the same anchors and the text:

```json
{ "anchors": "project:vega task:report-builder", "body": "## What it does\n..." }
```

An anchor is `entity:value`. A value containing spaces is quoted. A bare term with no `entity:`
is looked up across both entities and accepted only when exactly one record matches; on more
than one the candidates come back and nothing is loaded. The tool does not score, weigh or
choose.

A note attached to several tasks arrives under each of their anchors. The same markdown showing
up twice in one session is the relation working, not a duplicate.

`rekall_wrapup` is narrower on purpose. `rekall_context` may be handed a company and will
happily return everything under it; a write cannot, because a project names forty tasks and
none of them is the answer. Anything that does not resolve to exactly one task is refused with
the form that would have worked, and an ambiguous bare label is reported rather than picked.
The tool description carries the state-not-process rule, because that description is the only
part of this system the model reads before deciding what to write.

`rekall_query`, `rekall_get`, `rekall_search` and `rekall_schema` were all deleted. The first
three were alternative ways in, and a second way in is a second thing to get wrong.
`rekall_schema` existed to tell Claude which entities a user had defined at runtime; with the
entities fixed in code, its answer is a constant and belongs in the tool description.

Responses are capped, with an explicit truncation notice rather than a silent cut.

### Two protocol eras

MCP split in two with revision `2026-07-28`. Up to `2025-11-25` a client opens with an
`initialize` handshake and the agreed revision holds for the session. From `2026-07-28` there is
no handshake and no session: every request carries its own revision, in the
`MCP-Protocol-Version` header and again in `params._meta`, along with its method in `Mcp-Method`
and, for a `tools/call`, its tool name in `Mcp-Name`.

This endpoint answers both. It cost one branch, because the handler never kept state between
requests: the stateless model the new revision mandates is how it already worked. The rest is
checking, in the modern era, that the mirrored headers agree with the body — a gateway routing
on the header while the server executes on the body is two components acting on two different
requests, so a disagreement is `-32020` on a 400 rather than a guess.

| | Handshake era | Stateless era |
|---|---|---|
| Opens with | `initialize`, answered with the revision asked for | nothing |
| Unknown method | `-32601` on a 200 | `-32601` on a 404, so a probe can tell this from a wrong address |
| Unknown revision | — | `-32022` on a 400, listing what is supported |
| `server/discover` | answered | answered, and mandatory |

`2024-11-05` is still accepted, because that is what the endpoint used to pin and a client
registered before this change still opens with it. It is left out of the versions
`server/discover` advertises: its transport was HTTP+SSE, which this server has never spoken, so
a client free to choose should not choose it.

Not implemented, deliberately: `ttlMs`/`cacheScope` on `tools/list`, `structuredContent`, JSON
Schema 2020-12 and `x-mcp-header`. They solve problems this deployment does not have — one
process, one user, on localhost.

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

## 8. Known weakenings

This guarantee has been traded down twice, and both trades are worth stating plainly.

**First**: it used to be a PostgreSQL role. The MCP connection authenticated as
`rekall_reader`, which held `SELECT` and nothing else, so the database refused a write
regardless of what the code did. On an embedded H2 file a separate identity would mean a second
`DataSource`, a second `EntityManagerFactory` and a duplicated set of repository interfaces —
more machinery than the refactor removed elsewhere, so it was not done.

**Second**: it stopped being absolute. The wrapup is written by Claude, and a wrapup Claude
cannot put back is a wrapup nobody writes: the alternative was printing markdown into the chat
for someone to paste, which is a step that gets skipped on the second day. What replaced "no
writes" is a shape:

1. `rekall-mcp` has no controller, no `CatalogService` and no `DocumentService` on its classpath
2. the one write service it can reach, `WrapupService`, takes a task and a body and can do
   nothing else — it cannot create, rename or delete any record, and cannot touch a note
3. every read still runs under `@Transactional(readOnly = true)`, so Hibernate will not flush
4. `McpTool.writes()` is declared rather than inferred, and the startup log names the write
   surface out loud instead of describing every tool as read-only because that used to be true

The residual risk is real and small: Claude can overwrite one task's wrapup with something
wrong, or with something that replaces a correction made by hand. The first is repaired by
writing it again; the second is announced in the tool's answer, because nothing keeps a copy.
What it cannot do is lose a note, move a task or delete anything.

If the application ever moves back to a database with real roles, the first guarantee comes
back as a changeset and a second pool, with `INSERT, UPDATE` on `wrapup` and `SELECT`
everywhere else.

---

## 9. Reversed decisions

Worth keeping, because each was right under its premise and wrong once the premise changed.

| Decision | Reversed because |
|---|---|
| PostgreSQL is a hard requirement, H2 is unsafe | True while a generated DDL plan had to apply atomically; H2 commits implicitly on every DDL statement. With the DDL engine gone there is no plan to apply |
| Rename of a table or column is unsupported | Was a consequence of diffing a meta-model against the physical schema. There is no diff now; a rename is an ordinary migration |
| `rekall_context` is hardcoded around project and task, with the names as configuration | Replaced by a generic list of anchors, then by three known entities. The configuration properties in between existed for about a day |
| The meta-model is the point of the application | See §7 |
| A note belongs to exactly one owner, and the database enforces it | True while a note described one thing. Most notes describe something several tasks share, and a single owner made the second task's copy a fork. Replaced by `document_task` |
| Environments are a first-class entity whose notes arrive with the task | The entity carried two fields and a label. Everything it actually held was already prose, and prose is what a note is. Deleted; the same information is now a note attached to the tasks that need it |

---

## 10. The interface

One surface, three panes: pick a task, pick its note, write. The application used to be a
screen per entity, which is the database schema turned into navigation: reaching the thing you
actually work on, a note, took three or four clicks and two forms.

| Decision | Why |
|---|---|
| The anchor bar is always present, never a summoned palette | What you type in it is what you type after `/rk`. It is the one thing this product has that nothing else does, and it was previously only ever displayed, never used |
| Every control sits above the scrolling content | The old sidebar listed projects underneath the task groups, so filtering changed their height and the projects moved. A target that moves is a target you find again every time |
| The project is a scope, not a destination | You do not go into a project, you filter by one. A control that governs a list belongs above that list |
| Four levels, one control | A select per level would put three controls above the list and make choosing a project a two-step act. One popover with projects nested under their companies is the same information in one gesture, and doubles as the map |
| A row shows only the path the scope leaves ambiguous | Inside one project the project name is the same word on every row, which is noise. It reappears when the scope widens |
| Tasks and Notes are two ways of browsing, not two tables | "The task I am on" and "the note I remember writing" are both real. With notes shared across tasks the second stopped being reachable through the first |
| The wrapup is pinned above the notes, not filed among them | It is the answer to the question you arrive with — what is this now — and the notes are the background to that answer. Filed in the list it would read as the first note, which is the one thing it must not be |
| What sets it apart is form, not a third colour | A raised card, a rule under it, a glyph nothing else uses. The palette spends amber on where you are and cyan on anchors, and a new hue for "this row is special" is how a two-colour system becomes a five-colour one |
| A task with no wrapup shows an empty card, not nothing | The absence is the reason to write one, and hiding the row would leave nothing to click |
| It has a pane of its own, not the note editor with a fixed title | A note has a name, a kind and any number of tasks; a wrapup has none of the three. Every control that would be meaningless is absent rather than disabled |
| The empty state hands over the command, not a form | The primary way a wrapup gets written is Claude, so the screen offers `/rk <anchor> wrapup` ready to copy, and writing it by hand is the secondary path underneath |
| Who wrote it is on screen, with how long ago | Both of you write here and neither merges with the other. Seeing "written by you, 2 days ago" is what tells you the next `/rk … wrapup` is about to replace your own words |
| Notes written since it are counted, as a remark | A wrapup goes stale silently, which is the one way it can start lying. The cheap half of that is detectable, and it is reported as a fact rather than as a warning: newer notes make a wrapup older, not wrong |
| Writing saves itself | A Save button on a notes application is a way to lose work. The state is reported, not requested |
| Destructive actions state their blast radius | The previous screens deleted a project and everything under it on one unguarded click |
| A scoped search says what it is hiding | Finding nothing inside one project teaches you the note does not exist, and the next thing you do is write it twice |
| One record editor for all three levels, opened from the row | Four forms that drift apart, reached through a settings screen listing the same tree a second time, is the arrangement this replaces. Creating and editing ask the same two questions at every level |
| The anchor is assembled live while the label is typed | Nobody should have to save a record to find out what it answers to. The label follows the title until it is touched, and never afterwards |
| Moving a label is allowed and is announced | Nothing stored points at a label, so the breakage is entirely outside the application, where only a warning can reach it |
| Anchors have a colour of their own | Amber marked both the selected row and the anchor, so the thing you click and the thing you copy looked identical. Cyan is spent on labels and anchors and on nothing else |

### Export

`GET /api/export` returns the whole database as a zip of folders, `project/task/note.md`.

The tree cannot represent a note attached to several tasks, so the note is written under each
of them and `MANIFEST.md` records which files are copies of one row. Writing it once with
pointers would produce a tree that only Rekall can read, which defeats the purpose: the format
has to survive the application being abandoned.

A task's wrapup is written as `WRAPUP.md` beside its notes: in capitals and first in the
folder, because on a tree someone opens two years from now it is the file that says what the
thing was.

Folders are named after the label, which is the name that is already an identifier and the one
the manifest's anchors carry, so a folder can be matched to a `/rk` line without guessing.
Labels are sanitised into folder names rather than escaped: a project called `../../etc`
becomes a folder, never a path, and a test asserts no entry in the archive contains `..`.

Keyboard: `⌘K` the anchor bar, `J`/`K` the list, `1`–`4` the status, `T` a new task, `E` edits
the one in view, `N` a new note, `W` its wrapup, `B` swaps Tasks and Notes, `⌘↵` saves the
record editor. Every shortcut is inert while a field has focus.

---

## 11. Roadmap

| Phase | Deliverable | Done when |
|---|---|---|
| 1 | Fixed model, H2, one MCP tool, typed UI | `/rk project:vega task:report-builder-main-workflow` answers correctly. **Done** |
| 2 | Use it on Vega for two weeks, entering data by hand | Either it replaced the folders or it did not |
| 3 | Importer for the existing `ESA/` tree | Only if phase 2 says the tool is worth filling |
| 4 | Export to a folder tree | A backup and an escape hatch, not a sync. **Done**: `GET /api/export` returns a zip of `project/task/note.md` |
| 5 | The wrapup | A session ends by recording what the implementation now is, and the next one opens on it. **Done**: `rekall_wrapup`, one row per task, editable in the console |

Phase 2 is deliberately not a coding task. The risk to this project was never the design, it
was re-architecting instead of finishing; the only way to find out what is missing is to use it.
