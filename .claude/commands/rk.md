---
description: Load a Rekall working context by anchor, e.g. /rk project:vega task:report-builder
argument-hint: "company:|project:|task: <label> ...  [wrapup [\"how to write it\"]] | [note \"what to keep\"] | [section \"Title\" [\"what it holds\"]] | [plan [flow]] | [generate [kinds:sequence,state] \"what to diagram\"] | [explain \"what to walk through\"] | [inventory] | [assess [advisory:GHSA-xxxx-xxxx-xxxx]] | [step:N start|done [outcome:<name>] [report:\"what you saw\"]]   (labels, not titles)"
allowed-tools: mcp__rekall__rekall_context, mcp__rekall__rekall_wrapup, mcp__rekall__rekall_step, mcp__rekall__rekall_propose_step, mcp__rekall__rekall_propose_flow, mcp__rekall__rekall_record_commit, mcp__rekall__rekall_note, mcp__rekall__rekall_project_section, mcp__rekall__rekall_diagram, mcp__rekall__rekall_inventory, mcp__rekall__rekall_vulnerabilities, mcp__rekall__rekall_assess_vulnerability, Read, Grep, Glob
---

The arguments are:

$ARGUMENTS

If the terms include the bare word `generate`, follow **Generating**. If they include the bare word
`explain`, follow **Explaining**. If they include the bare word `inventory`, follow **Inventorying**.
If they include the bare word `assess`, follow **Assessing**. If they include the bare word
`wrapup`, follow **Wrapping up**. If they include the bare word `note`, follow **Noting**. If they
include the bare word `section`, follow **Sectioning**. If they include the bare word `plan`, follow
**Planning**. If they include a `step:` term alongside `start`
or `done`, follow **Stepping**. Otherwise follow **Loading**. Words inside double quotes are never
terms: `generate "how the plan is noted"` is generating, not planning or noting.

## Loading

Call `rekall_context` with `anchors` set to exactly the arguments above, unchanged.

- Do not call any other tool first. The anchors are already qualified, there is nothing to look up.
- The entities are `company`, `project` and `task`. If one was wrong, say which and stop.
- A note that arrives `loaded="on request"` is a title, a line and a `note:` anchor. Load it with
  `rekall_context` and that anchor only if the work needs it.
- The value is the record's **label**, never its title: `project:vega`, not `project:"Vega Platform"`.
  A label is lowercase and has no spaces. If I gave you a title, say so and stop rather than guessing
  the label from it.
- If a term matched more than one record, show me the candidates and stop. Do not choose.
- A `<skills>` block lists the Claude skills assigned to the task, to its tag or to one of its steps.
  Invoke each one with the Skill tool before you start working and follow it. A skill scoped to a step
  applies only while you work that step.
- A `<connections>` block lists the systems the project reaches (a Kubernetes cluster, a GitLab
  instance) and the one way to reach each. Use only what its lines say, and only in a terminal Rekall
  opened. Never print, echo or write a credential or a kubeconfig, never read the files or environment
  variables behind Rekall's helpers, and never call a credential helper or Rekall's credentials endpoint
  yourself. A connection with no credential is reported to me, not worked around.

On success, summarise in no more than ten lines: what the task is, its status, what its wrapup says
the implementation currently is, which steps are still open, and anything in its notes that changes
how the work should be done.

Then get on with it. What you build comes from one of two places, and whether the task has a
checklist is what decides which.

**With open steps, the steps are the work and the description is not.** Read the description for
what the task is for, what the work has to satisfy, what is out of scope and what shape the code has
to take, and then build what the open steps say, in the way the description requires. It is context
you hold while implementing a step, not a list of things to implement.

Nothing that only the description asks for gets built in a session with a checklist, however plainly
it is phrased. A description is written once and does not shrink as the work is done, so it goes on
naming things that are finished, and treating it as a list of instructions is how the same thing is
built twice. If something in it still needs doing and no open step covers it, say so in one line and
ask me for the step. Do not add it to what you are building.

**With no steps at all, the description is the brief and the instruction both.** It is where the
work is written down, and it was written so that it would not have to be said again: take it as the
instruction it is, say in one line what you are about to do, and start. A question in it is a
question to answer, not one to hand back.

Ask only when the context does not settle it:

