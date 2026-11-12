---
description: Load a Rekall working context by anchor, e.g. /rk project:vega task:report-builder
argument-hint: "company:|project:|task: <label> ...  [wrapup [\"how to write it\"]] | [note \"what to keep\"] | [section \"Title\" [\"what it holds\"]] | [plan [flow]] | [replan] | [revise] | [rewrite fix|restructure] | [generate [kinds:sequence,state] \"what to diagram\"] | [explain \"what to walk through\"] | [inventory] | [assess [advisory:GHSA-xxxx-xxxx-xxxx]] | [autoconfigure] | [merge [pull-request base:<branch>]] | [step:N start|done [outcome:<name>] [report:\"what you saw\"]]   (labels, not titles)"
allowed-tools: mcp__rekall__rekall_context, mcp__rekall__rekall_wrapup, mcp__rekall__rekall_step, mcp__rekall__rekall_propose_step, mcp__rekall__rekall_propose_flow, mcp__rekall__rekall_replan, mcp__rekall__rekall_feedback, mcp__rekall__rekall_description, mcp__rekall__rekall_delete_step, mcp__rekall__rekall_record_commit, mcp__rekall__rekall_note, mcp__rekall__rekall_project_section, mcp__rekall__rekall_read_sections, mcp__rekall__rekall_search, mcp__rekall__rekall_diagram, mcp__rekall__rekall_inventory, mcp__rekall__rekall_vulnerabilities, mcp__rekall__rekall_assess_vulnerability, Read, Grep, Glob, WebFetch
---

The arguments are:

$ARGUMENTS

If the terms include one of the words below, the mode is the first row that matches, in this order. A
`step:` term counts only alongside `start` or `done`. With none of them, follow **Loading** below.
Words inside double quotes are never terms: `generate "how the plan is noted"` is generating, not
planning or noting.

| Bare word in the terms | File |
|---|---|
| `generate` | `generating.md` |
| `explain` | `explaining.md` |
| `inventory` | `inventorying.md` |
| `assess` | `assessing.md` |
| `autoconfigure` | `autoconfiguring.md` |
| `merge` | `merging.md` |
| `wrapup` | `wrapping-up.md` |
| `note` | `noting.md` |
| `section` | `sectioning.md` |
| `plan` | `planning.md` |
| `replan` | `replanning.md` |
| `revise` | `revising.md` |
| `rewrite` | `rewriting.md` |
| `step:` with `start` or `done` | `stepping.md` |

When the terms pick a mode, read its file with Read before doing anything else, and follow only that.
The files are in `~/.claude/rk/`; if one is not there, read it from `.claude/rk/` in the working
directory. `wrapping-up.md` is the one place the wrapup rules live: the other modes and the sections
below point to it.

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
- A `<thread>` block holds my revisions of the description, each entry `PENDING` (waiting) or `RUNNING`
  (taken into a pass) until a session replies. Open entries are worked only through `revise` or `replan`,
  never during a plain `/rk` run.
- A `<sections>` block lists the project's pages by title; their text is not loaded. When the work
  touches what one names, read it with `rekall_read_sections` and follow it: a `Contributing` page is
  how this project takes a change, so filing an issue, naming a branch or opening a pull request goes
  the way it says.

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

When that work is done, finish it yourself with `rekall_wrapup` (see `wrapping-up.md`): the wrapup is
what claims a task with no steps, and on an `auto-commit` project it is also what commits. Do not wait
for me to ask for it, and do not stop before it: a run queue does not go on to the next task while
this one is unclaimed.

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
then **claimed** with `/rk ... step:N start` and `/rk ... step:N done` (see `stepping.md`). When you
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
often as I allowed, so tell me what keeps sending the work back and stop. When the flow is finished, write the wrapup once (see `wrapping-up.md`) and tell me the claimed steps are
waiting for my review.

Anchors that name a project and no task have no brief in them. Summarise what is there and wait.

### Departing from the plan

A design in a step, a diagram in its detail, in a note or in the description is what was planned, not
a contract. While you build, the code can show something better than what was drawn: a layer that
already does the job, a type that fits, a simpler sequence. You may build the better solution. You
may not do it silently.

- Depart only for a reason you can state: the plan cannot be built as drawn, or the other way is
  plainly simpler or more correct. Not for taste.
- Record it with `rekall_deviation` on the task, for the step it refers to: its number, counted from
  1 as the checklist lists it. It is not a note: it is kept apart from the task's notes and the
  console lists it as a deviation of that step. It says what was planned, what was built instead,
  why, and the diagram of the piece as built. The steps still to do that this changes are named in it
  by number.
- A step's detail cannot be edited from here, so the deviation is where the change lives. Recording
  again for the same step rewrites that deviation, so send the whole text.
- The wrapup describes the system as it is and does not carry the difference; it can say a deviation
  exists.

### What the work produces

When a step or the description asks for something to be kept as a note (an analysis, a list, a
draft, a runbook, "put the result in a note"), or the task exists to write one, that output goes
to `rekall_note` on this task, not into the wrapup. The wrapup can say the note exists; it does not
carry it. See `noting.md`.

When a step or the description asks for documentation about the project as a whole in a section
(`create a documentation in a section named Deployment`), that goes to `rekall_project_section`
under the title I named, not to a note or the wrapup. See `sectioning.md`.
