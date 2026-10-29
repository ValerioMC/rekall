# Steps, flows, deployment graphs and epics

## 4.2 The steps

A task can be broken into steps, and each one is somewhere on a line: `DRAFT` while the
checklist owner is still wording it, `OPEN` once it is promoted and ready to work, `RUNNING`
while a session works on it, `CLAIMED` when that session says it is finished, `DONE` when a
person accepts the work. It exists because of a gap the other two markdown fields on a task leave
between them: the description is the brief and grows as the work is redefined, the wrapup is the
state of the implementation as prose, and neither says which parts are finished. Working that
out meant reading both and comparing them, which is slow by hand and a guess for a model. A row
with a state says it.

| Decision | Why |
|---|---|
| A table rather than checkboxes in the description | The description is one blob that Claude cannot rewrite and that carries no per-item state. A row can be ticked in one click, ordered, and rendered differently depending on where it is |
| `position`, dense from zero, maintained by `TaskStepService` | Every write that could leave a gap renumbers the list. A sparse ordering is correct right up until something reads it as an index. An `@OrderColumn` on the inverse side of the association was the alternative, and it is a column two writers can disagree about |
| `state` is an enum, not a boolean; `running_at`, `claimed_at`, `done_at` follow it in `TaskStep.markState` | The state and its moment are one fact, kept together the way the boolean and `done_at` were. Reopening a step clears all three: a reopened step has no history of being anything else. `completedAt()` is `claimed_at` first, then `done_at`, because a wrapup written right after a claim already accounts for the work and a later console tick does not change what it built |
| Capped at 20,000 characters, like a wrapup | A step whose detail runs past a screen is a task, and the model has a level for that |
| `ON DELETE CASCADE` on the task, like a wrapup | It describes one piece of one task and means nothing beside another |
| A session can write `RUNNING` and `CLAIMED`, never `DONE` | `rekall_step` moves a step as far as claimed, which is what lets a session drive its own checklist. The last tick is a person in the console saying they reviewed the work: a session claiming its own work accepted is what the `CLAIMED`/`DONE` split exists to prevent. The navigator's progress count is built on `DONE` alone, so it still means "accepted" |
| `DRAFT` is the creation default, and only the console leaves it | A new step is a line the checklist owner is still wording, not work: it is kept out of the `<steps>` block a session reads (counted only as `draft="N"` on the tag), `rekall_step` refuses to move it, `markRunning` skips it, and `reviewActive()` still treats a task that holds only drafts as stepless. `TaskStepService.edit` promotes `DRAFT -> OPEN` and sends an untouched `OPEN` step back; a step that has been started, claimed or done cannot return to draft, because the run behind it would be lost. `settleDraftsAtTail` keeps every draft after every non-draft in `position` order, so the numbering a session sees is the numbering the console shows. `stepCount` on `TaskResponse` counts non-draft steps; `draftStepCount` carries the rest |

The rendering is asymmetric, and that is the feature. An open or running step is written into
the context with its detail, because it is the work about to be done or being done now, and its
line is tagged `(in progress)` or `(claimed, waiting for the console to accept it)` so a session
that reloads mid-run does not start it again or redo it. A finished step is written as its title
alone: it needs no doing, and spending the window on the detail of finished work is how a load
costs twice what it is worth. A draft step is left out of the block entirely, present only as
`draft="N"` on the tag and a one-line `drafts` count in the field list. The counts,
including `running` and `awaiting-review` when they are non-zero, go in the field list ahead of
the block.

**The live loop.** `TaskStepService` publishes a `StepStreamEvent` after every write, from the
console or from MCP alike, carrying the affected task's whole checklist. `TaskReviewService`
publishes a `TaskReviewEvent` when a stepless task's review line moves, and `WrapupService` a
`WrapupStreamEvent` when a wrapup is written or deleted (the latter carrying the new
`WrapupView`, or a `deleted` flag). `StepEventStream` in `rekall-http` holds each until the
transaction commits and fans it out to every open console over `GET /api/steps/stream` as
Server-Sent Events under the frame names `steps`, `task-review` and `wrapup`; `useStepStream`
in the UI applies each to the store. A session moves a step to `RUNNING` over MCP and the
console animates the move without a reload: the checklist node breathes, and the branch feeding
it carries a band of light toward it. Opening a terminal on a step ("Run here") is the
other way in: `PtyTerminalManager` moves that step to `RUNNING` on start through
`TaskStepService.markRunning` and back to `OPEN` on close through `releaseRunning`, so the
launch animates whether or not the console is on the checklist pane, and a terminal that dies
without its step being claimed does not strand it. Both are no-ops unless the step is `OPEN`
(start) or still `RUNNING` (end), so an MCP `claimed` or a console tick made meanwhile is never
undone. There is one terminal per task: opening again while one is up is routed to it rather
than spawning another, and if it names a different step the manager releases the old one and
marks the new, so the running marker follows the terminal across a checklist the way it would
in one terminal window. A task with no checklist takes the same path at task scope:
`PtyTerminalManager` calls `TaskReviewService.sessionRunning(taskId, true)` on start and
`(taskId, false)` on close, so the review line runs while the terminal is open on the task and
no step. A wrapup written from a terminal-run session (over MCP) or by an MCP call lands in the
pane with the claim it triggers rather than on the next reload.