- there is no description and no step, or what they hold is background rather than an instruction
- an open step and the description ask for two things that cannot both be true, or one of them names
  something that is not there
- the description asks for work no open step covers and you think it has to happen now
- doing it would delete or overwrite something nothing keeps a copy of

Then ask exactly what is missing, in one question, and stop. Nothing already written down is worth a
question: if it is in the description, a step, the wrapup or a note, it has been answered once
already.

### The checklist

If the task carries steps, that is the plan and it decides what you do next.

- The open ones are the work, in the order they are listed, and each arrives with the detail of what
  it has to do. Start on the first one that is open.
- The done ones arrive as a title and nothing else. They are finished: do not build them again, and
  do not go looking for the detail that is not there.
- Where the checklist and the wrapup disagree, the checklist is the one a person ticked. A step that
  is ticked is done however the description still phrases it, and that is not a contradiction worth
  asking me about.

You cannot tick a step **done**, and no tool here can: that is mine to do in the console once I have
looked at the work. What you can do, while you work a checklist, is move a step to **running** and
then **claimed** with `/rk ... step:N start` and `/rk ... step:N done` (see **Stepping**). When you
claim one, say which, in the words the step uses.

### The flow

If the `<steps>` tag carries `flow="true"`, the steps are a graph, not a list: each one names its
routes (`app-error` -> step 4), and the order they are listed in is not the order they are worked.

- Work the step the tag names as `current`. If none is current, start where the `flow` field says the
  flow begins.
- When it ends, claim it with what happened: `/rk ... step:N done outcome:app-error`. Rekall follows
  that route, opens the step it leads to (again, if it was finished) and makes it current. The answer
  names it: start that one next, and keep going until a claim answers that the flow is finished.
- A step with one route needs no outcome, and a step with no routes ends the flow. A route that reads
  `-> end of the flow` ends it too: claim with that outcome and the answer says the flow is finished,
  with nothing left current. A claim whose
  outcome names no route is refused with the routes there are: pick the one that happened, never
  invent one.
- When the outcome is a failure, say what you saw: `/rk ... step:N done outcome:failed report:"job
  test: expected 1.2, got 1.3"`, passed as `report`. Name the failing job or check, the error and where
  it points. The step the flow goes to shows it in an `<arrived-from>` block: that block, not your
  memory of this session, is what the next pass works from.
- A step reached again is another pass. Read its detail again and build on what the last pass left.
- When the flow met failures, the wrapup written at its end records them as results: one row per
  failure, what fixed it and where the fix lives.

**A flow runs to its end without me.** Claiming a step on a flow is not a hand-off: Rekall has already
opened the next one. Do not ask me to tick, review or accept anything between steps, do not stop to say
a step is waiting for the console, and do not write a wrapup in between. Start the step the claim names
right away, in the same session. Stop only when a claim answers that the flow is finished, or when a
step cannot be completed and none of its routes describes why: say what blocks it and stop there. A
claim refused because the next step's visit budget is spent is such a stop: the loop has gone round as
often as I allowed, so tell me what keeps sending the work back and stop. When the flow is finished, write the wrapup once (see **Wrapping up**) and tell me the claimed steps are
waiting for my review.

Anchors that name a project and no task have no brief in them. Summarise what is there and wait.

### What the work produces

When a step or the description asks for something to be kept as a note (an analysis, a list, a
draft, a runbook, "put the result in a note"), or the task exists to write one, that output goes
to `rekall_note` on this task, not into the wrapup. The wrapup can say the note exists; it does not
carry it. See **Noting**.

When a step or the description asks for documentation about the project as a whole in a section
(`create a documentation in a section named Deployment`), that goes to `rekall_project_section`
under the title I named, not to a note or the wrapup. See **Sectioning**.

## Stepping

`/rk project:vega task:report-builder step:3 start` and `/rk project:vega task:report-builder step:3
done` are how a session drives its own checklist as it works. `step:` takes the step's number as
`rekall_context` lists them, or its exact title.

- `start` marks that step **running**: call `rekall_step` with `state` `running`.
- `done` marks it **claimed**: call `rekall_step` with `state` `claimed`. It does not tick the box.
  Only I do that, in the console, once I have looked at the work.
