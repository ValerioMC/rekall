---
description: Load a Rekall working context by anchor, e.g. /rk project:vega task:report-builder
argument-hint: "company:|project:|task: <label> ...  (labels, not titles)"
allowed-tools: mcp__rekall__rekall_context
---

Call `rekall_context` with `anchors` set to exactly this, unchanged:

$ARGUMENTS

Then:

- Do not call any other tool first. The anchors are already qualified, there is nothing to look up.
- The entities are `company`, `project` and `task`. If one was wrong, say which and stop.
- The value is the record's **label**, never its title: `project:vega`, not `project:"Vega Platform"`.
  A label is lowercase and has no spaces. If I gave you a title, say so and stop rather than guessing
  the label from it.
- If a term matched more than one record, show me the candidates and stop. Do not choose.

On success, summarise in no more than ten lines: what the task is, its status, and anything in its
notes that changes how the work should be done. Then wait. Do not start editing, do not propose a
plan, do not read files from the repository until I ask.
