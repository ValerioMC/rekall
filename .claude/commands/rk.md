---
description: Load a Rekall working context by anchor, e.g. /rk project:stvv task:code-validator
argument-hint: entity:value [entity:value ...]
allowed-tools: mcp__rekall__rekall_context, mcp__rekall__rekall_schema
---

Call `rekall_context` with `anchors` set to exactly this, unchanged:

$ARGUMENTS

Then:

- Do not call any other tool first. The anchors are already qualified, there is nothing to look up.
- If the call is refused because an entity part is unknown, call `rekall_schema` once, tell me which
  anchor was wrong and what the valid names are, and stop.
- If a term matched more than one record, show me the candidates and stop. Do not choose.

On success, summarise in no more than ten lines: what the task is, its status, the environment it
runs on, and anything in its documents that changes how the work should be done. Then wait. Do not
start editing, do not propose a plan, do not read files from the repository until I ask.