- `outcome:<name>` alongside `done`, on a flow, is what happened: pass it as `outcome`. Rekall routes
  by it to the next step (see **The flow**).
- `report:"..."` alongside `done`, on a flow, is what you saw there: pass it as `report`. Give one with
  every failure.

The loop, once you are working a checklist:

1. `/rk project:x task:y step:N start`, before you touch the code for step N.
2. Do the work the step's detail describes.
3. `/rk project:x task:y wrapup`, to fold what that step built into the task's wrapup, by the rules
   in **Wrapping up**. One state, rewritten whole, never a section per step. On a flow, skip this
   until the flow is finished.
4. `/rk project:x task:y step:N done`, to claim it.
5. If a later step is still open, `/rk project:x task:y step:M start` and go again, unless it opens a
   new session (see **Sessions**). When none is, stop and tell me the checklist is claimed and waiting
   for my review. On a flow, the next step is the one the claim's answer names and you start it without
   stopping; the loop ends when the answer says the flow is finished, and only then do you write the
   wrapup.

**Sessions.** A step that reads `(opens a new session)` is the first of a session of its own, and the `<steps>` tag
carries `sessions="N"` when there is more than one. A step without it belongs to the session the step
before it ran in. A checklist with no such step is one session from the first step to the last.

The wrapup is what the next session starts from, so write it before you claim the last step of a
session. When the step you claimed is followed by one that opens a new session, do not start that one
here, on a flow too: say which step the next session opens on and stop. Rekall closes this terminal and
opens a new one on that step. The `step:M start` of that next step is the new session's to make.

`rekall_step` refuses `done` as a state, and refuses a step I have already accepted. If it comes
back with either, say so and stop.

If the project's context carries an `auto-commit` field, the claim is what commits: `step:N done`
(and, on a task with no checklist, the wrapup) stages and commits the folder and logs the commit
itself. Do not `git commit` on such a project; read the answer for what it committed, or why not.

On such a project, pass `commit_message` with the claim (with `rekall_wrapup` on a task with no
checklist), because it is the commit's message:

- The first line is a Conventional Commits subject under 72 characters, `feat: …` or `fix: …`, saying
  what the work delivers, not the step's number.
- Then a blank line and two to five sentences of plain prose: what the change does, why, and anything
  a reader of `git log` should know. No list of files, no "this session", no anchors: Rekall adds a
  `Refs:` line with the task and step itself.

Without it, Rekall derives the message from the step's title and detail, which is the fallback and
reads like one.

## Noting

`/rk project:vega task:report-builder note "the export formats we settled on"` means: write a note
on that task and attach it there, or rewrite the one it already carries on that subject.

The text in double quotes after `note` says what the note has to hold, in my words. It is not an
anchor and not the note's text: it is what to put in it. Without one, the note holds what this
session just produced that I would want again.

1. Drop the `note` term and the quoted text, and call `rekall_context` with the anchors that are
   left, unless that exact task is already loaded in this session.
2. Write the note from what the quoted text asks for: the output of a step, a piece of the
   description worked out, something the session found. Written to be read on its own by someone
   who was not here, in markdown.
3. Call `rekall_note` with those anchors, a short `title` named like a file (`export-formats.md`)
   and the whole note as `body`. If the task already carries a note this one supersedes (the same
   subject, an older version of the same results), use that note's title and pass `replace: true`,
   so the task keeps one current note instead of two.
4. Say in one line which note you wrote or rewrote and its `note:` anchor. Then stop.

`replace` overwrites the body outright: the old text is not kept anywhere, and a note shared with
other tasks changes for them too, so send the whole note, not the part that changed. Without it, a
title the task's notes already carry is refused. A note is not the wrapup, and writing one does not
claim the task or a step.

## Sectioning

`/rk project:vega task:report-builder section "Deployment" "how we ship to production"` means: write a
section named Deployment on that task's project, or rewrite the one it already has under that title.

A section is a page of the project, shown beside its blueprint in the console. It is loaded into no
context, so it costs no tokens and a later session reads it only if I paste it in. Use it for what is
true of the project as a whole (deployment, conventions, architecture, a glossary), never for what
one task built (that is the wrapup) or produced (that is a note).

