# Rekall — product specification (entities & functionality only)

This document describes what the application *is* and *does*: its data model, the rules that
govern it, and the functionality it exposes. It intentionally contains no visual, layout, or
component-level design direction — that part is left open for whoever designs the interface.

## What it's for

Rekall is a personal, local-first tool for someone who works on many ongoing bodies of work (for
different clients or projects) together with an AI coding assistant. The core problem it solves:
an AI assistant starting a new working session has no memory of past sessions and no context
about the work unless it's explained from scratch every time, which is slow and error-prone.

Rekall lets the person keep a structured record — companies, the projects they run, the tasks
within each project, freeform notes, and a live "here's what the code currently does" summary per
task — and hand all of it to an AI assistant in a single command, addressed by short, typed
identifiers. At the end of a session, the assistant can write back a fresh summary of what a
task's implementation now is, so the next session starts from the current state instead of
re-reading the codebase.

It runs entirely against a single local data file — no server, no account, no network service —
and can hold more than one such file (e.g. one per client, or one for personal projects), switching
between them.

## Entities

### Company
The outermost record — who the work is for.
- **name** — unique across the whole system. This is also its public identifier (see Anchors).
- **description** — free text.
- Holds many **Projects**. Deleting a company deletes every project and task beneath it.

### Project
Something you work on over time, inside one company.
- **label** — a short slug (lowercase letters, digits, `-`, `_`, `.`; no spaces). Unique *within
  its company* (two different companies may each have a project called `website`). This is the
  project's public identifier.
- **title** — the free-text name people actually call it. Can change at any time without
  affecting the label/identifier.
- **status** — one of: Active, Paused, Done.
- **description** — a short prose summary of what the project is.
- **blueprint** — a separate, long-form markdown document: how the project is built, how it's
  organized, and the conventions to follow while working in it — the project's own "how this
  works" reference. Distinct from the short description: this is meant to be read start-to-finish
  as documentation, and is included whenever an AI assistant loads context for this project or
  any of its tasks.
- Belongs to exactly one company. Holds many **Tasks**. Deleting a project deletes every task
  beneath it.

### Task
A unit of work inside a project.
- **label** — a short slug, unique *within its project*.
- **title** — the free-text name.
- **status** — one of: To do, In progress, Blocked, Done.
- **description** — a short prose summary.
- Belongs to exactly one project.
- May be linked to any number of **Notes** (see below).
- May have **at most one Wrapup** (see below).
- May have any number of **Time entries** (see below).
- Deleting a task does not delete the notes attached to it (they may still be attached to other
  tasks) — it only unlinks them, and a note left attached to nothing afterward is removed.

### Note
A piece of freeform markdown documentation.
- **title**.
- **kind** — a loose, non-enforced category (e.g. "context," "notes," "architecture," "report,"
  or anything else the user types).
- **body** — markdown, can be long (tens of thousands of characters).
- Can be attached to **one or more tasks at once** (a many-to-many relationship). This is
  deliberate: shared knowledge (e.g. "how to access the shared staging cluster") is written once
  and appears automatically wherever it's relevant, rather than being copy-pasted per task.
- A note attached to several tasks shows, from within any one of them, which other tasks it's
  also attached to, and editing it there affects what those other tasks see too.

### Wrapup
The current state of one task's implementation — not a log of changes, a snapshot of *what it is
right now*, replaced wholesale each time it's rewritten (never appended to, never merged).
- Exactly one per task, or none.
- **body** — markdown.
- **written by** — either a person or an AI assistant. Whichever wrote it last is recorded and
  shown, since the next rewrite (by either side) fully replaces what's there, and knowing whose
  words are about to be overwritten matters.
- The intended workflow: an AI assistant writes this at the end of a work session, describing
  what now exists; a person can also correct it by hand at any time. The next session opens by
  reading this first, rather than by re-reading the code.
- The system can note, informationally, whether newer notes exist on the task than the wrapup's
  last update — a hint that the wrapup might be stale, not a hard rule.

### Time entry
One session of work on a task.
- **started at**, **stopped at** (the latter is empty while the session is still running).
- Only **one time entry across the entire system** may be open (running) at once — starting a
  timer on a task automatically stops whatever else was running elsewhere. There is no concept of
  running two timers in parallel.