## 4.3 The flow

A checklist is an order, and some work is not: build, deploy, test, and a failed test goes to a
fix that goes back to deploy. A **route** (`step_link`) says that when one step ends with an
outcome the work moves to another step of the same task, and a task whose steps carry any route
is a **flow**. A **visit** (`step_visit`) is one entry of the flow into a step, numbered across
the task, closed with the outcome it left by. The visit left open is where the flow stands.

| Decision | Why |
|---|---|
| Routes on the steps, not a second kind of record | A flow is the checklist with its order made explicit. Every rule of 4.2 still holds per step (draft, claim, a person's tick), so a flow needs no new state line and a task can become one by drawing a single route |
| The outcome is a slug, unique per step it leaves | The session names it and the console shows it, so both read `app-error`. One outcome naming two routes would leave the session unable to say which it took. `step_link::normalise_outcome` folds what was typed the way `Slug.of` folds a label |
| A claim follows the route; a person draws it | `FlowService::transition` is the only move that routes, and it runs inside the claim's transaction: refused routing rolls the claim back. Routes are written by `StepLinkService`: the console draws them on any step, and a session can only propose them between drafts (`propose_in`, refusing a released step at either end). `rekall-mcp` reads released routes in the context and follows them, never edits one |
| A flow is planned whole, as drafts | `rekall_propose_flow` hands `FlowProposalService` a `FlowPlan` (drafts in order, routes by position or to End), and `FlowPlan::check` refuses it before anything is written when a step is unreachable from the first, a loop has no way out to End, a step routes to itself, an outcome repeats on a step or a step has more than eight routes. It lands only on a task with no steps, because Start leads to the first step and unrouted steps already there would end the flow. Routes leaving a draft do not make a flow: `StepLinkService::is_flow_in`, the context and `readFlow` count released steps only, and the shelf names each draft's routes (`StepDraftRow`) for review before promotion |
| Arriving reopens finished work | A route back to a claimed or done step means another pass, so `TaskStepService::reopen_for_route_in` returns it to `OPEN`. That is the one way a session's move can undo a tick, and only along a route a person drew. A route into a draft is refused when the flow reaches it |
| One open visit per task | Entering a step leaves the current one without an outcome (`StepVisitService::enter_in`), so "where the work stands" is one row, never a guess between two. A claim on a step never started enters it first, so every pass is counted |
| A claim's report is read by the step it routes to | The step a failure sends the work to cannot be written for every failure in advance, so what was seen travels as data: `FlowService::transition` keeps the claim's `report` on the visit it closes (`text_rule`, `REPORT_MAX_CHARACTERS`, refused off a claim on a flow), and `render_arrival` writes it under the current step when the visit before it carried one. Only the current step shows it, since that is the pass that works from it |
| Visits travel on the step views | `TaskStepView` carries its routes and visits, so the `steps` frame and `/api/steps` already redraw the graph live and the console derives the trail, the counts and the current step itself (`common/flow/step-flow.ts`) |
| Start and End are not records | Start always leads to the first step on the checklist and is drawn by the console alone. End is a route whose `to_step_id` is null (`RouteTarget::End`), so it needs no step and no state: a claim by that route closes the visit and opens none, `FlowRoute::Ended` carries the outcome, and the flow reads as finished because no visit is open. A step with no routes on a flow is drawn joined to End without a stored route |
| Where a step was dragged is stored on the step | `task_step.flow_x` and `flow_y` (`PATCH /api/steps/{id}/flow-place`, `DELETE /api/tasks/{id}/flow-places`) are pure layout: nothing reads them but the canvas, and clearing them hands the steps back to dagre. `FlowPlace` refuses a place beyond 20,000 pixels |
| A flow runs unattended | A claim that follows a route answers with the step to start and says no review is awaited; the wrapup is asked for only when the flow ends. `.claude/commands/rk.md` carries the same rule, so a session never stops between moves to ask for a tick |
| A spent visit budget stops the flow | `StepVisitService::enter_in` is the one place a new visit is made, so it refuses one past `task_step.max_visits` (`admits_visit`). The refusal rolls back the claim that routed there: nothing moves, the session is told to stop and report. Re-entering the step the flow already stands on is not a visit. Raising the budget (`TaskStepService::limit_visits`, `VisitBudget` 1 to 99) or a reset lets it go on |
| A session ends where a step opens a new one | `task_step.starts_session` marks a step as the first of a session; the first step of a checklist opens the first whatever it says, so a checklist with none marked is one session. `TaskWorkService::read` answers `next_opens_session` (the step to work next, by the flow's open visit or else the first open one, is marked and not first) and `last_settled_at`, and `TaskWork::session_is_spent(started_at)` is the one rule: work is left, nothing runs, the next step opens a session and a claim came after this terminal started. Both consumers apply it after the settle grace: `Worker::review` for an item the run queue runs (after its usage ceiling), and `SessionRelay` for a terminal opened by hand, which leaves a task the queue is running to the queue. Either calls `PtyTerminalManager::renew_session`, which closes the terminal and opens a `Work` one on the task with the same choices; only a terminal that follows steps can be renewed. A session that starts the next step itself is left alone, because the rule is read again after the grace. `rekall_context` renders the boundary and `.claude/commands/rk.md` tells the session to stop at it. Marks are set by `rekall_propose_step` / `rekall_propose_flow` (`new_session`) and by the console (`TaskStepService::begin_session`, `PATCH /api/steps/{id}/session`) |
| Reset forgets the visits, not the states | The states are a person's review; the trail is a count. `FlowService::reset` clears one, never the other |

A claim chooses its route by one rule (`flow::choose_route`): the outcome it names; with no
outcome, the step's only route, or the end of the flow when it has none; several routes and no
outcome is refused with their names, because only the session knows what happened. An outcome on
a plain checklist, or on a move that is not a claim, is refused rather than ignored.

The context marks a flow as `flow="true"`, `current` and `moves` on `<steps>`, `(current, visit N)`
on the step, a `Routes:` line under every step (finished ones too, since a route can return to
them) and a `flow` field naming the step to work. The console draws it in `StepFlowView`: dagre
lays the steps out top to bottom in checklist order (`common/flow/flow-layout.ts`, with unseen
edges keeping the spine), walked routes solid and the rest dotted, the route the last move came
in by animated toward the lit current step, the trail of moves above it. Start and End are drawn
at both ends, a dragged step overrides its dagre place (`placeSteps` redraws only the routes that
touch it between the boxes), and `composables/useFlowDrag.ts` owns the pointer work: moving a
step, and drawing a route from a step's handle onto another step or End.

## 4.4 Deployment graphs

A flow drawn on one task is drawn again on the next release. A **deployment graph** is that flow kept
on the project instead: steps routed by outcome, each reaching one of the project's connections,
copied onto a task when the task is to be released that way.

| Decision | Why |
|---|---|
| Its own three tables, not template steps on a task | A graph belongs to the project and outlives every task it is laid on; a step row carries a task's states and visits, which a template has none of |
| Written whole, by `DeploymentGraphService` in `rekall-console` | The editor holds the graph as one draft and a route names its target by the caller's `key`, so a step needs no id before it is saved. `GraphDraft::of` resolves the keys, folds outcomes to slugs and runs `FlowPlan::check_shape`; `replace` deletes the steps (their routes cascade) and writes the draft again. It is a catalog write, so `rekall-mcp` cannot reach it |
| Shape on save, the rest on attach | `FlowPlan::check` is split: `check_shape` (size, distinct titles, routes that fit) refuses a save, while reachability and a way out of every loop only decide whether the graph can run. `StoredGraph::problem` names the first reason it cannot, the console shows it, and a graph with no routes runs as a checklist |
| A step reaches only what its project binds | The binding is what lists the connection in `<connections>`, so a step on another connection would name something the session cannot reach. A connection unbound later is kept on the step and reported, a deleted one is set null |
| Attaching copies | `GraphAttachService::attach` lays each graph step as an `OPEN` step of the task (`add_open_in`, `limit_visits_in`, `reach_connection_in`) and each route through `StepLinkService::add_in`, in one transaction, onto a task of the same project. The task then owns its flow: editing the graph changes no task already laid, and a run's states and visits stay on the task |
| The graph goes after the work | A graph releases work, so laying it in place of the checklist left the session with nothing that said to build anything. A checklist is chained by `done` and its last step leads into the graph's first; a drawn flow has its routes to the end retargeted there (`StepLinkService::retarget_in`) and its steps with no route out given `done`; a task with no steps gets `IMPLEMENTATION_TITLE` first. When the first step is finished and nothing is current, the flow is entered at the first unfinished step, since a flow begins at its first. Refused while drafts wait (no route would reach them) and on a title the task already has, which is what refuses a graph laid twice |
| The release graph is laid when the console runs the task | A project marks at most one graph `release_default` (`DeploymentGraphService::choose_release`, which clears the others). `GraphAttachService::release_on_run` lays it as `attach` would, unless the task already carries `graph_laid_at` (set by every lay, by hand or not, so a task is laid once) or `release_skipped`. The console calls it from **Run here** and the open-in-terminal button before the session starts (`useReleaseOnRun`), and a refusal is told without stopping the run. A session started outside the console never lays it: the graph is a catalog write, which `rekall-mcp` cannot reach, so laying stays a console act |
| A step's place is saved with the graph | The editor's canvas is the task flow's (`layoutFlow`, `placeSteps`, `useFlowDrag`), so a dragged step keeps its place and the routes touching it are redrawn between the boxes. The place travels in the whole-graph write as `flowX`/`flowY`, both or neither, within `FlowPlace`'s reach, and **Tidy layout** clears them all in the next write. It stays on the graph: attaching lays the task's steps out afresh |
| The step names its connection in the context | `ContextService` resolves each step's `connection_id` to a `StepConnectionView` (label, still bound or not), and the renderer writes a `Connection:` line under every step it renders with its detail. How to reach it stays in `<connections>`, so the credential rule is written once |

## 4.5 Epics

A feature often outgrows the task planned for it, and its tasks multiply. An **epic** is the task
they share: `kind = EPIC`, a description and notes that hold the concept, and the tasks of its
project whose `epic_id` names it.

| Decision | Why |
|---|---|
| A kind of task, not a table of its own | An epic needs exactly what a task has (label, anchor, status, description, notes) and none of a second record's plumbing. `epic_id` points at a `task` row, so the anchor grammar, the note links and the console's selection all serve it unchanged |
| One level, one project | `task::Model::require_joinable` refuses an epic under an epic, a task as its own epic, a plain task as an epic and an epic of another project. Two levels would make "what does this task inherit" a walk, and a cross-project epic would hand one project's notes to another's sessions |
| An epic is never worked | It is a container for a description. `task::Model::require_workable` is the one rule, checked where work starts: `TaskStepService::insert_in` (no step, so no checklist and no flow: the console, `rekall_propose_step` and `rekall_propose_flow` are all stopped there), `GraphAttachService::lay_in` (no deployment graph, attached or released on run), `TerminalLaunchService::resolve` (no terminal) and `RunQueueService::add` (no queued run). The console hides the steps pane, the terminal pane, the run and Claude Code launch buttons and leaves epics out of the queue picker |
| Its kind and project are kept while tasks belong to it | `TaskService` refuses turning an epic with tasks back into a task, or moving it to another project, as a 409: either would leave tasks pointing at a plain task or across projects. A task that already has steps cannot become an epic |
| Deleting it frees its tasks | `ON DELETE SET NULL` on `epic_id`: the tasks are the work and outlive the container |
| Inherited in the context, not copied | `ContextService::render_task` adds an `epic` field to a task that belongs to one and appends the epic as a reference (`inherited_epic`) with its description and notes in full and its other tasks as anchors, without its skills, commits or wrapup. An epic loaded by its own anchor renders as `## Epic:`, with a `kind` field counting its tasks. Nothing is written into the child, so editing the epic reaches every task at once |

In the console an epic is selected like a task and opens on `EpicPane`, which takes the
description pane's place for it: its tasks' progress, the tasks (filed, created or taken out
through `setTaskEpic` on the ordinary task update, so no endpoint of its own) and its description,
written with the same autosave as a task's (`useDescriptionDraft`). The dossier shows one
`EpicDossierCard` in place of the description and steps cards. The navigator keeps epics on an `epics`
`NavigatorFilingDrawer` ahead of the status groups (`NavigatorEpicRow`, from `partitionTasks`), closed
until opened by hand or by selecting an epic (`useNavigatorShelves`), each with the status mix of its tasks;
a task in an epic wears an `EpicChip` beside its anchor, on a row whose third line carries what the
task holds, and its dossier opens with an `EpicMembershipCard` that selects the epic.

