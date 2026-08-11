---
description: Load a Rekall working context by anchor, e.g. /rk project:stvv task:code-validator
argument-hint: entity:value [entity:value ...]
allowed-tools: mcp__rekall__rekall_context
---

Call `rekall_context` with `anchors` set to exactly this, unchanged:

$ARGUMENTS

Then:

- Do not call any other tool first. The anchors are already qualified, there is nothing to look up.
- The entities are `project`, `task` and `environment`. If one was wrong, say which and stop.
- If a term matched more than one record, show me the candidates and stop. Do not choose.

On success, summarise in no more than ten lines: what the task is, its status, the environment it
runs on, and anything in its notes that changes how the work should be done. Then wait. Do not
start editing, do not propose a plan, do not read files from the repository until I ask.
