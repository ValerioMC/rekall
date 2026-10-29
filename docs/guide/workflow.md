# Wrapups, steps, flows, commits and review

## Wrapup

A task has at most one wrapup: what its implementation currently is, not a changelog. A session writes it fresh every time from the code, keeping only what is still true and needed to start work. The kind of information picks the form: an opening sentence on what the task delivers, a table for the pieces of the implementation, a bullet per rule, a table for results of checks, a numbered list for a flow, open points last. It names code by the code's own names (class, file, endpoint, table) and does not transcribe field lists or signatures. It aims at about 300 words; past 600 the tool's answer asks for a shorter rewrite, though the write still lands. It is capped at 20,000 characters, against 100,000 for a note.

A quoted term after `wrapup` is a directive on what to write. It can narrow the subject, dictate the wording, or set the language or length.

```
/rk project:vega task:report-builder wrapup "export module only"
```

The description pane and the steps pane both carry a **Generate the wrapup every session** toggle under the header, so it is set from whichever surface the work is driven from. Turning it on reveals an optional directive field. Set once on the task, the toggle stands in for the quoted term: the context load then tells the session a wrapup is expected every time without being asked, and the words it should follow. Turning the toggle off drops the directive with it.

A wrapup written after a step finishes folds that step's work into the same description. The console counts steps ticked and notes added since the wrapup was last written. You can edit the wrapup in the console; the next `/rk … wrapup` replaces it and the tool reports when it overwrites a hand edit, because nothing keeps the text it replaces. Neither the wrapup nor the description keeps earlier versions: a write replaces the text outright and a delete removes it.

### Notes written by a session

A session can keep what it produced as a note on the task, rather than in the wrapup. Ask for it with a quoted term after `note`, saying what the note has to hold:

```
/rk project:vega task:report-builder note "the export formats we settled on"
```

The session writes the note and calls `rekall_note`, which creates it on that task, in full, with kind `notes`. It then travels with the task's context and shows up in the console without a reload. A step or a description that asks for its output to be kept as a note, or a task whose whole point is to write one, gets the same without the `note` term. A title the task's notes already carry is refused, unless the session passes `replace`: then that note's body is rewritten in place, its title and placements kept, so a runbook or a table of results stays one current note instead of piling up versions. The old body is not kept anywhere, and a note on several tasks changes on all of them; the tool's answer says how many. Sharing, detaching or deleting a note stays in the console. Writing a note does not claim the task or a step.

## Steps

A step sits on a line: **draft** while you are still wording it, **open** once you promote it and it is ready to work, **running** while a session works it, **claimed** when the session reports it finished, **done** when you accept it. Each step is a title and an optional markdown detail. The pane opens on the first step whose work is not finished.

A new step is created as a draft and sits on a staging shelf below the checklist, off the rail. **Promote** moves it onto the list; an open step not yet started can go back with **To draft**. A draft is not work: it stays out of the checklist a session reads, `rekall_step` will not move it, and the navigator's step count ignores it. A task that holds only drafts is still treated as having no checklist.

A session drives its own checklist over `/rk`:

```
/rk project:vega task:report-builder step:3 start   # step 3 -> running
/rk project:vega task:report-builder step:3 done    # step 3 -> claimed
/rk project:vega task:release step:3 done outcome:app-error   # on a flow, follow that route (see Flows)
```

The console holds one `text/event-stream` connection (`GET /api/steps/stream`) carrying six frames: `steps` for a checklist, `task-review` for a stepless task's review line, `wrapup` for a wrapup write or delete, `commit-reference` for a commit logged against a task or step, `run-queue` for the run queue, and `note` for a note a session wrote or rewrote. A step moved from an in-app terminal, a box ticked in another window, a wrapup or a note written over MCP, or a commit logged by a session all land without a reload.

Claude receives an open or running step with its detail, tagged `(in progress)` or `(claimed, …)`. A finished step arrives as its title alone. A draft step is not sent at all, only counted as `draft="N"` on the `<steps>` tag. A step ticked without a following wrapup is marked `(finished since the wrapup was written)` and handed back with its detail until the next wrapup folds it in.

With steps on a task, the open steps are the work and the description becomes the constraints the steps are built against. Anything the description asks for that no open step covers is not built; `/rk` flags it and asks for the step.

Only you set a step to **done**. `rekall_step` stops at `claimed`. The navigator's progress count is built on `done`.

