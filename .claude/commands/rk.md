---
description: Load a Rekall working context by anchor, e.g. /rk project:vega task:report-builder
argument-hint: "company:|project:|task: <label> ...  [wrapup]   (labels, not titles)"
allowed-tools: mcp__rekall__rekall_context, mcp__rekall__rekall_wrapup
---

The arguments are:

$ARGUMENTS

If the last term is the bare word `wrapup`, follow **Wrapping up**. Otherwise follow **Loading**.

## Loading

Call `rekall_context` with `anchors` set to exactly the arguments above, unchanged.

- Do not call any other tool first. The anchors are already qualified, there is nothing to look up.
- The entities are `company`, `project` and `task`. If one was wrong, say which and stop.
- The value is the record's **label**, never its title: `project:vega`, not `project:"Vega Platform"`.
  A label is lowercase and has no spaces. If I gave you a title, say so and stop rather than guessing
  the label from it.
- If a term matched more than one record, show me the candidates and stop. Do not choose.

On success, summarise in no more than ten lines: what the task is, its status, what its wrapup says
the implementation currently is, and anything in its notes that changes how the work should be done.
Then wait. Do not start editing, do not propose a plan, do not read files from the repository until
I ask.

## Wrapping up

`/rk project:vega task:report-builder wrapup` means: record what that task's implementation looks
like **now**, replacing what was there.

1. Drop the trailing `wrapup` and call `rekall_context` with the anchors that are left, so you are
   working from the current wrapup and notes rather than from memory of this session. Skip this only
   if you already loaded that exact task in this session and nothing has changed since.
2. Call `rekall_wrapup` with those same anchors and the complete new text as `body`.
3. Say in one line what you wrote and that it replaced what was there. Then stop.

The anchors have to name exactly one task. A `project:` anchor on its own names forty and is refused.

**Write the state, not the session.** The wrapup describes the system as it stands, for a reader who
was not here and does not care what it looked like before.

- Yes: what exists, what it does, how the parts fit, which decisions are settled, what is still open,
  where the sharp edges are.
- No: "added", "changed", "now also", "previously", "fixed", "refactored", "before/after", anything
  dated, anything phrased as a step you took, anything that reads as a changelog entry.

If a sentence only makes sense to someone who watched the change happen, it does not belong in the
wrapup. That is what the notes are for.

Send it whole. Nothing is merged and no previous version is kept, so keep whatever is still true
from the wrapup you just read and rewrite the rest. Describing only the part you touched would leave
the task claiming to be a fraction of itself.

If the tool answers that the version you replaced had been edited by hand, tell me: those were my
words and nothing kept a copy.
