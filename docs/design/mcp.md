# MCP server

Transport: HTTP on the same process as the UI.

```bash
claude mcp add --transport http rekall http://localhost:47355/mcp
```

Six tools. `rekall_context` reads, taking one string:

```json
{ "anchors": "project:vega task:report-builder-main-workflow" }
```

`rekall_wrapup` writes, taking the same anchors and the text:

```json
{ "anchors": "project:vega task:report-builder", "body": "## What it does\n..." }
```

`rekall_step` moves one step of a task along its line, taking the anchors, the step's number or
title, and the target state:

```json
{ "anchors": "project:vega task:report-builder", "step": "3", "state": "running" }
```

`state` is `running`, `claimed` or `open`. It refuses `done`, and refuses a step a person has
already accepted: reopening one is a console decision.

`rekall_record_commit` logs the tip of the project's repo folder, or an earlier hash, against a
task or one of its steps. On a project set to auto-commit, `rekall_step` with `claimed` (and
`rekall_wrapup` on a task with no checklist) commits the folder itself; both take an optional
`commit_message` the session writes, a Conventional Commits subject and a few sentences of body,
and without one `CommitMessageGenerator` draws the message from the step's detail or the wrapup.
No commit message lists files.

Before either claim the project's checks run (`CheckExecutionService::run_for_step_claim`, and
`run_for_wrapup_claim` for the wrapup of a task with no steps; a wrapup on a task with steps runs
none). They run one at a time, the project's first, and the first red one refuses the call with a
`ToolError::Failure` naming it and the end of its output (`claim_gate::admit` in `rekall-mcp/src/check/`):
the step stays as it was, a refused wrapup is not written, and nothing is committed. A green run
adds a `Checks passed: ...` line to the answer. A refusal produces no `outcome`, so a flow does not
move. The checks are written only by the console: no tool creates or edits one. The server sets no
timeout on a request, so a claim lasts as long as the sum of its checks' timeouts; the one limit is
Claude Code's own wait for an MCP tool call.

`rekall_propose_step` adds one step to the tail of the checklist, open and ready to work. With
`draft: true` (which `/rk … plan` does not pass; a person asks for it) it stays a draft instead: not work, off the
checklist a session reads until a person promotes it. A `Staging` value carries the choice into
`TaskStepService::propose`, which promotes the new step and keeps the drafts behind it. A title the
task already has is refused so a second plan does not double the checklist, and a task holds at most twenty
drafts (open steps have no such cap).

`rekall_propose_flow` is the same choice for a task whose work branches or loops: `/rk … plan` calls it
once, without `draft`, with every step and its routes, when a step's outcome decides what comes next. The
plan is checked whole and written in one transaction, so a refused plan leaves nothing on the shelf (4.3
in [workflow.md](workflow.md)); `FlowProposalService::propose` lays it as drafts and, for `Staging::Ready`,
releases them in the same transaction. The tool text asks the model to choose checklist or flow first and,
for a flow, to work out each step's arcs: what can end it and where each outcome goes.

`rekall_replan` is how a session redesigns a task that already has steps. `FlowProposalService::replan`
checks the plan whole (`FlowPlan::check_as_written`: a flow when any step has a route, a checklist
otherwise), then in one transaction deletes the task's draft and open steps (`TaskStepService::clear_unstarted_in`)
and lays the plan with `lay_in`. `TaskStepState::unstarted` is the one rule for what a session may delete:
running, claimed and done steps carry a run, work waiting for review or work a person accepted, so they stay
and the new steps follow them. A flow begins at the first step, so a flow is refused while any step stays;
a title a staying step has is refused; either refusal leaves everything as it was. `rekall_delete_step`
(`TaskStepService::remove`) deletes one draft or open step by number or title under the same rule, renumbers
the rest and drops the routes that touched it.

