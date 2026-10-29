# Data: model, context size, backups

## Model

```
Company ──< Project ──< Task >──< Document
               │         │       via document_task
               │         ├──< TaskStep ──< StepLink (a route to another step of the task)
               │         ├──< StepVisit ──> TaskStep (one entry of the flow into it)
               │         ├──1 Wrapup
               │         ├──< TokenUsage ──> TaskStep (optional)
               │         ├──> Task (its epic, optional)
               │         └──> Tag (optional)
               ├──< DeploymentGraph ──< GraphStep ──< GraphRoute
               ├──< Diagram ──> Task (the one it was generated from, optional)
               ├──< ProjectDependency
               └──< VulnerabilityExposure ──> SecurityAdvisory, Task (the fix, optional)
```

| Entity | Anchored by | Holds |
|--------|-------------|-------|
| `Company` | `name` | description, its projects |
| `Project` | `label`, unique per company | title, status, description, its tasks |
| `Task` | `label`, unique per project | title, status, kind (a task or an epic), the epic it belongs to, markdown description, a standing wrapup directive, its notes, steps, wrapup, an optional tag, when a deployment graph was laid on it and whether it is left unreleased |
| `Document` | none | title, kind, markdown body, the tasks it is on |
| `TaskStep` | through its task | title, optional detail, state, position, the connection it reaches when a deployment graph laid it |
| `StepLink` | through its task | the step it leaves, the step it leads to, its outcome (a slug, unique per step), position. Deleted with either step |
| `StepVisit` | through its task | the step entered, its move number in the task, when it was entered and left, the outcome it left by and what it reported. The open one is where the flow stands |
| `Wrapup` | through its task | markdown body, who wrote it last. One per task |
| `Tag` | `name`, unique | icon key, glow-colour key, the tasks currently wearing it |
| `TokenUsage` | through its task | the input, output, cache-write and cache-read tokens one Claude Code session spent on the task or one of its steps on one UTC day. Rewritten whole each time that session's transcript is read; `transcript_scan` remembers how much of each transcript was read |
| `ProjectDependency` | through its project | ecosystem, name, exact version, the file that declares it. Unique per project, ecosystem, name and version; replaced whole by `rekall_inventory` |
| `SecurityAdvisory` | its GHSA id, unique | CVE id, summary, description, link, publication date, the packages it affects as JSON. Recorded by a scan; never deleted |
| `VulnerabilityExposure` | through its project | the advisory, the package and version it reached the project through, a session's verdict, reasoning and conclusion, the fix task. One per project and advisory; deleted with either |
| `DeploymentGraph` | id | title, description, its steps and routes, whether it is the project's release graph. Written whole; goes with its project |
| `GraphStep` | through its graph | title, optional detail, position (the first is where the flow begins), the connection it reaches, visit budget, where it was dragged on the canvas |
| `GraphRoute` | through its graph | the step it leaves, the step it leads to or none for the end, its outcome (a slug, unique per step), position |
| `Diagram` | id | title, the question it answers, its sheets' types, whether one plays as a walkthrough, and its Mermaid sheets with their code links and walkthroughs as JSON. Goes with its project; loses its task when the task is deleted |

`label` is what an anchor resolves: lowercase letters, digits, `-`, `_`, `.`, no spaces, normalised on write. `title` is free text and changing it never breaks an anchor. Renaming a label moves the anchor, and the editor says so before saving.

A project belongs to one company, a task to one project. A note belongs to a scope (a project, a company, or none for global) and sits on any number of that scope's tasks. Deleting a task unlinks its notes; deleting a project or company deletes the notes it owns. A wrapup and a step belong to exactly one task and are deleted with it. Adding an entity is a SeaORM model and a migration in `rekall-store`, not a UI action.

## Context size and reference notes

On the title line of a task's description, a chip estimates what `/rk project:… task:…` costs a session: the characters of the markdown it hands over and roughly how many tokens that is (3.5 characters to a token, an estimate for comparing tasks, not a bill). Under the pointer it lists the parts heaviest first: the description, the steps, the wrapup, the commits chosen for the context, each note, and the project around them. `GET /api/tasks/{id}/context-size` measures it with the same context renderer that answers `rekall_context`, so the figure is the length of what a session gets.

A note most sessions do not need can go **By reference**, switched on the note's own pane. It then travels as its title, its first line and an anchor such as `note:3f2a9c1e` (`loaded="on request"`), and a session loads the rest with `rekall_context` and that anchor only when the work asks for it. **In full** puts it back. The mode belongs to the note, so it holds on every task the note is on.

## Backups

SQLite's `VACUUM INTO` copies the open database, consistently and without stopping it, into a zip in a `backups` folder beside it. One is taken when Rekall starts and whenever the newest is older than the interval, checked every hour; **Back up now** in **Settings > Backups** takes one on demand, and every restore takes one first. Only the newest are kept.

| Property | Default | Meaning |
|---|---|---|
| `rekall.backup.enabled` | `true` | Scheduled backups. **Back up now** and restores work either way |
| `rekall.backup.interval-hours` | `24` | Age of the newest backup that makes another one due |
| `rekall.backup.keep` | `10` | Backups kept, of every kind together; the oldest go first |

**Restore** on a listed backup, or **Restore from a file…** with a zip from elsewhere, replaces the whole database: the zip has to hold a Rekall SQLite database (`rekall.db`, checked by its header), what is there now is backed up first, and Rekall restarts on the restored file, migrating it forward if it came from an older version. That round trip is also how a database moves to another machine: **Download** a backup here, restore it there. An in-memory database has no backups. The REST side is `GET` and `POST /api/backups`, `GET /api/backups/{name}`, `POST /api/backups/{name}/restore` and `POST /api/backups/restore` (multipart `file`, up to 1 GB).

## Older databases

The schema starts from a baseline: the tables as release v0.2.x left them. A `rekall.db` written by v0.2.x is moved onto it the first time it is opened, with every row kept. A file older than v0.2.0 is refused with a message, and so is a backup of one, before it replaces anything. Open such a file once with v0.2.2, then open it here. The H2 file (`rekall.mv.db`) of the first, Java-based versions can no longer be read: import it with v0.2.2, which still has the importer.