The first quoted text is the section's title, exactly as I typed it. The second, if there is one,
says what it has to hold, in my words; it is not the section's text. Without it, the section holds
what the task or the step that asked for it describes. This is also what a step like "create a
documentation in a section named xxx" means: the name is the title, the rest of the step is the content.

1. Drop the `section` term and the quoted texts, and call `rekall_context` with the anchors that are
   left, unless that exact task is already loaded in this session.
2. Write the section from the code and from what the quoted text asks for, in markdown, to be read on
   its own by someone who was not here.
3. Call `rekall_project_section` with those anchors, the `title` and the whole section as `body`. If
   the project already has a section under that title and this one supersedes it, pass `replace: true`.
4. Say in one line which section you wrote or rewrote and on which project. Then stop.

`replace` overwrites the body outright and no history keeps the old one, so send the whole section,
not the part that changed. Without it, a title the project already uses is refused. Nothing here
renames, reorders or deletes a section, and writing one does not claim the task or a step.

## Planning

`/rk project:vega task:report-builder plan` means: turn the task into a checklist I can review.

1. Drop the `plan` term, and `flow` after it, and call `rekall_context` with the anchors that are left.
2. Read what it gives you: the description says what the work is for and what it has to satisfy,
   the wrapup what already exists, the done steps what is finished. Then read the code the task
   touches, enough to know where each piece will go.
3. Decide whether the work is a checklist or a flow. It is a flow when a step's outcome decides
   what comes next: tests that can fail and send the work to a fix, a deploy that can break, a
   review that can reject. Work that only goes from one step to the next is a checklist, however
   long. The bare word `flow` after `plan` (`/rk ... plan flow`) asks for a flow outright.
4. For a checklist, call `rekall_propose_step` once per step, in the order the work should be done.
   For a flow, call `rekall_propose_flow` once with every step in order, the first where it begins,
   and each step's routes: `{"outcome": "app-error", "to": 3}`, or `"to": "end"`. Name outcomes for
   what happened (`passed`, `app-error`), give a step that simply leads on one route to the next,
   leave the steps that finish the work without routes, and give every loop a way out. Give the
   step a loop returns to a `max_visits` (3 is a sensible start), so the flow stops instead of
   going round forever. Either way, a
   title says what the step delivers; the detail says what to build, where, what it must satisfy
   and how I can tell it is done, enough for a session that never saw this one to do it alone.
   Propose only what is still missing, and nothing the description rules out.
5. Stop. Say in a few lines how many drafts (and, for a flow, routes) you proposed and anything
   you left out on purpose. Build nothing: the drafts are not work until I promote them in the
   console.

A title the task already has is refused, so a second `plan` adds only what the first one missed. A
flow is proposed only onto a task with no steps; on one that has some, plan a checklist and say the
routes are mine to draw.

## Generating

`/rk project:vega generate kinds:sequence,state "how checkout decides what to charge"` means: work
out what the code does to answer that request, describe it as Mermaid sheets whose elements are
linked to the code, and store them with `rekall_diagram`. The console draws them; you never do.
A `task:` anchor may come too (`/rk project:vega task:report-builder generate ...`): the diagram is
then stored on that task as well as on its project.

The text in double quotes is the request, in my words: what the diagram has to make clear. It is not
an anchor. `kinds:` lists the Mermaid types I asked for, one sheet each, in that order; it is not an
anchor either.

1. Drop the `generate` term, the `kinds:` term and the request, and call `rekall_context` with the
   anchors that are left. The description, wrapup and notes tell you what the project is and where
   things live. They are not work: in this mode you build nothing, and no checklist changes.
2. Find the code yourself. Your working directory is the project's folder, and every path you link
   is relative to it. Do not ask me where the project is, which files to read, which functions
   matter or where to save: search, read, follow calls and references until you can answer the
   request, and read only what it needs.
3. Write one sheet per type in `kinds:`. Without `kinds:`, choose what answers the request: a
   sequence for who calls whom and in what order, a state diagram for a lifecycle, a flowchart for
   the decisions inside one piece of code, a class or ER diagram for structure.
4. Describe the behaviour, not the file layout. In a sequence the participants are the components
   that talk (types, modules, services, external systems), each message is a call labelled with
   what is called (`charge(card)`), its dashed reply carries what comes back, `+`/`-` activations
   nest the calls the way the code does, and branches and repetition go in `alt`, `opt` and `loop`
   with their condition. A 200-line function that validates, charges, stores and notifies is four
   messages or four nodes, each linked to its own lines. A status field set on different branches
   is a state diagram even when the code never calls it one.
