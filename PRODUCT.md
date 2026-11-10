# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

One developer working alone, on their own machine, who drives Claude Code sessions across several companies, projects and tasks. They open the console to plan and review work, then hand a task to a session with one command. Single user, localhost; multi-user and remote use are explicit non-goals (`docs/DESIGN.md`).

## Product Purpose

Rekall stores projects, tasks, markdown notes and checklists in a local app and loads one task in full into a Claude Code session with `/rk project:<label> task:<label>`. Success is a session that starts from the current state of the work instead of reading the code back, and work that stays honest while it runs.

## Positioning

Two claims together, and the second is what makes the first trustworthy:

1. **One-call context with a wrapup.** The task, its project, checklist and every attached note arrive in a single call, and a per-task wrapup records what the implementation is now (a state, overwritten, not a log).
2. **A plan-to-done control loop.** Steps, flows, checks and reviews: Claude can move a step only up to `claimed`, only a person marks it `done`, and adherence to the plan (steps accepted at the first pass, steps sent back, deviations) is measured.

## Operating Context

- The console is a Vue 3 web UI served by `rekall-server` on localhost:47355 (also embedded in a Tauri desktop window for macOS and Windows) over the same REST API; the MCP endpoint is on the same port.
- Sessions run inside the app (in-app terminal or chat view) or in the user's own terminal via "Open in terminal".
- The entry point for Claude is the `/rk` slash command and the Rekall MCP tools; the console is where a person edits, reviews, accepts and ticks done.
- Data lives in one local SQLite file; nothing authenticates and the API is refused from any machine but this one.

## Capabilities and Constraints

- Companies, projects, tasks and epics; notes shared across tasks; titled project pages; step checklists optionally routed into flows; reviews and checks; revisions of a task description; time and token usage; deviations; diagrams (Mermaid); dependency inventory and vulnerability exposure; connections (GitLab, GitHub, Kubernetes, Jira, Grafana); run queue; worktrees.
- Claude reads everything and writes a few narrow things (see `docs/design/write-safety.md`); the last tick, `done`, is a person's.
- Source-available, not open source (see License in README).
- Terminology stays as the product uses it: company, project, task, epic, step, flow, wrapup, note, anchor, label, check, revision, deviation.

## Evidence on Hand

- Product and architecture docs: `README.md`, `docs/DESIGN.md`, `docs/guide/`, `docs/design/console-ui.md`.
- The real console (`rekall-ui`) and demo data (`scripts/seed-demo-data.py`).
- No testimonials, customer names, benchmarks or pricing exist; none are to be invented.

## Product Principles

1. **State over history.** Show what the work is now; accumulating logs are secondary to the current wrapup, step state and plan.
2. **The person has the last word.** Claude proposes and claims; a person accepts and marks done. Surfaces make that boundary legible.
3. **The plan is measurable.** Deviations, sent-back steps and adherence are first-class, not buried.
4. **One call, one place.** Everything a session needs about a task is reachable from the task, without hunting across screens.
5. **Local, single-user, no ceremony.** The app must be reachable with one command and stay out of the way of the terminal session.