The mark in front of each navigator row says where an in-progress task's work stands. A hollow amber ring around a small light means nothing is handed in yet, and a green arc on that ring is the share of the checklist already accepted. An orbiting comet means a session or a timer is on the task now; it is the only mark that moves. When the timer runs, no session is at work and a claim is waiting for you, the comet keeps orbiting but an amber check replaces the light at its centre: something is running, and it needs your review to go on. A session still working a step keeps the plain comet, whatever it already claimed. An amber check in an open ring means work is claimed and waiting for your review. A filled green seal means every piece of work is accepted and the task is ready to move to Done. Tasks in any other status keep a plain dot in their status colour. Hover the mark to see its meaning in words. The selected row scrolls to the middle of the list when it is out of sight, and a gold comet circles its edge, brightest at the head and fading along its tail, so it is easy to find; with reduced motion set the comet is hidden.

### Flows

A checklist runs top to bottom. When the work branches or loops (build, deploy, test; a failed test goes to a fix that goes back to deploy), give the steps **routes**: each says that when one step ends with an outcome, the work moves to another. A task whose steps carry at least one route is a **flow**.

Switch the Steps pane to **Flow** in its header; a task with routes opens on it. The graph draws every step laid out by dagre, top to bottom in checklist order, and every route labelled with its outcome. A route the work has taken is solid, one not yet taken is dotted, and the route the last move came in by carries a light toward the step the work stands on, which is lit and tagged **here**. Each step shows how many times the work entered it (`×3`). **Start** always leads to the first step; **End** is where the work finishes, and it lights when it gets there. Point a route at End (pick *End of the flow* as its target) and a claim by that outcome finishes the flow, leaving no step current. A step with no routes on a flow is drawn joined to End too.

Steps **drag** on the graph: press a step and move it, and the place is kept (**Alt** with an arrow nudges the selected step from the keyboard). **Tidy layout**, shown once a step has been moved, hands every step back to the automatic layout. To add a route without the form, drag from the handle under a step onto another step or onto End; the route form opens with that target filled in and asks for the outcome. The strip above the graph is the trail: every move in order with its number and the outcome it left by, the open one last; an outcome left with a report is underlined and shows it on hover. Pick a step on the graph or the trail to open it beside the graph: its routes out, edited in place (an outcome renamed on blur, a target picked from the list, **Add**, the cross to remove), the routes that lead into it, every visit with how it left, and its **visit budget**: how many times the flow may enter it (1 to 99, empty for no limit). The node shows the count against it (`×2/3`), in red once it is spent. **Detail** opens it on the list to write its detail. **Reset moves** forgets the trail and the counts; the steps keep their states and routes. A press on the empty canvas and a drag pans the graph.

A session works a flow over the same `/rk` lines, adding what happened when it claims:

```
/rk project:vega task:release step:3 done outcome:app-error   # step 3 -> claimed, the work -> the app-error route
```

`rekall_step` with `claimed` and an `outcome` closes the visit on that step, follows the route that outcome names, opens the step it leads to (again, if it was finished) and makes it current; the answer names it and the line that starts it. A flow runs unattended: that answer tells the session to start the next step at once and to write the wrapup only when the flow is finished, and `/rk` says the same, so no step waits for a tick between moves. A move into a step whose visit budget is spent is refused and rolls the claim back: the flow stops where it stands, the session reports what keeps sending the work back, and you raise the budget or **Reset moves** to let it go on. A step with one route needs no outcome, a step with no routes ends the flow, a route to End ends it with its outcome, and an outcome that names no route is refused with the routes there are. A claim can carry a `report` (markdown, up to 4,000 characters) of what the session saw: it is kept on the visit the claim closes, and the step the flow goes to shows it in its context in an `<arrived-from>` block naming the step and outcome it came from, so a fix works from what failed rather than from a session's memory. A report off a claim on a flow is refused. Starting another step leaves the current one without an outcome. An outcome is a slug, so `App error` is stored and matched as `app-error`; one outcome names one route per step, a route never leads back to its own step, and a step holds at most eight. The context marks a flow with `flow="true"`, the current step and the move count on `<steps>`, `(current, visit N)` on that step (`visit N of M` with a budget, and `(entered N of at most M times)` on the others), and a `Routes:` line under every step, finished ones included. A person writes routes from the console (a session can only propose them between drafts, see [Planning](#planning)): `POST /api/steps/{id}/links`, `PATCH` and `DELETE /api/step-links/{id}`, each answering the task's whole checklist; `POST` and `PATCH` take `endsFlow: true` in place of `toStepId` for a route to End. `PATCH /api/steps/{id}/max-visits` (`maxVisits`, 1 to 99; null lifts it) sets the budget. `PATCH /api/steps/{id}/flow-place` (`x`, `y`; both null clears it) keeps where a step was dragged and `DELETE /api/tasks/{id}/flow-places` clears them all. `DELETE /api/tasks/{id}/visits` is the reset. Both travel on the step views, so the `steps` frame redraws the graph live.