`rekall_create_task` creates a task in the project its anchor names, with an optional description and
up to twenty steps, in one transaction. `TaskAuthoringService` in rekall-core is its only path: the
console's `TaskService` in rekall-console stays the one that edits, reviews and deletes. The task starts
`TODO` and its steps start open: a task asked for in one call is meant to be run, so there is no promotion
gate here, as with `rekall_propose_step` and `rekall_propose_flow` unless they are called with `draft: true`. A step carries what a flow step does
(`routes`, `max_visits`, `new_session`); the service lays the plan as drafts through
`FlowProposalService::lay_in` and releases them with `TaskStepService::release_drafts_in` in the same
transaction, so a task with any route is checked as a flow (`FlowPlan::check`) and one without as a
checklist (`FlowPlan::check_shape`), and a refused plan creates nothing. A label
the project already uses is refused rather than suffixed, so an anchor never changes under the session.
It publishes `task-created` on the event stream and the console reloads its task list. `kind: "epic"`
creates an epic, refused with steps before anything is written, and `epic` files the new task under an
epic of the same project (`TaskBrief`).

`rekall_join_epic` files an existing task under an epic: `EpicMembershipService::join` in
rekall-core, the one write a session makes on a task it did not just create. It checks
`require_joinable`, moves a task between epics and names the one it left, never takes a task out of an
epic and never changes a kind, and publishes `task-created` so the console reads its lists again.

`rekall_read_sections` is the read side of `rekall_project_section`: a context lists a project's
sections by title only, so this is the only way a session reads one. `ProjectSectionService::read` lists the titles, or returns one
section by title ignoring case; a title the project lacks is refused with the titles it has.

`rekall_search` is how a session learns that earlier work exists before redoing it: a context only
holds what it was handed. `SearchTool` calls the `SearchService` the console's search bar uses, or
`search_in_project` when a `project:` anchor comes, and answers with an index only: the kind, the
title, the anchor to pass to `rekall_context` (`SearchHit::context_anchor`, a `note:` anchor for a
note) and the excerpt, at most eight hits per kind. A term under three characters gets a message,
not an error, and no note body leaves beyond its excerpt.

`rekall_note` writes a new note onto one task, taking the anchors, a `title` and the `body`.
It is what a session uses for output worth keeping that is not the state of the implementation:
what a step asks to keep, or the note a task exists to write. `/rk … note "…"` asks for one
explicitly. It creates, and does nothing else: a title the task's notes already carry is refused
rather than overwritten, and the note is `FULL` with kind `notes` until a person changes it.

`rekall_inventory` replaces a project's dependency list whole, taking a `project:` anchor and `dependencies`
(`[{ecosystem, name, version, manifest}]`); `rekall_vulnerabilities` reads the exposures of a project, or
one by its GHSA id; `rekall_assess_vulnerability` records a verdict, its reasoning and its conclusion on one exposure. See
[vulnerabilities.md](vulnerabilities.md).

`rekall_deviation` records where a session's work left the plan: the anchors, the number of the `step` (from 1, as the checklist lists them, drafts not counted) and the `body`. It writes a `task_deviation` row, one per task and step, so it never appears among the task's notes; recording again for a step rewrites that row, a step the checklist does not have is refused, and the `deviation` event tells an open console. The context carries the deviations of a task in a `<deviations>` block, in step order, in full.

The context of a task also carries its thread in a `<thread pending="N" running="R" applied="M">` block, oldest first. A `PENDING` or `RUNNING` entry is rendered in full with its `seq`, `kind` and body (a `REVISION` as the `<quoted>` text before and the `<new>` text now), plus `pass="P"` once Run revision took it into a pass and `follows="S"` when it adds detail to entry S, because those are what a session acts on; an `APPLIED` entry is one line with its `seq`, `kind` and the first line of its reply, because it is history. The block is left out when the thread is empty. `/rk … revise` works one pass (the `RUNNING` entries, or every `PENDING` one when none runs, plan feedback first through the replanning rules) and answers every entry with `rekall_feedback`; `/rk … replan` works the plan feedback alone; a plain run never works either.

`rekall_feedback` answers the thread under a task's description: the anchors and a list of `replies`, each a `seq` and a short `reply`. It calls `FeedbackService::reply` and nothing else: every named entry must be `PENDING` or `RUNNING`; a `seq` the thread lacks, an applied entry, a blank reply or one over 2000 characters refuses the whole call, and a `feedback` event reaches an open console after the commit. The answer lists the entries still open (`Still open: #3`), so a session sees what it did not answer. It cannot create, edit or delete an entry. When the session on a task alone ends, `WorkMarker` calls `FeedbackService::release_running` and the entries it took and never answered go back to `PENDING`.