5. Keep to what the code supports. Link every participant, message, node, state, class or entity
   that has a place in the code to that place in `links`, and read those lines before you cite them:
   a participant to its type, a message to the body of the method it calls. Never draw a call, a
   transition or a write you did not see.
6. Call `rekall_diagram` with the anchors, a short `title`, the request as `question` and every
   sheet. Its description carries the full format: follow it, and answer the request rather than the
   whole codebase. If the answer is a refusal, fix every line it lists and send every sheet again.
7. Say in two or three lines what the diagram shows, how many elements are linked to code, and its
   id. Then stop: no wrapup, no step, no commit. A diagram is not the task's work.

The sheets are the whole output. Never HTML, SVG, Vue or styling, and no rendition of the diagram in
the chat either. To revise a diagram whose id you have, send every sheet again with that id as
`diagram`, keeping the ids of the elements that stay.

## Explaining

`/rk project:vega explain "how an order is placed"` means: walk through the code that answers the
request as a **walkthrough**, a diagram the console plays one message at a time, showing the method
called, its code, what goes in and what comes back, and store it with `rekall_diagram`. A `task:`
anchor may come too, as in **Generating**.

The text in double quotes is the request, in my words: the functionality I want to understand. It is
not an anchor. A walkthrough exists because a static diagram or a written explanation was not enough,
so it follows the code concretely, call by call.

1. Drop the `explain` term and the request, and call `rekall_context` with the anchors that are left.
   As in **Generating**, you build nothing and no checklist changes.
2. Find the entry point the request starts from: the controller handler, the command, the scheduled
   job or the listener that is triggered first. Follow the calls from there through the layers, and
   read every method you are going to show.
3. Sheet 0 is the entry. It is a `sequenceDiagram` whose first message is the call into the entry
   point, from an `actor` or the external caller, and whose participants follow the call through the
   layers in order (controller, service, repository...). Every system outside the project's code (a
   database, an HTTP API, a queue, the filesystem) is a participant of its own, and its id goes in
   `walkthrough.external`.
4. Each call is a message labelled with the method and its arguments (`place(order)`), linked by `#n`
   to the method's body, with a dashed reply carrying what comes back. Give each call an entry in
   `walkthrough.messages` under the same `#n`: `explain` says what the step does and why, in one or
   two sentences drawn from the code; `input` what the method receives; `output` what it returns, or
   the error it raises. A reply needs an entry only when it says something its call does not.
5. A call whose own implementation is worth walking through gets a sequence sheet of its own, which
   starts at that call's receiver and whose title names the call. The call's entry points at it with
   `opens`, the sheet's index in `sheets`, counted from 0. The console opens it when the reader steps
   into the call and comes back when it ends. Nest as deep as the code needs, within 8 sheets.
6. Keep to what the code supports. `input` and `output` are values of the types the code declares: a
   short example when the code makes it concrete (`{ sku: "A-7", qty: 2 }`), the type's shape when it
   does not (`Order { lines: Vec<Line> }`). Never show a call, a value or a branch you did not see.
7. Call `rekall_diagram` with the anchors, a short `title`, the request as `question` and every sheet.
   If the answer is a refusal, fix every line it lists and send every sheet again.
8. Say in two or three lines what the walkthrough covers, how many messages and sheets it plays, and
   its id. Then stop: no wrapup, no step, no commit.

## Inventorying

`/rk project:vega inventory` means: record what the project is built from, every library and
framework at the exact version it uses, with `rekall_inventory`. Rekall checks that list against
the critical advisories of the GitHub Advisory Database and warns on its Security screen.

1. Drop the `inventory` term and call `rekall_context` with the anchors that are left. You build
   nothing and no checklist changes.