### Deployment graphs

A project keeps **deployment graphs**: the route its work takes from a commit to a running application, written once and run on any task. Each is a flow kept on the project instead of on a task: steps routed by outcome, each reaching one of the project's connections. They are listed on the project page's **Deployment graphs** tab, beside the one open. A project without one asks there whether to set one up, and **Not now** folds the question away for that project in this browser.

**Set it up** (or the **+**) offers two starts. The **full release loop** is Push commit, Pipeline build, Deploy, Check and Fix, each with its instructions written, the first four on the project's first connection of its kind (GitLab for the first two, Kubernetes for the others). Push only pushes what the work already committed. A failed build or deploy and an unhealthy check go to Fix, which repairs the cause in the repository and goes back to the push (`fixed`), or ends the flow for you to decide when the repository cannot fix it (`blocked`); Fix may be entered three times. A **blank graph** is one step to draw from. Either opens in the editor: the graph on a canvas, Start to End with every route labelled by its outcome and the routes that loop back drawn dashed in amber, and beside it the picked step: its title, the connection it reaches (only the project's bindings are offered, a missing credential flagged), what the session does there, its routes out and its visit budget. **Add** inserts a preset step after the picked one, routed from it by `done` when it routed nowhere. A step drags to where you want it (**Alt** with an arrow nudges the picked one from the keyboard), its routes redrawn to follow it, and the place is saved with the graph; a step never dragged is laid out automatically, and **Tidy layout**, shown once a step has been moved, hands every step back to it. Dragging from a step's handle onto another step or End adds a route with a free outcome, focused to be named. Every edit is saved on its own a moment later. A refused one stays on screen with the reason. A graph the flow rules would refuse (a step never reached, a loop with no way out) is still saved and says why it **can't run yet**; one with no routes runs as a checklist.

A task lists its project's graphs above its empty checklist, or below its checklist while no step of it reaches a connection and no draft waits. **Use this graph** lays one after the task's own work: each graph step becomes an open step of the task with its detail, its visit budget and its connection, and each route a route between them. A plain checklist is chained in its order by `done` routes and its last step leads into the graph's first; a flow you drew has every route that ended it, and every step with no route out, lead there instead. A task with only its description gets an **Implement the task** step first, whose detail makes the description its brief. When the checklist's first step is already finished, the flow stands on the first unfinished one, so a task worked as a checklist is released from the push. The task is then a flow a session starts at once.

A project can make one graph its **release graph** with **Lay on every task the console runs** in the graph's editor; ticking it on another graph moves the choice there. Then **Run here** and the open-in-terminal button lay it on the task, as **Use this graph** would, before the session starts, and say so. A task is laid once, by hand or this way, so running it again lays nothing. Above its checklist the task says the graph will be laid when it runs, and **Don't release this task** leaves it unreleased (**Release it when run** undoes that). A graph that cannot be laid (drafts still waiting, a graph that cannot run) is reported and the session starts without it. A session started outside the console never lays it: laying a graph is a console act. `PUT /api/projects/{id}/release-graph` (`graphId`, null for none) chooses it and answers the project's graphs, with `releaseDefault` on each; `POST /api/tasks/{id}/release` lays it when it should and answers `laid` (the graph's title, or null) and the task's checklist; `PUT /api/tasks/{id}/release-skipped` (`skipped`) answers the task, which carries `graphLaidAt` and `releaseSkipped`. The task keeps its copy: editing the graph later changes no task it was laid on. In the session's context each such step names its connection (``Connection: `prod`, reached as its line in `<connections>` says.``), or, when the project no longer binds it, says to report it rather than reach the system another way; the flow canvas shows it on the step. `GET` and `POST /api/projects/{id}/graphs`, `PUT` and `DELETE /api/graphs/{id}` write a graph whole: `title`, `description` and `steps`, each with a `key` its routes name in `toKey` (none for the end of the flow), `connectionId`, `maxVisits` and its canvas place in `flowX` and `flowY` (both or neither); the answer carries `problem` while the graph cannot run. `POST /api/tasks/{id}/graph` (`graphId`) lays it on the task and answers its checklist: refused with 409 while the task has drafts or a step under one of the graph's titles (which is also how a graph laid twice is refused), and with 400 for another project's graph or one that cannot run. No MCP tool writes a graph.

### Planning

