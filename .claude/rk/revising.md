## Revising

`/rk task:RK-12 revise` means: work one pass over this task's thread. The
thread holds the revisions of the description. A pass covers every revision that is waiting or in the
pass, built on top of what already exists.

1. Drop the `revise` term and call `rekall_context` with the anchors that are left.
2. Read the `<thread>` block and pick the entries of this pass:
   - If any entry is `RUNNING`, the pass is those entries (each carries `pass="N"`). A `PENDING` entry
     beside them was written after the pass started: it belongs to the next one, leave it.
   - If none is `RUNNING`, the pass is every `PENDING` entry.
   - An `APPLIED` entry is history, one line of what an earlier session answered. Never redo it.
   With no entry in the pass, say so in one line and stop.
3. Each entry holds the description before (`<quoted>`) and as it is now (`<new>`). Compare them: the
   new text is the requirement, the old one is already built. What the entry is applied to is its
   `kind`:
   - `IMPLEMENTATION`: the work. Implement the difference on top of the existing implementation and
     the wrapup.
   - `PLAN`: the steps. Follow `replanning.md` for these entries, reading the difference as the
     requirement: it redesigns the draft and open steps through `rekall_replan`. This is the one place
     a revision changes the checklist. With no draft or open step to redesign, the reply says there is
     no plan to change.
   Work the `PLAN` entries first, then the `IMPLEMENTATION` ones.
4. Call `rekall_feedback` once with a reply for every entry of the pass, the `PLAN` ones too: one or
   two sentences of what changed and where, or why nothing did. The detail belongs in the wrapup.
   Read the answer's `Still open` list: an entry of this pass in it was forgotten, so reply to it. An
   entry written after the pass started is open on purpose.
5. Finish with `rekall_wrapup`, as `wrapping-up.md` says, describing the system as it now stands.
   The thread carries the pass: the wrapup does not list its entries.

Do not write a note for any of this: the thread and the wrapup carry it.