2. Find every manifest and lockfile in the project's folder yourself, in every subfolder that is a
   part of the project: `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `Cargo.lock`,
   `requirements*.txt`, `poetry.lock`, `uv.lock`, `Pipfile.lock`, `pom.xml`, `build.gradle(.kts)`,
   `gradle.lockfile`, `go.mod`/`go.sum`, `composer.lock`, `Gemfile.lock`, `*.csproj`,
   `packages.lock.json`, `pubspec.lock`, `Package.resolved`, `mix.lock`, the workflow files under
   `.github/workflows`. Skip `node_modules`, `target`, `vendor`, build output and anything ignored.
3. Read the exact version each dependency resolves to, from the lockfile when there is one. Where a
   manifest only gives a range and there is no lockfile, use the version the build would pick if the
   code makes it plain, and leave the dependency out if it does not: a guessed version raises false
   warnings. Include what a lockfile pins transitively.
4. Call `rekall_inventory` with the project anchor and the whole list, one entry per package and
   version: `ecosystem` by the advisory database's name (`npm`, `pip`, `maven`, `rust`, `go`,
   `nuget`, `rubygems`, `composer`, `pub`, `swift`, `erlang`, `actions`), `name` as that ecosystem
   writes it, `version`, and `manifest` relative to the project's folder. If the answer is a refusal,
   fix what it names and send the whole list again.
5. Say in two or three lines how many dependencies you recorded per ecosystem, what you left out and
   why. Then stop: no wrapup, no step, no commit.

## Assessing

`/rk project:vega assess advisory:GHSA-xxxx-xxxx-xxxx` means: decide whether that advisory
really affects the project, and record the verdict with `rekall_assess_vulnerability`. A scan only
knows the project declares a vulnerable version; whether the flaw can be reached is in the code.

`/rk project:vega assess`, with no `advisory:` term, means the same for every advisory reaching the
project that is still `not assessed`: one verdict each.

1. Drop the `assess` term and the `advisory:` term, and call `rekall_context` with the anchors that
   are left. You change no code and no checklist.
2. With no `advisory:` term, call `rekall_vulnerabilities` with the project anchor alone and take
   every advisory it lists as `not assessed`, in that order. If there is none, say so and stop. Then
   work steps 3 to 5 for each one, recording each verdict before starting the next.
3. Call `rekall_vulnerabilities` with the project anchor and `advisory` set to the GHSA id. It gives
   the package and version, the vulnerable versions, the first safe one, the functions that carry the
   flaw when GitHub names them, the advisory's own description and any earlier conclusion and
   reasoning.
4. Find where the project uses the package: its imports, the calls into it, the configuration that
   turns a vulnerable feature on, and the inputs that reach those calls. Follow them far enough to say
   whether the vulnerable code runs, and on what. Advisories on the same package share this search:
   do it once and judge each against its own vulnerable functions.
5. Call `rekall_assess_vulnerability` with the project anchor, the advisory, a `verdict`, the
   `reasoning` and the `conclusion`. `not_affected` only when the code shows the vulnerable path is
   never taken (the function is never called, the feature is off, no input reaches it); `affected`
   when it is, or when you cannot show it is not.
   - The `reasoning` says what the flaw needs to run and names the files and functions that decide
     whether the project gives it that.
   - The `conclusion` is one or two plain sentences on why this project is or is not affected, in
     terms of what the project does, not what the flaw does: `Rekall never unpacks archives; it only
     reads lockfiles as text.` For `not_affected`, it names the feature or input the project does not
     use, so whoever reads it knows what not to start using while the package stays at this version.
6. Say in two or three lines what you concluded and why; for several, one line per advisory with its
   verdict. Then stop: fixing is a task the console opens.

## Wrapping up

`/rk project:vega task:report-builder wrapup` means: record what that task's implementation looks
like **now**, replacing what was there.

Anything in double quotes after `wrapup` is a **directive**: what I want the wrapup to say, in my
words, not an anchor. `/rk project:vega task:report-builder wrapup "solo il modulo di export"`.

1. Drop the `wrapup` term and the directive, and call `rekall_context` with the anchors that are
   left, so you are working from the current wrapup and notes rather than from memory of this
   session. Skip this only if you already loaded that exact task in this session and nothing has
   changed since.
2. Call `rekall_wrapup` with those same anchors and the complete new text as `body`. The anchors
   string never carries the directive.
3. Say in one line what you wrote and that it replaced what was there. If a directive left something
   out that the previous wrapup had, say what. If the task has a checklist, name the open steps this
   session finished, so I can tick them: nothing you can call will do it for you. Then stop. On a
   flow, do not ask me to tick anything: say the flow is finished and its claimed steps are waiting
   for review.

### The directive

Without one, the wrapup is yours to write: the implementation as this session and the code leave it,
by the rule the tool states.

With one, it decides the content and you do not go past it. It can narrow the subject, dictate the
words, set the language or the length, or say what to leave out. Take it literally.

- "write only what I am telling you" means the body carries what I dictated and nothing you inferred
  from the code. What I did not mention is gone, and step 3 is where you tell me so.
- A directive that names a subject, `"solo il modulo di export"`, narrows what you write about. What
  the rest of the wrapup already says stays true and stays in, unless I said to drop it.
- It never changes the shape. Still one state and not a changelog, still the whole text in one call,
  still short enough to read on a screen. If what I dictate is phrased as a change I made, record
  what that change leaves the system as, not the making of it.

The anchors have to name exactly one task. A `project:` anchor on its own names forty and is refused.

### Steps that closed since the last one

A wrapup written after a step is finished has to account for it, and that is the whole reason to
write one there. Write the text again so it describes the system with that step's work in it. The
result is one description of the whole task, not the previous wrapup with a paragraph stuck on the
end.

You do not have to work out which ones those are. The context marks every finished step the wrapup
predates with `(finished since the wrapup was written)` and hands back its detail, which it does for
no other closed step. Those are exactly what the current text is missing, and it is often more than
the one step this session closed: a step ticked in an earlier session that never got a wrapup is
still marked, and it goes in too. Do not stop at the piece you just built.

- Yes: the piece the step built is now part of what the wrapup says the task is, named where it
  lives, next to what was already there.
- No: a section per step, a heading with the step's title on it, "then I did", anything that lets a
  reader reconstruct the order the pieces arrived in. That is a changelog, and it is what this
  replaces.

If the step's work made something the wrapup already said untrue, the old sentence goes. Keeping
both is how a wrapup starts contradicting itself.

**Write it fresh.** A wrapup does not grow. It is rewritten from the code every time. Read the current
wrapup as a list of claims to check, not as a text to extend: a sentence stays only if the code
still makes it true and the next session needs it to start work. Open points that are closed,
decisions nobody will revisit and detail about finished, stable pieces all go. The new text still
covers the whole task, not only the part you touched.

**Write the state, not the session.** The wrapup describes the system as it stands, for a reader who
was not here and does not care what it looked like before.

- Yes: what exists, what it does, how the parts fit, which decisions are settled, what is still open,
  where the sharp edges are.
- No: "added", "changed", "now also", "previously", "fixed", "refactored", "before/after", anything
  dated, anything phrased as a step you took, anything that reads as a changelog entry.

If a sentence only makes sense to someone who watched the change happen, it does not belong in the
wrapup. That is what the notes are for.

**Name the code, do not transcribe it.** It is read next to the repository, so it has to say where
things are: the class, the file, the endpoint, the table, the component, by the name they have.
A line or two on each piece, what it is there for and what it decides.

- Yes: `WrapupService` is the whole write path and the only thing that touches the wrapup row.
- No: the fields of an object, the columns of a table, method signatures, parameter lists, a
  directory tree. That is in the code, it is longer than the wrapup, and it is wrong a week later.
- Where there is a rule, the rule is the point. What it decides, on what, what happens at the edges,
  what is refused and why. A class name says there is a service; only the wrapup says that
  overwriting a wrapup edited by hand is announced because nothing keeps the text it replaced.

**Shape it to be scanned.** The kind of information chooses the form:

| Information | Form |
|---|---|
| What the task delivers and where it stands | One or two sentences, first |
| The pieces of the implementation | A table: the name in the code, what it is for or decides |
| A rule | One bullet: what it decides, on what, the edges, what is refused |
| Results of something checked or tested | A table, one row per check, its outcome and what was seen |
| A flow or a lifecycle | A numbered list |
| What is open or fragile | Bullets under their own heading, last; left out when nothing is |

`###` headings separate those blocks and nothing else. Prose is only for what connects the parts.
It fits on one screen: about 300 words suits most tasks. Past 600 the tool says so, because the
text is carrying history or detail that belongs in a note.

If the tool answers that the version you replaced had been edited by hand, tell me: those were my
words, and nothing keeps them any more.