`/rk project:vega task:report-builder plan` has a session turn the task into a checklist for you to review. It reads the description, the wrapup, the finished steps and the code the task touches, then calls `rekall_propose_step` once per step, in order: a title saying what the step delivers, a detail saying what to build, where, what it must satisfy and how you can tell it is done. Every proposal lands as a draft on the staging shelf, so nothing is work until you promote it, and the session builds nothing. A title the task already has is refused, so a second `plan` adds only what the first one missed; a task holds at most 20 drafts.

When a step's outcome decides what comes next (tests that can fail, a deploy that can break, a review that can reject), the session plans a flow instead: one `rekall_propose_flow` call with every step in order and its routes, each `{outcome, to}` naming another step of the plan by number or `end`, and an optional `max_visits` per step. Type `/rk … plan flow` to ask for a flow outright. The plan is checked whole before anything lands: every step reachable from the first, no loop without a way out to End, no step routing to itself, one route per outcome and at most eight per step, no title twice. A refusal names what to change and leaves the shelf empty. A flow is proposed only onto a task with no steps, drafts included, since the flow begins at the first one. Each draft on the shelf lists its routes (`app-error → Fix the application`); the task is not a flow, and no session follows those routes, until you promote the drafts.

The Steps pane starts it from **Plan here** in its header, with or without a session already running. While one runs on the task in the console's terminal, the button types the command into it. Without one, it opens a terminal in the project's folder that starts on `/rk … plan` instead of the plain `/rk`, so the session plans the task rather than working it; the project needs its folder set, as for **Run here**. Either way the drafts land on the shelf for you to reword, promote and then run. While the task has no steps and no drafts, the only state a flow can land on, **Plan a flow** sits beside it and does the same with `/rk … plan flow`. An empty checklist also shows the `/rk … plan` and `/rk … plan flow` chips, to copy into a session outside the console.

A claimed step is reviewed from its detail: **Accept** ticks it to done, **Send back** returns it to open for another pass. Sending back seals the detail that pass worked from above the editor (`POST /api/steps/{id}/send-back`), and the detail starts empty for the feedback. `/rk` hands the next session each earlier pass in a `<previous-pass>` and the feedback in a `<feedback>`, so it builds on what was done instead of reading the feedback as a new brief. The **N awaiting review** count in the pane header jumps to the first one. The step node itself only moves a step forward, so a stray click never walks it back; reopening an accepted step is a separate **Reopen** button that arms before it fires.

## Epics

An epic is a task of kind **Epic**, chosen in the task dialog. It holds no steps: it carries the
concept a feature shares, as a description and notes, and gathers the tasks of its project that
name it in their **Epic** field. Each of those tasks inherits it: `/rk` on one of them loads the
epic after it, in full, marked `inherited`, with its description and notes as part of the brief.

- The navigator keeps a project's epics on a shelf above its tasks that starts closed, each with the
  status mix of its tasks, and opens it when an epic is selected elsewhere; a task in an epic wears the epic's name next to its anchor, and the card above its dossier opens the epic.
- Selecting an epic opens it in the console: how far its tasks are, the tasks themselves and its
  description, written in place. **Add existing task** files a task of the project under it (one in
  another epic moves here), **New task** creates one already filed, and a row's cross takes a task
  out, with an Undo. In its dossier one **Epic tasks** card, with how many are done, stands where a
  task has its description and steps; its notes are listed below as on any task.
- A session creates one with `rekall_create_task` and `kind: "epic"`, creates tasks under it with
  `epic`, and files an existing task under it with `rekall_join_epic`.
- An epic belongs to no epic, takes no step, has no flow, is never run (no terminal, no Claude
  Code launch, no run queue) and is never laid on a deployment graph. A task with steps cannot
  become one, and an epic with tasks cannot turn back into a task or move to another project.
  Deleting an epic leaves its tasks on their own.

## Commits