`rekall_project_section` is `rekall_note` for a project: the anchors name a task, the section lands on that
task's project, and `replace` rewrites the body of the section with the same title (ignoring case) instead
of refusing. Sections live in their own table, `project_section`, not in `document`, because a note is
reached through tasks and loads into a context and a section does neither. Renaming, reordering and
deleting are the console's, in `ProjectSectionCatalogService` in rekall-console.

A project's components are the folders of it that are parts of it, one row each in `project_component`
(`name`, `folder`, `description`, `position`; the name is unique per project ignoring case, and starts as
the folder's last segment when none is given). Unlike a section, they are context: `ContextService`
reads them with the project and `ContextRenderer.render_components` writes a `<components>` block, so the
project's own anchor and every task of it carry them once, and `ContextSizeService` counts them as their
own part. They are read-only to a session, no MCP tool writes them; the console's
`ProjectComponentCatalogService` in rekall-console lists, adds, edits, orders and deletes them.

A task's description and its step details can name a component as `[[Name]]`. The stored text never
changes: `ComponentReferences` (rekall-core, `component/`) expands each reference while the context
is rendered, from the components `ContextService` puts on the task record as `component_lookup`
(resolution only, the `<components>` block stays on the project). The name matches ignoring case and
surrounding spaces. The first reference to a component in a rendered task becomes its name, folder and
description (cut at `COMPONENT_REFERENCE_MAX`), later ones its name and folder; code spans and fenced
blocks are left as written; a name that matches no component of the task's project becomes
`[[Name]] (component not found in this project)`. `ContextSizeService` measures the expanded text.

The same scan resolves `[[connection:label]]` against the project's bindings, which `ContextService` puts
on the task record as `connection_lookup` (handed in through `ComponentReferences::with_connections`, the
reason `component` may use `connection`). The label matches ignoring case and surrounding spaces and
becomes `**label** (connection "Title", reached as its line in `<connections>` says)`, so how to reach it
is still written once; a label the project does not bind becomes `[[connection:label]] (connection not
found in this project)`. The prefix decides: a component cannot be referenced by a name starting
`connection:` or `task:`.

`[[task:label]]` names another task of the same project, resolved against `task_lookup` (the project's
tasks as `TaskReferenceView`, the reason `component` and `context` may use `task`). It becomes
`**Title** (task `task:label`)`, or `[[task:label]] (task not found in this project)`. The reference
also loads that task: `ContextService::render_task` collects the labels with `referenced_task_labels`
from the description and every checklist step's detail, and appends each named task once, in the order
first named, after the project in the record's `references`, so it renders as a nested `Task` record.
That record (`referenced_task`) carries the task's description, wrapup, notes and status, with a
`referenced` field saying it is background for the task above. Its checklist, flow, skills and commits
stay out, so its open steps are never read as the work, and its own `[[task:...]]` are expanded but not
followed: one level, no cycles. A task naming itself loads nothing more. `ContextSizeService` counts each
as a `Referenced task: Title` part.

The console draws the same syntax in read mode. `markComponentReferences`
(`rekall-ui/src/common/markdown/component-reference.ts`, registered in `common/config/markdown-editor.ts`)
turns `[[Name]]` into a chip that knows only the written name, with the backend's edge cases; markdown-it
never offers it code spans or fences. The project is not known to the global renderer, so
`AppMarkdownEditor` takes the project's `components` and `useComponentReferenceChips` settles each chip
(found with a folder and description title, missing, or plain for text with no project) after every render
and whenever the list changes. `useSelectedProjectComponents` loads the selected task's project into
`useProjectComponentsStore` and answers `null` until the list has arrived, so nothing is called broken
early. Write mode stays plain text, apart from the completion: `componentCompletion`
(`common/markdown/component-completion.ts`) is a CodeMirror completion source handed to `MdEditor` through
its `completions` prop. It matches `[[` plus what was typed, filters the project's components itself
(`filter: false`, name contains the text), skips code and fences through the syntax tree, and applies
`Name]]` through `view.dispatch`, replacing a closing `]]` already after the caret. `common/config/markdown-editor.ts`
adds `Tab` as `acceptCompletion`, which does nothing when no list is open.

Connections follow the same path. `markComponentReferences` renders a reference whose name starts with
`connection:` as a connection chip (`common/markdown/connection-reference.ts`), `AppMarkdownEditor` takes
the project's `connections` (`useSelectedProjectConnections`, `null` until the bindings have arrived) and
`useConnectionReferenceChips` settles each chip (found with kind, title and target, missing, or plain).
`connectionCompletion` is a second completion source on the same `[[` trigger: it lists the bindings after
the components, by kind (`sortedByKind`), labelled `connection:label` so typing either the prefix or the
label filters them, and applies `connection:label]]`.

Tasks follow it too, except for when the list opens: a project holds too many tasks to offer after a bare
`[[`, so `taskCompletion` (`common/markdown/task-completion.ts`) answers only once `[[task:` is typed,
filters the project's other tasks by label or title, and shows the title and status beside each.
`useSelectedProjectTasks` reads them from the console store, which already holds every task.
`markComponentReferences` renders `[[task:label]]` as a task chip (`common/markdown/task-reference.ts`),
painted in the accent colour with the title and status on hover. All three chip kinds are kept painted by
`useReferenceChips`, and all three completions write through `insertReference`.

The suggestion list is grouped into `Components`, `Connections` and `Tasks` sections (each option's
`section`), and each row starts with a glyph: `referenceCompletionIcons`
(`common/markdown/reference-completion-icon.ts`, added in `common/config/markdown-editor.ts`) merges an
`addToOptions` renderer into the editor's completion config, which draws the glyph named by the option's
`type` (`referenceCompletionType`): an outline cube for a component, an outline checkbox for a task, and
the connection's logo (`CONNECTION_KIND_PATH`, shared with `ConnectionKindIcon`) in its brand colour.
The editor's other suggestions keep CodeMirror's own icon. The card itself is styled in
`assets/markdown-editor.css`.

A note can travel by reference. It then arrives as its title, its first line and an anchor such
as `note:3f2a9c1e` (the first eight characters of its id), with `loaded="on request"`, and
`rekall_context` loads it in full by that anchor. `ContextRenderer`, which writes every context,
is shared with `ContextSizeService`, so the size the console shows for a task is the length of
what a session receives.

An anchor is `entity:value`. A value containing spaces is quoted. A bare term with no `entity:`
is looked up across both entities and accepted only when exactly one record matches; on more
than one the candidates come back and nothing is loaded. The tool does not score, weigh or
choose.

A note attached to several tasks arrives under each of their anchors. The same markdown showing
up twice in one session is the relation working, not a duplicate.

The two writes are narrower on purpose. `rekall_context` may be handed a company and will
happily return everything under it; a write cannot, because a project names forty tasks and
none of them is the answer. Anything that does not resolve to exactly one task is refused with
the form that would have worked, and an ambiguous bare label is reported rather than picked. The
`rekall_wrapup` description carries the state-not-process rule, because that description is the
only part of this system the model reads before deciding what to write; the `rekall_step`
description says the last tick is not its to make.

Responses are capped, with an explicit truncation notice rather than a silent cut.

## Two protocol eras

MCP has two revisions in play, split at `2026-07-28`. Up to `2025-11-25` a client opens with an
`initialize` handshake and the agreed revision holds for the session. From `2026-07-28` there is
no handshake and no session: every request carries its own revision, in the
`MCP-Protocol-Version` header and again in `params._meta`, along with its method in `Mcp-Method`
and, for a `tools/call`, its tool name in `Mcp-Name`.

| | Handshake era | Stateless era |
|---|---|---|
| Opens with | `initialize`, answered with the revision asked for | nothing |
| Unknown method | `-32601` on a 200 | `-32601` on a 404, so a probe can tell this from a wrong address |
| Unknown revision | — | `-32022` on a 400, listing what is supported |
| `server/discover` | answered | answered, and mandatory |
| Result envelope | a result is a result | every result carries `resultType`, and one without it is discarded |
| `tools/list` | the tools | the tools plus `ttlMs` and `cacheScope`, both required |

The last two rows are the ones that fail silently, and they fail in the same place. A `tools/list`
missing `resultType`, or missing either cache annotation, is not repaired or read leniently: it is
thrown away whole, and a client that cannot read the tool list registers no tools. The server
shows as connected, `/rk` is there, and there is nothing behind it to call. `server/discover` is
the exception that hides this, because its own schema defaults both annotations.

