## Revising

`/rk project:vega task:report-builder revise` means: evaluate the pending entries of this task's
thread and build what they ask for, on top of what already exists.

1. Drop the `revise` term and call `rekall_context` with the anchors that are left.
2. Read the `<thread>` block. Act only on the entries whose `state` is `PENDING`: an `APPLIED` entry
   is history, one line of what an earlier session answered, and it is never redone. With no
   `PENDING` entry, say so in one line and stop.
3. Work each pending entry by its `kind`:
   - `REVISION`: the description was rewritten. The entry holds the text before (`<quoted>`) and the
     text now (`<new>`). Compare them and implement the difference on top of the existing
     implementation and the wrapup: the new text is the requirement, the old one is already built.
   - `IMPLEMENTATION`: a correction of the work already done. Treat it as the requirement for what
     to change, and fix the code the entry points at.
   - `PLAN`: feedback on the steps. It is worked through `replan` (see `replanning.md`), not here:
     leave it `PENDING` and say so.
4. When the task has open steps, the checklist is not yours to change here: if the entries need a
   different plan, say a replan is my call and leave the steps as they are.
5. Call `rekall_feedback` once with a reply for every `REVISION` and `IMPLEMENTATION` seq you acted
   on: one or two sentences of what changed and where. The detail belongs in the wrapup.
6. Finish with `rekall_wrapup`, as `wrapping-up.md` says, describing the system as it now stands.

Do not write a note for any of this: the thread and the wrapup carry it.
