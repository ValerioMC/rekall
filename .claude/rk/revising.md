## Revising

`/rk project:vega task:report-builder revise` means: work one pass over this task's thread. A pass
covers every entry that is waiting or in the pass, of any kind, built on top of what already exists.

1. Drop the `revise` term and call `rekall_context` with the anchors that are left.
2. Read the `<thread>` block and pick the entries of this pass:
   - If any entry is `RUNNING`, the pass is those entries (each carries `pass="N"`). A `PENDING` entry
     beside them was written after the pass started: it belongs to the next one, leave it.
   - If none is `RUNNING`, the pass is every `PENDING` entry.
   - An `APPLIED` entry is history, one line of what an earlier session answered. Never redo it.
   - An entry with `follows="S"` continues entry S: read S's reply line, then do what the follow-up
     adds on top of that answer.
   With no entry in the pass, say so in one line and stop.
3. Work the entries by `kind`, in this order:
   - `PLAN`: feedback on the steps. Follow `replanning.md` for these entries: it redesigns the draft
     and open steps through `rekall_replan`. This is the one place a revision changes the checklist.
     With no draft or open step to redesign, the reply says there is no plan to change.
   - `REVISION`: the description was rewritten. The entry holds the text before (`<quoted>`) and the
     text now (`<new>`). Compare them and implement the difference on top of the existing
     implementation and the wrapup: the new text is the requirement, the old one is already built.
   - `IMPLEMENTATION`: a correction of the work already done. Treat it as the requirement for what
     to change, and fix the code the entry points at.
4. Call `rekall_feedback` once with a reply for every entry of the pass, the `PLAN` ones too: one or
   two sentences of what changed and where, or why nothing did. The detail belongs in the wrapup.
   Read the answer's `Still open` list: an entry of this pass in it was forgotten, so reply to it. An
   entry written after the pass started is open on purpose.
5. Finish with `rekall_wrapup`, as `wrapping-up.md` says, describing the system as it now stands.
   The thread carries the pass: the wrapup does not list its entries.

Do not write a note for any of this: the thread and the wrapup carry it.