A task, or one of its steps, keeps a ledger of the commits that built it: the hash, the commit's own subject line as the comment, and the diff it introduced, read from the project's **repo folder** (set on the project's page). The console shows the ledger under the description as a collapsible rail, and under each step's detail; a row opens to its diff, and can be deleted by hand.

**Log commit** on the description, or inside a running or claimed step, logs the tip of the repo. The chevron next to it opens a picker over the last 30 commits, newest first, with their subject and age, so an earlier commit can be logged against the task or step instead; rows already logged there are marked. A hash the log does not reach can be pasted in the same panel, abbreviated or full. A hash that names no commit is refused and nothing is written. Logging the same commit against the same task and step twice is a no-op.

A session does the same over MCP right after `git commit`:

```
rekall_record_commit  anchors="project:vega task:report-builder"                 # the tip
rekall_record_commit  anchors="project:vega task:report-builder" step="3"        # against step 3
rekall_record_commit  anchors="project:vega task:report-builder" commit="a0fd5cc" # an earlier commit
```

A logged commit stays out of `rekall_context` until it is chosen for it. Each row carries an **add to context** toggle (shown on hover; **in context** once on) and the rail fills the node of every chosen commit and counts them. A chosen commit travels with the task's context as a `<commit hash="…" step="…">` element inside `<commits>`: its subject, then its diff (capped at 60,000 characters), oldest logged first. Choosing is console-only; `rekall_record_commit` logs but never chooses.

The REST side is `GET /api/commit-references`, `POST /api/tasks/{id}/commit-references/latest`, `POST /api/tasks/{id}/commit-references` (body `commitHash`, optional `stepId`), `GET /api/tasks/{id}/recent-commits`, `GET /api/commit-references/{id}/diff`, `PATCH /api/commit-references/{id}` (body `inContext`) and `DELETE /api/commit-references/{id}`.

### Commit on claim

A project can commit for the session instead of waiting for it to. The **Commit on claim** switch sits under the **Folder** field on the project page's **Settings** tab and only arms when that folder is a git repository: the strip under it says which branch is checked out and whom git would commit as there (`user.name` / `user.email` as `git config` resolves them in that folder, so the global identity unless the repo overrides it). Outside a repository the switch stays off and says why; on a repository with no `user.email` it arms but warns that the commit will fail until one is set. The setting is saved on the project (`autoCommit` on `PUT /api/projects/{id}`; the server keeps it off whenever the folder is not a repository) and `GET /api/projects/{id}/repository` is what the strip reads.

With it on, a claim commits: `rekall_step … state="claimed"` stages everything in the folder (`git add -A`), commits it, and logs that commit against the step; on a task with no checklist, the Claude-authored `rekall_wrapup` that claims the task does the same against the task. A task with a checklist never commits on its wrapup.

The message says what the work does, never which files it touched. The session that claims writes it, as `commit_message` on `rekall_step` or `rekall_wrapup`: a Conventional Commits subject under 72 characters, a blank line, and a few sentences of body. Without one, Rekall derives it: the subject is the step or task title, typed `fix:` or `refactor:` when the title says so, `docs:` when every file is documentation, `test:` when every file is a test and `feat:` otherwise; the body is the opening of the step's detail (or of the wrapup, for a task with no checklist) as plain prose, headings, emphasis and code dropped, cut at a sentence. Either way the body is wrapped at 72 columns and ends with a `Refs: project:… task:…, step N` line. A clean tree commits nothing and says so; a git refusal (no identity, a hook, a folder that stopped being a repository) is reported in the tool's answer and the claim stands either way. `rekall_context` marks such a project with an `auto-commit` field telling the session not to `git commit` itself.

## Description review

A task with no checklist walks the same line at task scope: **open**, **running** while a terminal is open on it with no step target, **claimed** when a Claude-authored wrapup lands, **accepted** when you accept it in the console. Nothing new is typed for it: running follows the terminal and claimed follows the wrapup write. The description pane shows the running pill and a review bar: on **running** it carries **Accept** and a link to the wrapup, so a session driven by hand still has a console exit; on **claimed** it adds **Send back** (with an optional note the next session sees); accepting offers to also mark the task done. A note left on send back rides in `rekall_context` as a `review` field on the task (`sent back — "…"`), so the next session opened on that anchor reads why, not just that it was. It arrives on the same `GET /api/steps/stream` connection as a `task-review` frame, the wrapup that claims it rides the same connection as a `wrapup` frame so the pane shows the new text with the claim rather than on the next reload, and `PATCH /api/tasks/{id}/review` is the console-only Accept / Send back. Adding a first step retires the task-level line and the checklist takes over.

## Review queue and notifications

**Review** (the checklist icon with a count, in the top bar) counts everything a session claimed and you have not reviewed, across every task: claimed steps, and tasks with no checklist whose wrapup claimed them. It lists them longest-waiting first, and a row opens the step on its steps pane, or the task on its description's review bar. It is built in the console from the same step and review frames the event stream already carries, so it moves as sessions claim and you accept, and it is not there while nothing waits.

When something joins the queue while the console is hidden or another app has the focus, Rekall posts a system notification: through the native bridge in the desktop app (`rekall-app/desktop/src/bridges/notifier.rs`, the system asks for permission the first time), through the browser's Notification API elsewhere. What was already waiting when the console loaded never notifies. **Settings > Notifications** turns it off; the choice is stored on the machine, and in a browser that is also where permission is asked, since a browser grants it only from a click.