- Past entries can be corrected by hand (adjusting start/stop times) or deleted.
- The total time on a task is the sum of all its entries, live-updating while one is running.

## Relationships at a glance

```
Company 1───< Project 1───< Task >───< Note
                  │                    │
                  │ (blueprint,        └── one Wrapup
                  │  a document           per Task
                  │  field on Project,
                  │  not a Note)
                  └───< Task >───< Time entry
```

- A project always belongs to exactly one company; a task always belongs to exactly one project.
  Nothing in this model is a floating, ownerless record.
- A note's relationship to tasks is many-to-many; everything else above is one-to-many.

## Anchors — the addressing scheme

Every Company, Project and Task can be referred to by a short, typed, human-writable identifier
called an anchor: `entity:value` — for example `company:acme`, `project:vega`,
`task:report-builder`. The value is always the record's *label* (or the company's *name*), never
its free-text title, because the label is the one thing guaranteed not to change silently.

- A task's full anchor is written as two parts together — the project it belongs to, then the
  task — e.g. `project:vega task:report-builder`, because a task's label is only unique inside
  its own project, not globally.
- A bare word with no `entity:` prefix is also accepted and searched across all three kinds; if
  it matches more than one record, the system reports the ambiguity (listing what it could mean)
  rather than guessing.
- Renaming a label changes its anchor going forward. Nothing internal to the system points at a
  label directly (relationships are by internal id), but anything written down externally
  referencing the old anchor will stop resolving — this is a known, accepted trade-off, not a bug.

## Functionality

### Catalog management
Create, edit and delete companies, projects and tasks. Editing a project or task can move it to a
different parent (a different company, or a different project) and/or change its label — both
are ordinary edits, not special operations, though changing a label is understood to move its
anchor.

Deleting any record is a cascading operation and the person doing it should be able to see what
else goes with it before confirming (e.g. deleting a project also deletes N tasks and detaches M
notes).

### Working context for an AI assistant
The headline capability: given one or more anchors, assemble everything relevant into a single
bundle handed to an AI assistant at the start of a work session:
- The record(s) named by the anchor(s), including their own fields.
- What each one references (a task's project; a project's company), also fully resolved — but
  only one level of "reach," not unbounded (loading a task does not also dump every sibling task
  in its project).
- What points back at the loaded record, listed only as further anchors (not expanded), so the
  window isn't spent on content that wasn't asked for.
- Every note attached to a loaded task, in full.
- The project's blueprint, if one is loaded (directly or via a task).
- The task's wrapup, if it has one — presented as the current state of the work, ahead of the
  notes, since that's the first question a new session is answering.

The assistant, at the end of a session, can write back a full replacement for one task's wrapup
— this is the only write this integration is allowed to make.

### Browsing and writing
A working surface for a person (not the assistant) to:
- See and filter the tasks in view (by free-text search across title/label/project/company, and
  scoped to "everything," one company, or one project at a time).
- Pick a task and write/read the notes attached to it, and its wrapup.
- Create a note and, at any point, attach or detach it from any number of tasks.
- Start, pause and review time tracking for the task in view, and correct or delete past sessions.
- See and write a project's description and its blueprint document.
- Quickly copy the anchor for anything in view, formatted exactly as it should be typed to load it
  again.

### Data portability
Export everything (every company, project, task and note) as a downloadable archive laid out as a
folder tree mirroring the company/project/task hierarchy, for backup or for reading outside the
application.

### Local data management
- The application's data lives in a single local file; the person points the application at a
  folder the first time it runs, and a fresh file is created there (or an existing one is opened,
  if one is already at that location).
- More than one such file/location can be registered, given a label, and switched between —
  useful for keeping, say, work and personal contexts fully separate.
- If the currently active file becomes unreachable (e.g. it lived on a drive that's now
  disconnected), the application should say so plainly, offer to reconnect once it's back, offer
  to switch to another known, reachable one, or offer to point at a new location — without ever
  silently starting over.

## Explicitly out of scope for this spec

Nothing about how any of this is laid out on screen, what it's called visually, what colors or
components are used, or how many screens vs. panels it takes — that is left entirely open.
